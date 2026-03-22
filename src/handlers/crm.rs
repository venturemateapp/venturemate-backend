// Phase 2: CRM Handler
use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::utils::get_user_id;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/crm")
            .route("/dashboard", web::get().to(get_dashboard_stats))
            .route("/contacts", web::get().to(list_contacts))
            .route("/contacts", web::post().to(create_contact))
            .route("/contacts/{id}", web::get().to(get_contact))
            .route("/contacts/{id}", web::put().to(update_contact))
            .route("/contacts/{id}", web::delete().to(delete_contact))
            .route("/deals", web::get().to(list_deals))
            .route("/deals", web::post().to(create_deal))
            .route("/deals/{id}", web::get().to(get_deal))
            .route("/deals/{id}", web::put().to(update_deal))
            .route("/deals/{id}/stage", web::patch().to(update_deal_stage))
            .route("/activities", web::get().to(list_activities))
            .route("/activities", web::post().to(create_activity))
            .route("/activities/{id}/complete", web::post().to(complete_activity))
    );
}

// Dashboard
async fn get_dashboard_stats(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let total_contacts = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM contacts WHERE business_id = $1"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let total_deals = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM deals WHERE business_id = $1"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let pipeline_value = sqlx::query_scalar::<_, Option<f64>>(
        "SELECT COALESCE(SUM(value), 0) FROM deals WHERE business_id = $1 AND stage NOT IN ('closed_won', 'closed_lost')"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(Some(0.0));
    
    let deals_by_stage = sqlx::query_as::<_, (String, i64, Option<f64>)>(
        "SELECT stage, COUNT(*), COALESCE(SUM(value), 0) FROM deals WHERE business_id = $1 GROUP BY stage"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    let recent_activities = sqlx::query_as::<_, Activity>(
        r#"SELECT * FROM activities 
           WHERE business_id = $1 
           ORDER BY created_at DESC LIMIT 10"#
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    let stats = CrmDashboardStats {
        total_contacts,
        total_deals,
        total_pipeline_value: pipeline_value.unwrap_or(0.0),
        deals_by_stage: deals_by_stage.into_iter()
            .map(|(stage, count, value)| StageCount {
                stage,
                count,
                total_value: value.unwrap_or(0.0),
            })
            .collect(),
        recent_activities,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(stats)))
}

// Contacts
async fn list_contacts(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let offset = query.offset();
    let limit = query.limit();
    
    let contacts = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE business_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(business_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(contacts)))
}

async fn create_contact(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateContactRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let contact = sqlx::query_as::<_, Contact>(
        r#"INSERT INTO contacts (business_id, name, email, phone, company, job_title, contact_type, source, notes, tags, custom_fields)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '[]', '{}')
           RETURNING *"#
    )
    .bind(business_id)
    .bind(&body.name)
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&body.company)
    .bind(&body.job_title)
    .bind(&body.contact_type)
    .bind(&body.source)
    .bind(&body.notes)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(contact)))
}

