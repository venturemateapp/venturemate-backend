//! Health Score API Handlers

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::health_score::{
    AnalyzeWebsiteRequest, GetHealthScoreHistoryRequest, RefreshHealthScoreRequest,
};
use crate::models::ApiResponse;
use crate::services::HealthScoreService;
use crate::utils::get_user_id;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/health-score")
            .service(get_health_score)
            .service(refresh_health_score)
            .service(get_health_score_history)
            .service(analyze_website)
    );
}

/// GET /api/v1/health-score/{business_id}
#[get("/{business_id}")]
async fn get_health_score(
    service: web::Data<HealthScoreService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.get_health_score(business_id.into_inner()).await {
        Ok(Some(score)) => HttpResponse::Ok().json(ApiResponse::success(score)),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Health score not found")),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/health-score/{business_id}/refresh
#[post("/{business_id}/refresh")]
async fn refresh_health_score(
    service: web::Data<HealthScoreService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.calculate_health_score(business_id.into_inner()).await {
        Ok(score) => HttpResponse::Ok().json(ApiResponse::success(score)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/health-score/{business_id}/history
#[get("/{business_id}/history")]
async fn get_health_score_history(
    service: web::Data<HealthScoreService>,
    business_id: web::Path<Uuid>,
    query: web::Query<GetHealthScoreHistoryRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.get_score_history(business_id.into_inner(), query.days).await {
        Ok(history) => HttpResponse::Ok().json(ApiResponse::success(history)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/health-score/{business_id}/analyze-website
#[post("/{business_id}/analyze-website")]
async fn analyze_website(
    service: web::Data<HealthScoreService>,
    business_id: web::Path<Uuid>,
    req: web::Json<AnalyzeWebsiteRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.analyze_website(business_id.into_inner(), req.website_url.clone()).await {
        Ok(result) => HttpResponse::Ok().json(ApiResponse::success(result)),
        Err(e) => e.into_response(),
    }
}
