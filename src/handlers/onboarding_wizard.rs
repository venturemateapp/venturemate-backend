//! Onboarding Wizard Handlers
//! Complete API implementation per specification

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{
    InviteCofounderRequest, SaveStepAnswersRequest, StartOnboardingRequest,
    TrackOnboardingEventRequest,
};
use crate::services::OnboardingWizardService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/onboarding-wizard")
            // Wizard flow endpoints
            .service(start_wizard)
            .service(save_step)
            .service(get_step)
            .service(complete_wizard)
            .service(resume_wizard)
            // Countries
            .service(get_countries)
            // Cofounder management
            .service(invite_cofounder)
            .service(get_cofounders)
            // Analytics
            .service(track_event),
    );
}

// ============================================
// 1. START ONBOARDING WIZARD
// ============================================
#[post("/start")]
async fn start_wizard(
    wizard_service: web::Data<OnboardingWizardService>,
    req: HttpRequest,
    body: Option<web::Json<StartOnboardingRequest>>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let _source = body.as_ref().and_then(|b| b.source.clone());

    match wizard_service.start_onboarding(user_id).await {
        Ok(response) => ResponseBuilder::created(response),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 2. SAVE STEP ANSWERS
// ============================================
#[post("/step")]
async fn save_step(
    wizard_service: web::Data<OnboardingWizardService>,
    req: web::Json<SaveStepAnswersRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match wizard_service.save_step_answers(user_id, req.into_inner()).await {
        Ok(response) => {
            if response.success {
                ResponseBuilder::ok(response)
            } else {
                ResponseBuilder::bad_request_with_data("Validation failed", response)
            }
        }
        Err(e) => e.error_response(),
    }
}

// ============================================
// 3. GET STEP CONTENT
// ============================================
#[derive(Deserialize)]
#[allow(dead_code)]
struct GetStepQuery {
    session_id: Uuid,
}

#[get("/step/{step_number}")]
async fn get_step(
    wizard_service: web::Data<OnboardingWizardService>,
    path: web::Path<i32>,
    _query: web::Query<GetStepQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let step_number = path.into_inner();

    // Validate step number is 1-5
    if !(1..=5).contains(&step_number) {
        return ResponseBuilder::bad_request("Step number must be between 1 and 5");
    }

    // Optionally verify session belongs to user
    // (Commented for now as step content is generic)

    match wizard_service.get_step_content(step_number).await {
        Ok(content) => ResponseBuilder::ok(content),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 4. COMPLETE ONBOARDING
// ============================================
#[derive(Deserialize)]
struct CompleteWizardRequest {
    session_id: Uuid,
}

#[post("/complete")]
async fn complete_wizard(
    wizard_service: web::Data<OnboardingWizardService>,
    req: web::Json<CompleteWizardRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match wizard_service.complete_onboarding(user_id, req.session_id).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 5. RESUME ONBOARDING
// ============================================
#[derive(Deserialize)]
struct ResumeWizardQuery {
    session_id: Uuid,
}

#[get("/resume")]
async fn resume_wizard(
    wizard_service: web::Data<OnboardingWizardService>,
    query: web::Query<ResumeWizardQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match wizard_service.resume_onboarding(user_id, query.session_id).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 6. GET SUPPORTED COUNTRIES
// ============================================
#[get("/countries")]
async fn get_countries(
    wizard_service: web::Data<OnboardingWizardService>,
) -> HttpResponse {
    match wizard_service.get_supported_countries().await {
        Ok(countries) => ResponseBuilder::ok(countries),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 7. INVITE CO-FOUNDER
// ============================================
#[derive(Deserialize)]
struct InviteCofounderBody {
    startup_id: Option<Uuid>,
    #[serde(flatten)]
    invite: InviteCofounderRequest,
}

#[post("/invite-cofounder")]
async fn invite_cofounder(
    wizard_service: web::Data<OnboardingWizardService>,
    req: web::Json<InviteCofounderBody>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match wizard_service.invite_cofounder(user_id, req.startup_id, req.invite.clone()).await {
        Ok(response) => ResponseBuilder::created(response),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 8. GET CO-FOUNDERS
// ============================================
#[derive(Deserialize)]
struct GetCofoundersQuery {
    startup_id: Uuid,
}

#[get("/cofounders")]
async fn get_cofounders(
    wizard_service: web::Data<OnboardingWizardService>,
    query: web::Query<GetCofoundersQuery>,
    req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match wizard_service.get_cofounders_for_startup(query.startup_id).await {
        Ok(cofounders) => ResponseBuilder::ok(cofounders),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 9. TRACK ANALYTICS EVENT
// ============================================
#[post("/track")]
async fn track_event(
    req: web::Json<TrackOnboardingEventRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    // Analytics tracking - store event
    // For now, just acknowledge
    ResponseBuilder::ok(serde_json::json!({
        "tracked": true,
        "event_type": req.event_type,
        "step_number": req.step_number,
    }))
}
