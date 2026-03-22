use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::{
    CreateCofounderProfileRequest, UpdateCofounderProfileRequest,
    SendMatchRequest, RespondToMatchRequest, SearchCofounderRequest,
};
use crate::services::CofounderService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/cofounders")
            .service(create_profile)
            .service(get_my_profile)
            .service(update_profile)
            .service(search_profiles)
            .service(get_matches)
            .service(send_match_request)
            .service(respond_to_match)
            .service(get_match_requests)
    );
}

// ============================================
// PROFILE MANAGEMENT
// ============================================

#[post("/profile")]
async fn create_profile(
    req: web::Json<CreateCofounderProfileRequest>,
    service: web::Data<CofounderService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.create_profile(user_id, req.into_inner()).await {
        Ok(profile) => ResponseBuilder::created(profile),
        Err(e) => e.error_response(),
    }
}

#[get("/profile")]
async fn get_my_profile(
    service: web::Data<CofounderService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.get_profile(user_id).await {
        Ok(profile) => ResponseBuilder::ok(profile),
        Err(e) => e.error_response(),
    }
}

#[put("/profile")]
async fn update_profile(
    req: web::Json<UpdateCofounderProfileRequest>,
    service: web::Data<CofounderService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.update_profile(user_id, req.into_inner()).await {
        Ok(profile) => ResponseBuilder::ok(profile),
        Err(e) => e.error_response(),
    }
}

// ============================================
// MATCHING
// ============================================

#[get("/search")]
async fn search_profiles(
    query: web::Query<SearchCofounderRequest>,
    service: web::Data<CofounderService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.search_profiles(user_id, query.into_inner()).await {
        Ok(profiles) => ResponseBuilder::ok(profiles),
        Err(e) => e.error_response(),
    }
}

#[get("/matches")]
async fn get_matches(
    service: web::Data<CofounderService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.get_matches(user_id).await {
        Ok(matches) => ResponseBuilder::ok(matches),
        Err(e) => e.error_response(),
    }
}

#[post("/matches")]
async fn send_match_request(
    req: web::Json<SendMatchRequest>,
    service: web::Data<CofounderService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let target_user_id = match Uuid::parse_str(&req.user_id) {
        Ok(id) => id,
        Err(_) => return ResponseBuilder::validation_error("Invalid user ID"),
    };

    match service.send_match_request(user_id, target_user_id, req.message.clone()).await {
        Ok(match_result) => ResponseBuilder::ok(match_result),
        Err(e) => e.error_response(),
    }
}

#[post("/matches/{id}/respond")]
async fn respond_to_match(
    path: web::Path<Uuid>,
    req: web::Json<RespondToMatchRequest>,
    service: web::Data<CofounderService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.respond_to_match(user_id, path.into_inner(), req.accept, req.message.clone()).await {
        Ok(_) => ResponseBuilder::ok(serde_json::json!({"message": "Response recorded"})),
        Err(e) => e.error_response(),
    }
}

#[get("/matches/requests")]
async fn get_match_requests(
    service: web::Data<CofounderService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match service.get_pending_requests(user_id).await {
        Ok(requests) => ResponseBuilder::ok(requests),
        Err(e) => e.error_response(),
    }
}
