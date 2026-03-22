//! Document Generation API Handlers
//! 
//! API endpoints for business plans, pitch decks, and investor documents.

use actix_web::{get, post, web, HttpRequest, HttpResponse};
use tracing::{error, info};
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::models::documents::{GenerateBusinessPlanRequest, GeneratePitchDeckRequest, DocumentType};
use crate::services::DocumentGenerationService;
use crate::utils::get_user_id;

/// Configure document generation routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/documents")
            .service(generate_business_plan)
            .service(generate_pitch_deck)
            .service(get_document_status)
            .service(get_business_documents)
            .service(download_document)
            .service(get_pitch_deck_templates)
    );
}

/// POST /api/v1/documents/business-plan/generate - Generate business plan
#[post("/business-plan/generate")]
async fn generate_business_plan(
    service: web::Data<DocumentGenerationService>,
    req: web::Json<GenerateBusinessPlanRequest>,
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

    info!("Generating business plan for user: {}", user_id);

    match service.generate_business_plan(user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Accepted().json(ApiResponse::success(response)),
        Err(e) => {
            error!("Failed to generate business plan: {}", e);
            e.into_response()
        }
    }
}

/// POST /api/v1/documents/pitch-deck/generate - Generate pitch deck
#[post("/pitch-deck/generate")]
async fn generate_pitch_deck(
    service: web::Data<DocumentGenerationService>,
    req: web::Json<GeneratePitchDeckRequest>,
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

    info!("Generating pitch deck for user: {}", user_id);

    match service.generate_pitch_deck(user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Accepted().json(ApiResponse::success(response)),
        Err(e) => {
            error!("Failed to generate pitch deck: {}", e);
            e.into_response()
        }
    }
}

/// GET /api/v1/documents/status/{document_id} - Get document status
#[get("/status/{document_id}")]
async fn get_document_status(
    service: web::Data<DocumentGenerationService>,
    document_id: web::Path<Uuid>,
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

    match service.get_document_status(user_id, document_id.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/documents/business/{business_id} - List business documents
#[get("/business/{business_id}")]
async fn get_business_documents(
    service: web::Data<DocumentGenerationService>,
    business_id: web::Path<Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
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

    let doc_type = query.get("type")
        .and_then(|t| t.parse::<DocumentType>().ok());

    match service.get_business_documents(user_id, business_id.into_inner(), doc_type).await {
        Ok(documents) => HttpResponse::Ok().json(ApiResponse::success(documents)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/documents/{document_id}/download - Download document
#[get("/{document_id}/download")]
async fn download_document(
    service: web::Data<DocumentGenerationService>,
    document_id: web::Path<Uuid>,
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

    match service.download_document(user_id, document_id.into_inner()).await {
        Ok((filename, content)) => HttpResponse::Ok()
            .content_type("application/pdf")
            .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
            .body(content),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/documents/pitch-deck/templates - Get pitch deck templates
#[get("/pitch-deck/templates")]
async fn get_pitch_deck_templates(
    service: web::Data<DocumentGenerationService>,
) -> HttpResponse {
    match service.get_pitch_deck_templates().await {
        Ok(templates) => HttpResponse::Ok().json(ApiResponse::success(templates)),
        Err(e) => e.into_response(),
    }
}
