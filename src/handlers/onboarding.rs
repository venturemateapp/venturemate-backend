use actix_web::{get, post, web, HttpRequest, HttpResponse};

use crate::models::{
    BusinessDetailsRequest, FounderProfileRequest, IdeaIntakeRequest, ReviewOnboardingRequest,
};
use crate::services::onboarding_service::OnboardingService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onboarding")
            .service(start_session)
            .service(submit_idea_intake)
            .service(submit_founder_profile)
            .service(submit_business_details)
            .service(complete_onboarding)
            .service(get_status),
    );
}

#[post("/start")]
async fn start_session(
    onboarding_service: web::Data<OnboardingService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match onboarding_service.start_session(user_id).await {
        Ok(session) => ResponseBuilder::created(session),
        Err(e) => e.error_response(),
    }
}

#[post("/idea-intake")]
async fn submit_idea_intake(
    req: web::Json<IdeaIntakeRequest>,
    onboarding_service: web::Data<OnboardingService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match onboarding_service.submit_idea_intake(user_id, req.into_inner()).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

#[post("/founder-profile")]
async fn submit_founder_profile(
    req: web::Json<FounderProfileRequest>,
    onboarding_service: web::Data<OnboardingService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match onboarding_service.submit_founder_profile(user_id, req.into_inner()).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

#[post("/business-details")]
async fn submit_business_details(
    req: web::Json<BusinessDetailsRequest>,
    onboarding_service: web::Data<OnboardingService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match onboarding_service.submit_business_details(user_id, req.into_inner()).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

#[post("/review")]
async fn complete_onboarding(
    req: web::Json<ReviewOnboardingRequest>,
    onboarding_service: web::Data<OnboardingService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match onboarding_service.complete_onboarding(user_id, req.into_inner()).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

#[get("/status")]
async fn get_status(
    onboarding_service: web::Data<OnboardingService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match onboarding_service.get_status(user_id).await {
        Ok(status) => ResponseBuilder::ok(status),
        Err(e) => e.error_response(),
    }
}
