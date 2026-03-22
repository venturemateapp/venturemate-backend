// Phase 2: Social Media Handler
use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::utils::get_user_id;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/social")
            .route("/accounts", web::get().to(list_accounts))
            .route("/accounts", web::post().to(connect_account))
            .route("/accounts/{id}", web::get().to(get_account))
            .route("/accounts/{id}", web::delete().to(disconnect_account))
            .route("/accounts/{id}/toggle-ai", web::post().to(toggle_ai_content))
            .route("/content-calendar", web::get().to(get_content_calendar))
            .route("/content-calendar", web::post().to(create_content))
            .route("/content-calendar/{id}", web::get().to(get_content_item))
            .route("/content-calendar/{id}/schedule", web::post().to(schedule_content))
            .route("/content-calendar/{id}/publish", web::post().to(publish_content))
            .route("/content-calendar/{id}", web::delete().to(delete_content))
            .route("/generate-content", web::post().to(generate_ai_content))
            .route("/dashboard", web::get().to(get_social_dashboard))
    );
}

// Accounts
async fn list_accounts(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let accounts = sqlx::query_as::<_, SocialMediaAccount>(
        "SELECT * FROM social_media_accounts WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let responses: Vec<SocialAccountResponse> = accounts.into_iter()
        .map(|a| SocialAccountResponse {
            id: a.id,
            platform: a.platform,
            account_handle: a.account_handle,
            account_url: a.account_url,
            status: a.status,
            follower_count: a.follower_count,
            engagement_rate: a.engagement_rate,
            ai_content_enabled: a.ai_content_enabled,
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn connect_account(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<ConnectSocialAccountRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // In production, exchange auth_code for tokens with the platform's API
    let account = sqlx::query_as::<_, SocialMediaAccount>(
        r#"INSERT INTO social_media_accounts (business_id, user_id, platform, account_handle, status, ai_content_enabled, posting_schedule)
           VALUES ($1, $2, $3, $4, 'connected', false, '{}')
           RETURNING *"#
    )
    .bind(business_id)
    .bind(user_id)
    .bind(&body.platform)
    .bind(format!("@{}_handle", body.platform)) // Placeholder
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(account)))
}

async fn get_account(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let account_id = path.into_inner();
    
    let account = sqlx::query_as::<_, SocialMediaAccount>(
        "SELECT * FROM social_media_accounts WHERE id = $1 AND business_id = $2"
    )
    .bind(account_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match account {
        Some(a) => Ok(HttpResponse::Ok().json(ApiResponse::success(a))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Account not found"))),
    }
}

async fn disconnect_account(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let account_id = path.into_inner();
    
    sqlx::query("UPDATE social_media_accounts SET status = 'disconnected' WHERE id = $1 AND business_id = $2")
        .bind(account_id)
        .bind(business_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

async fn toggle_ai_content(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let account_id = path.into_inner();
    
    let account = sqlx::query_as::<_, SocialMediaAccount>(
        r#"UPDATE social_media_accounts 
           SET ai_content_enabled = NOT ai_content_enabled
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(account_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match account {
        Some(a) => Ok(HttpResponse::Ok().json(ApiResponse::success(a))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Account not found"))),
    }
}

// Content Calendar
async fn get_content_calendar(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<serde_json::Value>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let status_filter = query.get("status").and_then(|v| v.as_str());
    
    let items = if let Some(status) = status_filter {
        sqlx::query_as::<_, ContentCalendarItem>(
            "SELECT * FROM content_calendar_items WHERE business_id = $1 AND status = $2 ORDER BY scheduled_at ASC"
        )
        .bind(business_id)
        .bind(status)
        .fetch_all(pool.get_ref())
        .await
    } else {
        sqlx::query_as::<_, ContentCalendarItem>(
            "SELECT * FROM content_calendar_items WHERE business_id = $1 ORDER BY scheduled_at ASC NULLS LAST LIMIT 50"
        )
        .bind(business_id)
        .fetch_all(pool.get_ref())
        .await
    };
    
    let items = items.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let responses: Vec<ContentCalendarResponse> = items.into_iter()
        .map(|i| ContentCalendarResponse {
            id: i.id,
            platform: "Unknown".to_string(), // Would join with accounts table
            content_type: i.content_type,
            status: i.status,
            title: i.title,
            content: i.content,
            scheduled_at: i.scheduled_at,
            metrics: ContentMetrics {
                likes: i.likes,
                comments: i.comments,
                shares: i.shares,
                impressions: i.impressions,
            },
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn create_content(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateContentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Generate content if AI requested
    let (content, ai_generated) = if body.ai_generate {
        let generated = generate_content_with_ai(&body.topic, &body.tone).await?;
        (Some(generated.content), Some(serde_json::json!({
            "hashtags": generated.hashtags,
            "suggested_images": generated.suggested_images,
            "best_posting_time": generated.best_posting_time,
            "predicted_engagement": generated.predicted_engagement,
        })))
    } else {
        (None, None)
    };
    
    let item = sqlx::query_as::<_, ContentCalendarItem>(
        r#"INSERT INTO content_calendar_items (business_id, social_account_id, content_type, status, title, content, ai_generated_content, media_urls)
           VALUES ($1, $2, $3, 'draft', $4, $5, $6, '{}')
           RETURNING *"#
    )
    .bind(business_id)
    .bind(body.social_account_id)
    .bind(&body.content_type)
    .bind(body.topic.clone())
    .bind(content)
    .bind(ai_generated.unwrap_or_default())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(item)))
}

async fn get_content_item(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let item_id = path.into_inner();
    
    let item = sqlx::query_as::<_, ContentCalendarItem>(
        "SELECT * FROM content_calendar_items WHERE id = $1 AND business_id = $2"
    )
    .bind(item_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match item {
        Some(i) => Ok(HttpResponse::Ok().json(ApiResponse::success(i))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Content not found"))),
    }
}

async fn schedule_content(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<ScheduleContentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let item_id = path.into_inner();
    
    let item = sqlx::query_as::<_, ContentCalendarItem>(
        r#"UPDATE content_calendar_items 
           SET status = 'scheduled', scheduled_at = $3, timezone = $4
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(item_id)
    .bind(business_id)
    .bind(body.scheduled_at)
    .bind(&body.timezone)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match item {
        Some(i) => Ok(HttpResponse::Ok().json(ApiResponse::success(i))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Content not found"))),
    }
}

async fn publish_content(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let item_id = path.into_inner();
    
    let item = sqlx::query_as::<_, ContentCalendarItem>(
        r#"UPDATE content_calendar_items 
           SET status = 'published', published_at = NOW()
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(item_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match item {
        Some(i) => Ok(HttpResponse::Ok().json(ApiResponse::success(i))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Content not found"))),
    }
}

async fn delete_content(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let item_id = path.into_inner();
    
    sqlx::query("DELETE FROM content_calendar_items WHERE id = $1 AND business_id = $2")
        .bind(item_id)
        .bind(business_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

// AI Content Generation
async fn generate_ai_content(
    _pool: web::Data<PgPool>,
    _req: HttpRequest,
    body: web::Json<CreateContentRequest>,
) -> Result<HttpResponse> {
    let generated = generate_content_with_ai(&body.topic, &body.tone).await?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(generated)))
}

// Dashboard
async fn get_social_dashboard(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let total_accounts = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM social_media_accounts WHERE business_id = $1 AND status = 'connected'"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let total_followers = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(follower_count), 0) FROM social_media_accounts WHERE business_id = $1"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let scheduled_posts = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM content_calendar_items WHERE business_id = $1 AND status = 'scheduled'"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let published_posts = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM content_calendar_items WHERE business_id = $1 AND status = 'published'"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let dashboard = serde_json::json!({
        "total_accounts": total_accounts,
        "total_followers": total_followers,
        "scheduled_posts": scheduled_posts,
        "published_posts": published_posts,
        "engagement_rate": 0.0, // Would calculate
    });
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(dashboard)))
}

// Helper functions
async fn generate_content_with_ai(topic: &Option<String>, tone: &Option<String>) -> Result<AiGeneratedPost, actix_web::Error> {
    // In production, this would call an AI service
    let topic_str = topic.as_deref().unwrap_or("your business");
    let _tone_str = tone.as_deref().unwrap_or("professional");
    
    Ok(AiGeneratedPost {
        content: format!(
            "🚀 Exciting update about {}! We're making great progress and can't wait to share more. Stay tuned for what's coming next! 🎯\n\n#startup #growth #innovation",
            topic_str
        ),
        hashtags: vec![
            "startup".to_string(),
            "growth".to_string(),
            "innovation".to_string(),
            "entrepreneur".to_string(),
        ],
        suggested_images: vec![],
        best_posting_time: "Tuesday 9:00 AM".to_string(),
        predicted_engagement: "High".to_string(),
    })
}

async fn get_default_business_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, actix_web::Error> {
    let business_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM businesses WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorBadRequest("No business found"))?;
    
    Ok(business_id)
}
