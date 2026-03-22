// Phase 3: Investor Matchmaking Handler
use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::investor::*;
use crate::models::documents::{DataRoom, DataRoomResponse, CreateDataRoomRequest, DataRoomDocument, DataRoomAccess, AddDocumentToDataRoomRequest};
use crate::models::business::Business;
use crate::models::{ApiResponse, PaginatedResponse};
use crate::utils::get_user_id;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/investors")
            .route("/profile", web::get().to(get_investor_profile))
            .route("/profile", web::post().to(create_investor_profile))
            .route("/profile", web::put().to(update_investor_profile))
            .route("/search", web::post().to(search_investors))
            .route("/matches", web::get().to(get_matches))
            .route("/matches/{id}/pitch", web::post().to(submit_pitch))
            .route("/matches/{id}/status", web::patch().to(update_match_status))
            .route("/stats", web::get().to(get_matchmaking_stats))
            .route("/data-rooms", web::get().to(list_data_rooms))
            .route("/data-rooms", web::post().to(create_data_room))
            .route("/data-rooms/{id}", web::get().to(get_data_room))
            .route("/data-rooms/{id}/documents", web::post().to(add_document_to_data_room))
            .route("/data-rooms/{id}/grant-access", web::post().to(grant_data_room_access))
    );
}

// Investor Profile
async fn get_investor_profile(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let profile = sqlx::query_as::<_, InvestorProfile>(
        "SELECT * FROM investor_profiles WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match profile {
        Some(p) => Ok(HttpResponse::Ok().json(ApiResponse::success(profile_to_response(p)))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Profile not found"))),
    }
}

async fn create_investor_profile(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateInvestorProfileRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let existing = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM investor_profiles WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if existing.is_some() {
        return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error("ALREADY_EXISTS", "Investor profile already exists")));
    }
    
    let preferred_countries = serde_json::to_value(&body.preferred_countries).unwrap_or_default();
    let investment_stage = serde_json::to_value(&body.investment_stage).unwrap_or_default();
    let preferred_industries = serde_json::to_value(&body.preferred_industries).unwrap_or_default();
    let past_investments = serde_json::json!([]);
    
    let profile = sqlx::query_as::<_, InvestorProfile>(
        r#"INSERT INTO investor_profiles (user_id, investor_type, firm_name, bio, website_url, linkedin_url, location, preferred_countries, investment_stage, check_size_min, check_size_max, currency, preferred_industries, past_investments, thesis, value_add, is_verified, is_active)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, COALESCE($12, 'USD'), $13, $14, $15, $16, false, true)
           RETURNING *"#
    )
    .bind(user_id)
    .bind(&body.investor_type)
    .bind(&body.firm_name)
    .bind(&body.bio)
    .bind(&body.website_url)
    .bind(&body.linkedin_url)
    .bind(&body.location)
    .bind(preferred_countries)
    .bind(investment_stage)
    .bind(body.check_size_min)
    .bind(body.check_size_max)
    .bind(&body.currency)
    .bind(preferred_industries)
    .bind(past_investments)
    .bind(&body.thesis)
    .bind(&body.value_add)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(profile_to_response(profile))))
}

