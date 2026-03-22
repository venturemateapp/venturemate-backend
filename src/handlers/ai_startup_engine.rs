//! AI Startup Engine Handler
//! API endpoints per specification

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::{
    ApiResponse, ProcessStartupRequest, RegenerateFieldRequest,
};
use crate::services::AiStartupEngineService;
use crate::utils::{get_user_id, success_response};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai")
            .service(process_startup)
            .service(get_generation_status)
            .service(regenerate_field)
            .service(list_industries)
            .service(get_regulatory_requirements),
    );
}

/// POST /api/v1/ai/process-startup
/// Process startup through AI engine
#[post("/process-startup")]
async fn process_startup(
    body: web::Json<ProcessStartupRequest>,
    ai_service: web::Data<AiStartupEngineService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match ai_service.process_startup(user_id, body.into_inner()).await {
        Ok(response) => {
            HttpResponse::Accepted().json(ApiResponse::success(response))
        }
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/ai/status/{generation_id}
/// Check processing status
#[get("/status/{generation_id}")]
async fn get_generation_status(
    path: web::Path<Uuid>,
    ai_service: web::Data<AiStartupEngineService>,
    req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    let generation_id = path.into_inner();

    match ai_service.get_generation_status(generation_id).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/ai/regenerate/{field}
/// Regenerate specific field
#[post("/regenerate/{field}")]
async fn regenerate_field(
    path: web::Path<String>,
    body: web::Json<RegenerateFieldRequest>,
    ai_service: web::Data<AiStartupEngineService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    let field = path.into_inner();
    let mut request = body.into_inner();
    request.field = field;

    match ai_service.regenerate_field(user_id, request).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/ai/industries
/// List predefined industries
#[get("/industries")]
async fn list_industries(
    ai_service: web::Data<AiStartupEngineService>,
) -> HttpResponse {
    match ai_service.list_industries().await {
        Ok(industries) => success_response(serde_json::json!({
            "industries": industries
        })),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/ai/regulatory/{country_code}
/// Get regulatory requirements for a country
#[get("/regulatory/{country_code}")]
async fn get_regulatory_requirements(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    ai_service: web::Data<AiStartupEngineService>,
) -> HttpResponse {
    let country_code = path.into_inner();
    let industry = query.get("industry").map(|s| s.as_str());

    match ai_service.get_regulatory_requirements(&country_code, industry).await {
        Ok(requirements) => success_response(serde_json::json!({
            "country": country_code,
            "industry": industry,
            "requirements": requirements
        })),
        Err(e) => e.into_response(),
    }
}
