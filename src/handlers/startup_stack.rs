//! Startup Stack Generator API Handlers
//! Complete CRUD implementation per specification

use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::models::{
    AiBlueprint, ConnectServiceRequest, UpdateApprovalRequest, UpdateMilestoneRequest,
    UpdateStartupRequest,
};
use crate::services::StartupStackService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/startups")
            // Startup CRUD
            .service(create_startup)
            .service(list_startups)
            .service(get_startup)
            .service(update_startup)
            .service(get_startup_progress)
            // Milestones
            .service(get_milestones)
            .service(update_milestone)
            .service(complete_milestone)
            // Approvals
            .service(get_approvals)
            .service(update_approval)
            // Services
            .service(get_services)
            .service(connect_service)
            // Documents
            .service(get_documents)
            // Dashboard data
            .service(get_upcoming_deadlines),
    );
}

// ============================================
// 1. STARTUP ENDPOINTS
// ============================================

#[post("")]
async fn create_startup(
    req: web::Json<AiBlueprint>,
    startup_service: web::Data<StartupStackService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match startup_service.create_from_blueprint(user_id, req.into_inner()).await {
        Ok(response) => ResponseBuilder::created(response),
        Err(e) => e.error_response(),
    }
}

#[get("")]
async fn list_startups(
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match startup_service.list_startups(user_id).await {
        Ok(startups) => ResponseBuilder::ok(startups),
        Err(e) => e.error_response(),
    }
}

#[get("/{startup_id}")]
async fn get_startup(
    path: web::Path<Uuid>,
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match startup_service.get_startup(path.into_inner(), user_id).await {
        Ok(startup) => ResponseBuilder::ok(startup),
        Err(e) => e.error_response(),
    }
}

#[put("/{startup_id}")]
async fn update_startup(
    path: web::Path<Uuid>,
    req: web::Json<UpdateStartupRequest>,
    startup_service: web::Data<StartupStackService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match startup_service.update_startup(path.into_inner(), user_id, req.into_inner()).await {
        Ok(startup) => ResponseBuilder::ok(startup),
        Err(e) => e.error_response(),
    }
}

#[get("/{startup_id}/progress")]
async fn get_startup_progress(
    path: web::Path<Uuid>,
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let startup_id = path.into_inner();

    // Verify ownership
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    if let Err(e) = startup_service.get_startup(startup_id, user_id).await {
        return e.error_response();
    }

    match startup_service.get_progress(startup_id).await {
        Ok(progress) => ResponseBuilder::ok(progress),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 2. MILESTONE ENDPOINTS
// ============================================

#[derive(Deserialize)]
struct MilestoneFilter {
    status: Option<String>,
}

#[get("/{startup_id}/milestones")]
async fn get_milestones(
    path: web::Path<Uuid>,
    query: web::Query<MilestoneFilter>,
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let startup_id = path.into_inner();

    // Verify ownership
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    if let Err(e) = startup_service.get_startup(startup_id, user_id).await {
        return e.error_response();
    }

    match startup_service.get_milestones(startup_id, query.status.clone()).await {
        Ok(milestones) => ResponseBuilder::ok(milestones),
        Err(e) => e.error_response(),
    }
}

#[put("/{startup_id}/milestones/{milestone_id}")]
async fn update_milestone(
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateMilestoneRequest>,
    startup_service: web::Data<StartupStackService>,
    _http_req: HttpRequest,
) -> HttpResponse {
    let (startup_id, milestone_id) = path.into_inner();

    match startup_service.update_milestone(milestone_id, startup_id, req.into_inner()).await {
        Ok(milestone) => ResponseBuilder::ok(milestone),
        Err(e) => e.error_response(),
    }
}

#[post("/{startup_id}/milestones/{milestone_id}/complete")]
async fn complete_milestone(
    path: web::Path<(Uuid, Uuid)>,
    startup_service: web::Data<StartupStackService>,
    _http_req: HttpRequest,
) -> HttpResponse {
    let (startup_id, milestone_id) = path.into_inner();

    let req = UpdateMilestoneRequest {
        status: Some("completed".to_string()),
        started_at: None,
        completed_at: Some(chrono::Utc::now()),
        notes: None,
    };

    match startup_service.update_milestone(milestone_id, startup_id, req).await {
        Ok(milestone) => ResponseBuilder::ok(milestone),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 3. APPROVAL ENDPOINTS
// ============================================

#[get("/{startup_id}/approvals")]
async fn get_approvals(
    path: web::Path<Uuid>,
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let startup_id = path.into_inner();

    // Verify ownership
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    if let Err(e) = startup_service.get_startup(startup_id, user_id).await {
        return e.error_response();
    }

    match startup_service.get_approvals(startup_id).await {
        Ok(approvals) => ResponseBuilder::ok(approvals),
        Err(e) => e.error_response(),
    }
}

#[put("/{startup_id}/approvals/{approval_id}")]
async fn update_approval(
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateApprovalRequest>,
    startup_service: web::Data<StartupStackService>,
    _http_req: HttpRequest,
) -> HttpResponse {
    let (startup_id, approval_id) = path.into_inner();

    match startup_service.update_approval(approval_id, startup_id, req.into_inner()).await {
        Ok(approval) => ResponseBuilder::ok(approval),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 4. SERVICE ENDPOINTS
// ============================================

#[derive(Deserialize)]
struct ServiceFilter {
    category: Option<String>,
}

#[get("/{startup_id}/services")]
async fn get_services(
    path: web::Path<Uuid>,
    query: web::Query<ServiceFilter>,
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let startup_id = path.into_inner();

    // Verify ownership
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    if let Err(e) = startup_service.get_startup(startup_id, user_id).await {
        return e.error_response();
    }

    match startup_service.get_services(startup_id, query.category.clone()).await {
        Ok(services) => ResponseBuilder::ok(services),
        Err(e) => e.error_response(),
    }
}

#[post("/{startup_id}/services/{service_id}/connect")]
async fn connect_service(
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<ConnectServiceRequest>,
    startup_service: web::Data<StartupStackService>,
    _http_req: HttpRequest,
) -> HttpResponse {
    let (startup_id, service_id) = path.into_inner();

    match startup_service.connect_service(service_id, startup_id, req.into_inner()).await {
        Ok(service) => ResponseBuilder::ok(service),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 5. DOCUMENT ENDPOINTS
// ============================================

#[get("/{startup_id}/documents")]
async fn get_documents(
    path: web::Path<Uuid>,
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let startup_id = path.into_inner();

    // Verify ownership
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    if let Err(e) = startup_service.get_startup(startup_id, user_id).await {
        return e.error_response();
    }

    match startup_service.get_documents(startup_id).await {
        Ok(documents) => ResponseBuilder::ok(documents),
        Err(e) => e.error_response(),
    }
}

// ============================================
// 6. DASHBOARD DATA
// ============================================

#[get("/upcoming-deadlines")]
async fn get_upcoming_deadlines(
    startup_service: web::Data<StartupStackService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match startup_service.get_upcoming_deadlines(user_id).await {
        Ok(deadlines) => ResponseBuilder::ok(deadlines),
        Err(e) => e.error_response(),
    }
}