async fn update_investor_profile(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateInvestorProfileRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let preferred_countries = serde_json::to_value(&body.preferred_countries).unwrap_or_default();
    let investment_stage = serde_json::to_value(&body.investment_stage).unwrap_or_default();
    let preferred_industries = serde_json::to_value(&body.preferred_industries).unwrap_or_default();
    
    let profile = sqlx::query_as::<_, InvestorProfile>(
        r#"UPDATE investor_profiles 
           SET investor_type = $2, firm_name = $3, bio = $4, website_url = $5, linkedin_url = $6, location = $7,
               preferred_countries = $8, investment_stage = $9, check_size_min = $10, check_size_max = $11,
               preferred_industries = $12, thesis = $13, value_add = $14
           WHERE user_id = $1
           RETURNING *"#
    )
    .bind(user_id)
    .bind(&body.investor_type)
    .bind(&body.firm_name)
    .bind(&body.bio)
    .bind(&body.website_url)
    .bind(&body.linkedin_url)
    .bind(&body.location)
    .bind(preferred_countries)
    .bind(investment_stage)
    .bind(body.check_size_min)
    .bind(body.check_size_max)
    .bind(preferred_industries)
    .bind(&body.thesis)
    .bind(&body.value_add)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match profile {
        Some(p) => Ok(HttpResponse::Ok().json(ApiResponse::success(profile_to_response(p)))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Profile not found"))),
    }
}

// Search & Matching
async fn search_investors(
    pool: web::Data<PgPool>,
    req: web::Json<SearchInvestorsRequest>,
) -> Result<HttpResponse> {
    let mut query = String::from("SELECT * FROM investor_profiles WHERE is_active = true");
    
    if let Some(ref stages) = req.stages {
        if !stages.is_empty() {
            query.push_str(" AND investment_stage ?| array[");
            for (i, _) in stages.iter().enumerate() {
                if i > 0 { query.push(','); }
                query.push_str(&format!("${}", i + 1));
            }
            query.push(']');
        }
    }
    
    query.push_str(" ORDER BY rating DESC NULLS LAST LIMIT 50");
    
    let profiles = sqlx::query_as::<_, InvestorProfile>(&query)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let responses: Vec<InvestorProfileResponse> = profiles.into_iter()
        .map(profile_to_response)
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn get_matches(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Generate matches if none exist
    let existing_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM investor_matches WHERE business_id = $1"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    if existing_count == 0 {
        generate_matches(&pool, business_id).await?;
    }
    
    // Fetch matches and profiles separately, then combine
    let matches = sqlx::query_as::<_, InvestorMatch>(
        "SELECT * FROM investor_matches WHERE business_id = $1 ORDER BY match_score DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Get all investor IDs from matches
    let investor_ids: Vec<uuid::Uuid> = matches.iter().map(|m| m.investor_id).collect();
    
    // Fetch profiles for these investors
    let profiles = if !investor_ids.is_empty() {
        sqlx::query_as::<_, InvestorProfile>(
            "SELECT * FROM investor_profiles WHERE id = ANY($1)"
        )
        .bind(&investor_ids)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else {
        vec![]
    };
    
    // Build a map of profiles for lookup
    let profile_map: std::collections::HashMap<uuid::Uuid, InvestorProfile> = profiles
        .into_iter()
        .map(|p| (p.id, p))
        .collect();
    
    let responses: Vec<InvestorMatchResponse> = matches
        .into_iter()
        .filter_map(|m| {
            profile_map.get(&m.investor_id).map(|p| InvestorMatchResponse {
                id: m.id,
                investor: profile_to_response(p.clone()),
                match_score: m.match_score,
                match_reasons: serde_json::from_value(m.match_reasons.clone()).unwrap_or_default(),
                status: m.status,
                created_at: m.created_at,
            })
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn submit_pitch(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<SubmitPitchRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let match_id = path.into_inner();
    
    let investor_match = sqlx::query_as::<_, InvestorMatch>(
        r#"UPDATE investor_matches 
           SET status = 'pitched', business_pitch = $3
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(match_id)
    .bind(business_id)
    .bind(&body.pitch_message)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match investor_match {
        Some(m) => Ok(HttpResponse::Ok().json(ApiResponse::success(m))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Match not found"))),
    }
}

async fn update_match_status(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let match_id = path.into_inner();
    let status = body["status"].as_str().unwrap_or("");
    
    let investor_match = sqlx::query_as::<_, InvestorMatch>(
        "UPDATE investor_matches SET status = $3 WHERE id = $1 AND business_id = $2 RETURNING *"
    )
    .bind(match_id)
    .bind(business_id)
    .bind(status)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match investor_match {
        Some(m) => Ok(HttpResponse::Ok().json(ApiResponse::success(m))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Match not found"))),
    }
}

async fn get_matchmaking_stats(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let total_investors = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM investor_profiles WHERE is_active = true"
    )
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let matched_investors = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM investor_matches WHERE business_id = $1"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let pending_pitches = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM investor_matches WHERE business_id = $1 AND status = 'pitched'"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let interested_investors = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM investor_matches WHERE business_id = $1 AND status IN ('interested', 'connected', 'pitched')"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let meetings_scheduled = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM investor_matches WHERE business_id = $1 AND meeting_scheduled_at IS NOT NULL"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let match_rate = if total_investors > 0 {
        (matched_investors as f64 / total_investors as f64) * 100.0
    } else {
        0.0
    };
    
    let stats = MatchmakingStats {
        total_investors,
        matched_investors,
        pending_pitches,
        interested_investors,
        meetings_scheduled,
        match_rate,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(stats)))
}

// Data Rooms
async fn list_data_rooms(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let data_rooms = sqlx::query_as::<_, DataRoom>(
        "SELECT * FROM data_rooms WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Get document counts for each data room
    let data_room_ids: Vec<uuid::Uuid> = data_rooms.iter().map(|dr| dr.id).collect();
    
    #[derive(sqlx::FromRow)]
    struct DocCountRow {
        data_room_id: Option<uuid::Uuid>,
        count: Option<i64>,
    }
    
    let doc_counts: Vec<DocCountRow> = if !data_room_ids.is_empty() {
        sqlx::query_as::<_, DocCountRow>(
            r#"SELECT data_room_id, COUNT(*) as count 
               FROM data_room_documents 
               WHERE data_room_id = ANY($1)
               GROUP BY data_room_id"#
        )
        .bind(&data_room_ids)
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    } else {
        vec![]
    };
    
    let count_map: std::collections::HashMap<uuid::Uuid, i64> = doc_counts
        .into_iter()
        .filter_map(|r| r.data_room_id.map(|id| (id, r.count.unwrap_or(0))))
        .collect();
    
    let responses: Vec<DataRoomResponse> = data_rooms.into_iter()
        .map(|dr| DataRoomResponse {
            id: dr.id,
            name: dr.name,
            description: dr.description,
            document_count: *count_map.get(&dr.id).unwrap_or(&0),
            view_count: dr.view_count,
            is_public: dr.is_public,
            expires_at: dr.expires_at,
            access_url: format!("/data-rooms/{}", dr.id),
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn create_data_room(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateDataRoomRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let data_room = sqlx::query_as::<_, DataRoom>(
        r#"INSERT INTO data_rooms (business_id, name, description, is_public, expires_at)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING *"#
    )
    .bind(business_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(body.is_public)
    .bind(body.expires_at)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(data_room)))
}

async fn get_data_room(
    pool: web::Data<PgPool>,
    _req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let data_room_id = path.into_inner();
    
    let data_room = sqlx::query_as::<_, DataRoom>(
        "SELECT * FROM data_rooms WHERE id = $1"
    )
    .bind(data_room_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match data_room {
        Some(dr) => {
            // Increment view count
            sqlx::query("UPDATE data_rooms SET view_count = view_count + 1 WHERE id = $1")
                .bind(data_room_id)
                .execute(pool.get_ref())
                .await
                .ok();
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(dr)))
        },
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Data room not found"))),
    }
}

async fn add_document_to_data_room(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<AddDocumentToDataRoomRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let data_room_id = path.into_inner();
    
    // Verify data room ownership
    let _data_room = sqlx::query_as::<_, DataRoom>(
        "SELECT * FROM data_rooms WHERE id = $1 AND business_id = $2"
    )
    .bind(data_room_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Data room not found"))?;
    
    let doc = sqlx::query_as::<_, DataRoomDocument>(
        r#"INSERT INTO data_room_documents (data_room_id, document_id, folder_path, order_index)
           VALUES ($1, $2, $3, (SELECT COALESCE(MAX(order_index), 0) + 1 FROM data_room_documents WHERE data_room_id = $1))
           RETURNING *"#
    )
    .bind(data_room_id)
    .bind(body.document_id)
    .bind(&body.folder_path)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(doc)))
}

async fn grant_data_room_access(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let data_room_id = path.into_inner();
    
    let investor_id = body["investor_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let email = body["email"].as_str();
    let access_type = body["access_type"].as_str().unwrap_or("view");
    let expires_at = body["expires_at"].as_str().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&chrono::Utc)));
    
    // Verify data room ownership
    let _data_room = sqlx::query_as::<_, DataRoom>(
        "SELECT * FROM data_rooms WHERE id = $1 AND business_id = $2"
    )
    .bind(data_room_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorNotFound("Data room not found"))?;
    
    let access = sqlx::query_as::<_, DataRoomAccess>(
        r#"INSERT INTO data_room_access (data_room_id, investor_id, email, access_type, granted_by, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING *"#
    )
    .bind(data_room_id)
    .bind(investor_id)
    .bind(email)
    .bind(access_type)
    .bind(user_id)
    .bind(expires_at)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(access)))
}

// Helper functions
fn profile_to_response(profile: InvestorProfile) -> InvestorProfileResponse {
    let check_size_range = match (profile.check_size_min, profile.check_size_max) {
        (Some(min), Some(max)) => format!("${} - ${}", min, max),
        (Some(min), None) => format!("${}+", min),
        (None, Some(max)) => format!("Up to ${}", max),
        (None, None) => "Undisclosed".to_string(),
    };
    
    InvestorProfileResponse {
        id: profile.id,
        investor_type: profile.investor_type,
        firm_name: profile.firm_name,
        bio: profile.bio,
        website_url: profile.website_url,
        linkedin_url: profile.linkedin_url,
        location: profile.location,
        investment_stage: serde_json::from_value(profile.investment_stage).unwrap_or_default(),
        check_size_range,
        preferred_industries: serde_json::from_value(profile.preferred_industries).unwrap_or_default(),
        thesis: profile.thesis,
        value_add: profile.value_add,
        is_verified: profile.is_verified,
    }
}

async fn generate_matches(pool: &PgPool, business_id: Uuid) -> Result<(), actix_web::Error> {
    // Get business details
    let _business = sqlx::query_as::<_, Business>(
        "SELECT * FROM businesses WHERE id = $1"
    )
    .bind(business_id)
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    // Find matching investors (simplified algorithm)
    let investors = sqlx::query_as::<_, InvestorProfile>(
        "SELECT * FROM investor_profiles WHERE is_active = true"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    for investor in investors {
        let mut score = 50i32; // Base score
        let mut reasons = vec!["Active investor".to_string()];
        
        // Industry match (simplified)
        if let Ok(industries) = serde_json::from_value::<Vec<String>>(investor.preferred_industries.clone()) {
            if !industries.is_empty() {
                score += 20;
                reasons.push(format!("Invests in {} sectors", industries.len()));
            }
        }
        
        // Cap score at 100
        score = score.min(100);
        
        sqlx::query(
            r#"INSERT INTO investor_matches (business_id, investor_id, match_score, match_reasons, status)
               VALUES ($1, $2, $3, $4, 'pending')
               ON CONFLICT (business_id, investor_id) DO NOTHING"#
        )
        .bind(business_id)
        .bind(investor.id)
        .bind(score)
        .bind(serde_json::to_value(&reasons).unwrap_or_default())
        .execute(pool)
        .await
        .ok();
    }
    
    Ok(())
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
