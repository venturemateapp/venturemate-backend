use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::ai_generation::{
    AIGenerateBusinessPlanRequest as GenerateBusinessPlanRequest,
    GenerateColorPaletteRequest, GenerateLogoRequest,
    AIGeneratePitchDeckRequest as GeneratePitchDeckRequest,
    RegenerateSectionRequest, SelectLogoRequest,
};
use crate::services::ai_service::AIService;
use crate::services::business_service::BusinessService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/businesses/{business_id}/generate")
            .service(generate_business_plan)
            .service(generate_pitch_deck)
            .service(generate_one_pager)
            .service(regenerate_section),
    )
    .service(
        web::scope("/businesses/{business_id}/branding")
            .service(generate_logo_options)
            .service(select_logo)
            .service(generate_color_palette)
            .service(update_brand_colors)
            .service(get_brand_guidelines),
    )
    .service(web::scope("/generation-jobs").service(get_job_status));
}

#[post("/business-plan")]
async fn generate_business_plan(
    path: web::Path<Uuid>,
    _req: web::Json<GenerateBusinessPlanRequest>,
    ai_service: web::Data<AIService>,
    business_service: web::Data<BusinessService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let business_id = path.into_inner();

    // Get business details
    let business = match business_service.get_by_id(business_id, user_id).await {
        Ok(b) => b,
        Err(e) => return e.error_response(),
    };

    // Start generation in background
    match ai_service
        .generate_business_plan(
            business_id,
            user_id,
            &business.description.unwrap_or_default(),
            &business.industry,
            &business.country_code,
        )
        .await
    {
        Ok(content) => ResponseBuilder::accepted(serde_json::json!({
            "status": "generating",
            "content": content,
        })),
        Err(e) => e.error_response(),
    }
}

#[post("/pitch-deck")]
async fn generate_pitch_deck(
    path: web::Path<Uuid>,
    _req: web::Json<GeneratePitchDeckRequest>,
    ai_service: web::Data<AIService>,
    business_service: web::Data<BusinessService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let business_id = path.into_inner();

    let business = match business_service.get_by_id(business_id, user_id).await {
        Ok(b) => b,
        Err(e) => return e.error_response(),
    };

    match ai_service
        .generate_pitch_deck(
            business_id,
            user_id,
            &business.name,
            &business.tagline.unwrap_or_default(),
            &business.industry,
            &business.stage,
        )
        .await
    {
        Ok(content) => ResponseBuilder::accepted(serde_json::json!({
            "status": "generating",
            "content": content,
        })),
        Err(e) => e.error_response(),
    }
}

#[post("/one-pager")]
async fn generate_one_pager() -> HttpResponse {
    ResponseBuilder::ok(serde_json::json!({ 
        "message": "One-pager generation not yet implemented" 
    }))
}

#[post("/regenerate")]
async fn regenerate_section(
    _req: web::Json<RegenerateSectionRequest>,
) -> HttpResponse {
    ResponseBuilder::ok(serde_json::json!({ 
        "message": "Section regeneration not yet implemented" 
    }))
}

#[get("/{job_id}")]
async fn get_job_status(path: web::Path<Uuid>) -> HttpResponse {
    // TODO: Get job status from database
    ResponseBuilder::ok(serde_json::json!({
        "job_id": path.into_inner().to_string(),
        "status": "completed",
        "progress": 100,
    }))
}

#[post("/generate-logos")]
async fn generate_logo_options(
    _path: web::Path<Uuid>,
    _req: web::Json<GenerateLogoRequest>,
) -> HttpResponse {
    ResponseBuilder::ok(serde_json::json!({ 
        "message": "Logo generation endpoint - integrate with DALL-E" 
    }))
}

#[post("/select-logo")]
async fn select_logo(
    _path: web::Path<Uuid>,
    _req: web::Json<SelectLogoRequest>,
) -> HttpResponse {
    ResponseBuilder::ok(serde_json::json!({ 
        "message": "Logo selection endpoint" 
    }))
}

#[post("/generate-colors")]
async fn generate_color_palette(
    path: web::Path<Uuid>,
    req: web::Json<GenerateColorPaletteRequest>,
    ai_service: web::Data<AIService>,
    business_service: web::Data<BusinessService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let business_id = path.into_inner();

    let business = match business_service.get_by_id(business_id, user_id).await {
        Ok(b) => b,
        Err(e) => return e.error_response(),
    };

    match ai_service
        .generate_color_palette(
            &business.name,
            &business.industry,
            req.mood.as_deref(),
            req.base_color.as_deref(),
        )
        .await
    {
        Ok(palette) => ResponseBuilder::ok(palette),
        Err(e) => e.error_response(),
    }
}

#[put("/colors")]
async fn update_brand_colors() -> HttpResponse {
    ResponseBuilder::ok(serde_json::json!({ 
        "message": "Brand colors update endpoint" 
    }))
}

#[get("/guidelines")]
async fn get_brand_guidelines() -> HttpResponse {
    ResponseBuilder::ok(serde_json::json!({ 
        "message": "Brand guidelines endpoint" 
    }))
}
