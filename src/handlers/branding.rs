//! Branding Kit API Handlers
//! 
//! API endpoints for logo generation, color palettes, font pairings,
//! and complete brand asset management.

use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use tracing::{error, info};
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::models::branding::{GenerateBrandKitRequest, BrandKitStatusResponse, RegenerateLogoRequest};
use crate::services::BrandingService;
use crate::utils::get_user_id;

/// Configure branding routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/branding")
            .service(generate_brand_kit)
            .service(get_brand_kit_status)
            .service(regenerate_logo)
            .service(download_brand_kit)
            .service(get_color_presets)
            .service(get_font_presets)
            .service(get_generation_logs)
    );
}

/// POST /api/v1/branding/generate - Generate complete brand kit
#[post("/generate")]
async fn generate_brand_kit(
    service: web::Data<BrandingService>,
    req: web::Json<GenerateBrandKitRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    info!("Generating brand kit for user: {}", user_id);

    match service.generate_brand_kit(user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Accepted().json(ApiResponse::success(response)),
        Err(e) => {
            error!("Failed to generate brand kit: {}", e);
            e.into_response()
        }
    }
}

/// GET /api/v1/branding/status/{business_id} - Get brand kit status
#[get("/status/{business_id}")]
async fn get_brand_kit_status(
    service: web::Data<BrandingService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.get_brand_kit_status(user_id, business_id.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/branding/regenerate-logo - Regenerate logo
#[post("/regenerate-logo")]
async fn regenerate_logo(
    service: web::Data<BrandingService>,
    req: web::Json<RegenerateLogoRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    info!("Regenerating logo for user: {}", user_id);

    match service.regenerate_logo(user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Accepted().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/branding/{business_id}/download - Download brand kit ZIP
#[get("/{business_id}/download")]
async fn download_brand_kit(
    service: web::Data<BrandingService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.download_brand_kit(user_id, business_id.into_inner()).await {
        Ok((filename, content)) => HttpResponse::Ok()
            .content_type("application/zip")
            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
            .body(content),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/branding/color-presets - Get color palette presets
#[get("/color-presets")]
async fn get_color_presets(
    service: web::Data<BrandingService>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let category = query.get("category").cloned();

    match service.get_color_presets(category).await {
        Ok(presets) => HttpResponse::Ok().json(ApiResponse::success(presets)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/branding/font-presets - Get font pairing presets
#[get("/font-presets")]
async fn get_font_presets(
    service: web::Data<BrandingService>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let category = query.get("category").cloned();

    match service.get_font_presets(category).await {
        Ok(presets) => HttpResponse::Ok().json(ApiResponse::success(presets)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/branding/logs/{business_id} - Get generation logs
#[get("/logs/{business_id}")]
async fn get_generation_logs(
    service: web::Data<BrandingService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.get_generation_logs(user_id, business_id.into_inner()).await {
        Ok(logs) => HttpResponse::Ok().json(ApiResponse::success(logs)),
        Err(e) => e.into_response(),
    }
}
