//! Founder Dashboard Handler
//! Single endpoint to load all dashboard data

use actix_web::{get, web, HttpResponse};
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::services::DashboardService;
use crate::utils::get_user_id;

/// GET /api/v1/dashboard/{startup_id}
/// Get complete dashboard data for a startup
#[get("/dashboard/{startup_id}")]
pub async fn get_dashboard(
    pool: web::Data<sqlx::PgPool>,
    startup_id: web::Path<Uuid>,
    req: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_id = get_user_id(&req)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let startup_id = startup_id.into_inner();

    let service = DashboardService::new(pool.get_ref().clone());
    let dashboard = service
        .get_dashboard(startup_id, user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(dashboard)))
}

/// GET /api/v1/dashboard/{startup_id}/quick-actions
/// Get next actions only (lightweight endpoint)
#[get("/dashboard/{startup_id}/quick-actions")]
pub async fn get_quick_actions(
    pool: web::Data<sqlx::PgPool>,
    startup_id: web::Path<Uuid>,
    req: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_id = get_user_id(&req)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let startup_id = startup_id.into_inner();

    let service = DashboardService::new(pool.get_ref().clone());
    let dashboard = service
        .get_dashboard(startup_id, user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(dashboard.next_actions)))
}

/// GET /api/v1/dashboard/{startup_id}/activity
/// Get activity feed only
#[get("/dashboard/{startup_id}/activity")]
pub async fn get_activity_feed(
    pool: web::Data<sqlx::PgPool>,
    startup_id: web::Path<Uuid>,
    req: actix_web::HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let user_id = get_user_id(&req)
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let startup_id = startup_id.into_inner();

    let service = DashboardService::new(pool.get_ref().clone());
    let dashboard = service
        .get_dashboard(startup_id, user_id)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(dashboard.activity_feed)))
}
