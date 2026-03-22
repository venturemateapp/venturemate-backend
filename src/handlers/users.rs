use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};

use crate::models::{PaginationParams, UpdateAvatarRequest, UpdateProfileRequest, ApiResponse};
use crate::services::{AuthService, UserService};
use crate::utils::{get_user_id, ResponseBuilder, AppError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            // Profile
            .service(get_profile)
            .service(update_profile)
            .service(upload_avatar)
            .service(get_avatar)
            .service(delete_account)
            // Sessions
            .service(list_sessions)
            .service(revoke_session)
            // List
            .service(list_users),
    );
}

// ============================================================================
// PROFILE MANAGEMENT
// ============================================================================

#[get("/me")]
async fn get_profile(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match auth_service.get_user_response(user_id).await {
        Ok(user) => ResponseBuilder::ok(user),
        Err(e) => e.error_response(),
    }
}

#[put("/me")]
async fn update_profile(
    body: web::Json<UpdateProfileRequest>,
    auth_service: web::Data<AuthService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let ip = extract_ip(&http_req);
    let user_agent = extract_user_agent(&http_req);

    match auth_service.update_profile(user_id, body.into_inner(), ip, user_agent.as_deref()).await {
        Ok(profile) => ResponseBuilder::ok(profile),
        Err(e) => e.error_response(),
    }
}

#[post("/me/avatar")]
async fn upload_avatar(
    mut payload: Multipart,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    // Process multipart upload
    let mut avatar_data: Option<Vec<u8>> = None;
    let mut mime_type: Option<String> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .get_name()
            .unwrap_or("");

        if field_name == "avatar" {
            // Get content type
            if let Some(ct) = field.content_type() {
                mime_type = Some(ct.to_string());
            }

            // Read file data
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(bytes) => data.extend_from_slice(&bytes),
                    Err(e) => {
                        return AppError::BadRequest(format!("Failed to read file: {}", e)).into_response();
                    }
                }
            }
            avatar_data = Some(data);
        }
    }

    match (avatar_data, mime_type) {
        (Some(data), Some(mime)) => {
            let ip = extract_ip(&req);
            let user_agent = extract_user_agent(&req);

            let request = UpdateAvatarRequest {
                avatar_data: data,
                mime_type: mime,
            };

            match auth_service.update_avatar(user_id, request, ip, user_agent.as_deref()).await {
                Ok(response) => ResponseBuilder::ok(response),
                Err(e) => e.error_response(),
            }
        }
        _ => AppError::BadRequest("Avatar file required".to_string()).into_response(),
    }
}

#[get("/me/avatar")]
async fn get_avatar(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match auth_service.get_avatar(user_id).await {
        Ok((data, mime_type)) => {
            HttpResponse::Ok()
                .content_type(mime_type)
                .body(data)
        }
        Err(e) => e.error_response(),
    }
}

#[delete("/me")]
async fn delete_account(
    user_service: web::Data<UserService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match user_service.delete_account(user_id).await {
        Ok(_) => ResponseBuilder::no_content(),
        Err(e) => e.error_response(),
    }
}

// ============================================================================
// SESSION MANAGEMENT
// ============================================================================

#[get("/me/sessions")]
async fn list_sessions(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
    jwt: web::Data<crate::utils::Jwt>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    // Get current session ID from token
    let current_session_id = extract_token(&req)
        .and_then(|token| jwt.extract_session_id(&token).ok());

    match auth_service.list_active_sessions(user_id, current_session_id).await {
        Ok(sessions) => ResponseBuilder::ok(sessions),
        Err(e) => e.error_response(),
    }
}

#[delete("/me/sessions/{session_id}")]
async fn revoke_session(
    path: web::Path<uuid::Uuid>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let session_id = path.into_inner();

    match auth_service.revoke_session(user_id, session_id).await {
        Ok(_) => ResponseBuilder::ok(ApiResponse::<()>::success(())),
        Err(e) => e.error_response(),
    }
}

// ============================================================================
// LIST USERS
// ============================================================================

#[get("")]
async fn list_users(
    query: web::Query<PaginationParams>,
    user_service: web::Data<UserService>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match user_service.list_users(page, per_page).await {
        Ok((users, total)) => {
            let response = crate::models::PaginatedResponse::new(users, total, page, per_page);
            ResponseBuilder::ok(response)
        }
        Err(e) => e.error_response(),
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn extract_ip(req: &HttpRequest) -> Option<std::net::IpAddr> {
    req.peer_addr().map(|addr| addr.ip())
}

fn extract_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}
