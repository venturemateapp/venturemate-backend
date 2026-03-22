//! Recommendations API Handlers

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::recommendations::{
    ActOnRecommendationRequest, DismissRecommendationRequest, ListRecommendationsRequest,
};
use crate::models::ApiResponse;
use crate::services::RecommendationsService;
use crate::utils::get_user_id;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/recommendations")
            .service(list_recommendations)
            .service(refresh_recommendations)
            .service(dismiss_recommendation)
            .service(act_on_recommendation)
    );
}

/// GET /api/v1/recommendations/{business_id}
#[get("/{business_id}")]
async fn list_recommendations(
    service: web::Data<RecommendationsService>,
    business_id: web::Path<Uuid>,
    query: web::Query<ListRecommendationsRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.list_recommendations(business_id.into_inner(), query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/recommendations/{business_id}/refresh
#[post("/{business_id}/refresh")]
async fn refresh_recommendations(
    service: web::Data<RecommendationsService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.refresh_recommendations(business_id.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/recommendations/{recommendation_id}/dismiss
#[post("/{recommendation_id}/dismiss")]
async fn dismiss_recommendation(
    service: web::Data<RecommendationsService>,
    recommendation_id: web::Path<Uuid>,
    req: web::Json<DismissRecommendationRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.dismiss_recommendation(recommendation_id.into_inner(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/recommendations/{recommendation_id}/act
#[post("/{recommendation_id}/act")]
async fn act_on_recommendation(
    service: web::Data<RecommendationsService>,
    recommendation_id: web::Path<Uuid>,
    req: web::Json<ActOnRecommendationRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.act_on_recommendation(recommendation_id.into_inner(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}