async fn get_contact(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let contact_id = path.into_inner();
    
    let contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE id = $1 AND business_id = $2"
    )
    .bind(contact_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    match contact {
        Some(c) => Ok(HttpResponse::Ok().json(ApiResponse::success(c))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Contact not found"))),
    }
}

async fn update_contact(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateContactRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let contact_id = path.into_inner();
    
    let contact = sqlx::query_as::<_, Contact>(
        r#"UPDATE contacts 
           SET name = $3, email = $4, phone = $5, company = $6, job_title = $7, 
               contact_type = $8, source = $9, notes = $10
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(contact_id)
    .bind(business_id)
    .bind(&body.name)
    .bind(&body.email)
    .bind(&body.phone)
    .bind(&body.company)
    .bind(&body.job_title)
    .bind(&body.contact_type)
    .bind(&body.source)
    .bind(&body.notes)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    match contact {
        Some(c) => Ok(HttpResponse::Ok().json(ApiResponse::success(c))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Contact not found"))),
    }
}

async fn delete_contact(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let contact_id = path.into_inner();
    
    sqlx::query("DELETE FROM contacts WHERE id = $1 AND business_id = $2")
        .bind(contact_id)
        .bind(business_id)
        .execute(pool.get_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::NoContent().finish())
}

// Deals
async fn list_deals(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let offset = query.offset();
    let limit = query.limit();
    
    let deals = sqlx::query_as::<_, Deal>(
        "SELECT * FROM deals WHERE business_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(business_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(deals)))
}

async fn create_deal(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateDealRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let currency = body.currency.clone().unwrap_or_else(|| "USD".to_string());
    
    let deal = sqlx::query_as::<_, Deal>(
        r#"INSERT INTO deals (business_id, contact_id, title, description, value, currency, stage, probability, expected_close_date, custom_fields)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, '{}')
           RETURNING *"#
    )
    .bind(business_id)
    .bind(body.contact_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.value)
    .bind(&currency)
    .bind(&body.stage)
    .bind(body.probability.unwrap_or(0))
    .bind(body.expected_close_date)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(deal)))
}

async fn get_deal(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let deal_id = path.into_inner();
    
    let deal = sqlx::query_as::<_, Deal>(
        "SELECT * FROM deals WHERE id = $1 AND business_id = $2"
    )
    .bind(deal_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    match deal {
        Some(d) => Ok(HttpResponse::Ok().json(ApiResponse::success(d))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Deal not found"))),
    }
}

async fn update_deal(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<CreateDealRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let deal_id = path.into_inner();
    
    let deal = sqlx::query_as::<_, Deal>(
        r#"UPDATE deals 
           SET contact_id = $3, title = $4, description = $5, value = $6, 
               stage = $7, probability = $8, expected_close_date = $9
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(deal_id)
    .bind(business_id)
    .bind(body.contact_id)
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.value)
    .bind(&body.stage)
    .bind(body.probability.unwrap_or(0))
    .bind(body.expected_close_date)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    match deal {
        Some(d) => Ok(HttpResponse::Ok().json(ApiResponse::success(d))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Deal not found"))),
    }
}

async fn update_deal_stage(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let deal_id = path.into_inner();
    let stage = body["stage"].as_str().unwrap_or("");
    
    let deal = sqlx::query_as::<_, Deal>(
        "UPDATE deals SET stage = $3 WHERE id = $1 AND business_id = $2 RETURNING *"
    )
    .bind(deal_id)
    .bind(business_id)
    .bind(stage)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    match deal {
        Some(d) => Ok(HttpResponse::Ok().json(ApiResponse::success(d))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Deal not found"))),
    }
}

// Activities
async fn list_activities(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let offset = query.offset();
    let limit = query.limit();
    
    let activities = sqlx::query_as::<_, Activity>(
        "SELECT * FROM activities WHERE business_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(business_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(activities)))
}

async fn create_activity(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateActivityRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let activity = sqlx::query_as::<_, Activity>(
        r#"INSERT INTO activities (business_id, contact_id, deal_id, activity_type, title, description, scheduled_at, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING *"#
    )
    .bind(business_id)
    .bind(body.contact_id)
    .bind(body.deal_id)
    .bind(&body.activity_type)
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.scheduled_at)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(activity)))
}

async fn complete_activity(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let activity_id = path.into_inner();
    
    let activity = sqlx::query_as::<_, Activity>(
        "UPDATE activities SET completed_at = NOW() WHERE id = $1 AND business_id = $2 RETURNING *"
    )
    .bind(activity_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;
    
    match activity {
        Some(a) => Ok(HttpResponse::Ok().json(ApiResponse::success(a))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Activity not found"))),
    }
}

async fn get_default_business_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, actix_web::Error> {
    let business_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM businesses WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .ok_or_else(|| actix_web::error::ErrorBadRequest("No business found"))?;
    
    Ok(business_id)
}
