use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse, HttpMessage};

use futures::StreamExt;
use uuid::Uuid;

use crate::models::{
    ConnectDomainRequest, CreateWebsiteRequest, PublishWebsiteRequest,
    UpdatePageRequest, UpdateWebsiteRequest,
};
use crate::services::WebsiteService;
use crate::utils::{success_response, AppError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/businesses/{business_id}/website")
            // Website CRUD
            .service(get_website)
            .service(create_website)
            .service(update_website)
            .service(delete_website)
            // Publishing
            .service(publish_website)
            .service(unpublish_website)
            // Domain
            .service(connect_domain)
            .service(check_domain_status)
            // Pages
            .service(get_page)
            .service(update_page)
            // Assets
            .service(upload_asset),
    )
    .service(
        web::scope("/website")
            .service(list_templates)
            .service(get_template),
    )
    .service(preview_website);
}

// ============================================================================
// WEBSITE CRUD
// ============================================================================

#[get("")]
async fn get_website(
    path: web::Path<Uuid>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service.get_website(business_id, user_id).await {
        Ok(site) => success_response(site),
        Err(e) => e.into_response(),
    }
}

#[post("")]
async fn create_website(
    path: web::Path<Uuid>,
    body: web::Json<CreateWebsiteRequest>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service
        .create_website(business_id, user_id, body.into_inner())
        .await
    {
        Ok(site) => HttpResponse::Created().json(serde_json::json!({"success": true, "data": site})),
        Err(e) => e.into_response(),
    }
}

#[patch("")]
async fn update_website(
    path: web::Path<Uuid>,
    body: web::Json<UpdateWebsiteRequest>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service
        .update_website(business_id, user_id, body.into_inner())
        .await
    {
        Ok(site) => success_response(site),
        Err(e) => e.into_response(),
    }
}

#[delete("")]
async fn delete_website(
    path: web::Path<Uuid>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service.delete_website(business_id, user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// PUBLISHING
// ============================================================================

#[post("/publish")]
async fn publish_website(
    path: web::Path<Uuid>,
    body: web::Json<PublishWebsiteRequest>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service
        .publish_website(business_id, user_id, body.into_inner())
        .await
    {
        Ok(site) => success_response(site),
        Err(e) => e.into_response(),
    }
}

#[post("/unpublish")]
async fn unpublish_website(
    path: web::Path<Uuid>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service.unpublish_website(business_id, user_id).await {
        Ok(site) => success_response(site),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// DOMAIN MANAGEMENT
// ============================================================================

#[post("/domain")]
async fn connect_domain(
    path: web::Path<Uuid>,
    body: web::Json<ConnectDomainRequest>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service
        .connect_domain(business_id, user_id, body.domain.clone())
        .await
    {
        Ok(site) => success_response(site),
        Err(e) => e.into_response(),
    }
}

#[get("/domain/status")]
async fn check_domain_status(
    path: web::Path<Uuid>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let business_id = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service.check_domain_status(business_id, user_id).await {
        Ok(status) => success_response(status),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// PAGES
// ============================================================================

#[get("/pages/{page_id}")]
async fn get_page(
    path: web::Path<(Uuid, String)>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let (business_id, page_id) = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service.get_page(business_id, page_id, user_id).await {
        Ok(page) => success_response(page),
        Err(e) => e.into_response(),
    }
}

#[patch("/pages/{page_id}")]
async fn update_page(
    path: web::Path<(Uuid, String)>,
    body: web::Json<UpdatePageRequest>,
    website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let (business_id, page_id) = path.into_inner();
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    match website_service
        .update_page(business_id, page_id, user_id, body.into_inner())
        .await
    {
        Ok(page) => success_response(page),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// TEMPLATES
// ============================================================================

#[get("/templates")]
async fn list_templates(website_service: web::Data<WebsiteService>) -> HttpResponse {
    match website_service.list_templates().await {
        Ok(templates) => success_response(templates),
        Err(e) => e.into_response(),
    }
}

#[get("/templates/{code}")]
async fn get_template(
    path: web::Path<String>,
    website_service: web::Data<WebsiteService>,
) -> HttpResponse {
    let code = path.into_inner();

    match website_service.get_template(&code).await {
        Ok(template) => success_response(template),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// ASSETS
// ============================================================================

#[post("/assets")]
async fn upload_asset(
    path: web::Path<Uuid>,
    mut payload: Multipart,
    _website_service: web::Data<WebsiteService>,
    req: HttpRequest,
) -> HttpResponse {
    let _business_id = path.into_inner();
    let _user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return crate::utils::ResponseBuilder::unauthorized("Not authenticated"),
    };

    // Handle file upload
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(_) => continue,
        };

        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or_default();

        if name == "file" {
            let _filename = content_disposition.get_filename();
            let mut data = Vec::new();

            while let Some(chunk) = field.next().await {
                if let Ok(chunk) = chunk {
                    data.extend_from_slice(&chunk);
                }
            }

            // Store file via service
            // TODO: Implement asset storage
            return success_response(json!({
                "asset_id": Uuid::new_v4().to_string(),
                "size": data.len(),
            }));
        }
    }

    AppError::Validation("No file provided".to_string()).into_response()
}

// ============================================================================
// PREVIEW
// ============================================================================

#[get("/preview/{subdomain}")]
async fn preview_website(
    path: web::Path<String>,
    _website_service: web::Data<WebsiteService>,
) -> HttpResponse {
    let subdomain = path.into_inner();

    // This would render the website HTML
    // For now, return a placeholder
    HttpResponse::Ok()
        .content_type("text/html")
        .body(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - VentureMate Site</title>
</head>
<body>
    <h1>Website Preview</h1>
    <p>Subdomain: {}</p>
    <p>This is a placeholder for the generated website.</p>
</body>
</html>"#,
            subdomain, subdomain
        ))
}

// ============================================================================
// HELPERS
// ============================================================================

fn get_user_id(req: &HttpRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}

use serde_json::json;
