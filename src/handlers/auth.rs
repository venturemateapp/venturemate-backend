use actix_web::{post, get, web, HttpRequest, HttpResponse, HttpMessage};
use uuid::Uuid;

use crate::models::{
    CreateUserRequest, EmailVerificationRequest, LoginRequest,
    PasswordResetRequest, PasswordUpdateRequest, RefreshTokenRequest, ChangePasswordRequest,
    ResendVerificationRequest, ApiResponse, MessageResponse, GoogleOAuthCallbackRequest,
};
use crate::services::AuthService;
use crate::utils::success_response;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            // Registration & Login
            .service(register)
            .service(login)
            .service(google_auth_url)
            .service(google_callback)
            // Token management
            .service(refresh_token)
            .service(logout)
            // Password reset
            .service(request_password_reset)
            .service(reset_password)
            .service(change_password)
            // Email verification
            .service(verify_email)
            .service(resend_verification)
            // Status
            .service(get_auth_status),
    );
}

// ============================================================================
// REGISTRATION & LOGIN
// ============================================================================

#[post("/register")]
async fn register(
    body: web::Json<CreateUserRequest>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let ip = extract_ip(&req);
    let user_agent = extract_user_agent(&req);

    match auth_service.register(body.into_inner(), ip, user_agent.as_deref()).await {
        Ok(auth_response) => {
            HttpResponse::Created().json(ApiResponse::success(auth_response))
        }
        Err(e) => e.into_response(),
    }
}

#[post("/login")]
async fn login(
    body: web::Json<LoginRequest>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let ip = extract_ip(&req);
    let user_agent = extract_user_agent(&req);

    match auth_service.login(body.into_inner(), ip, user_agent.as_deref()).await {
        Ok(auth_response) => success_response(auth_response),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// GOOGLE OAUTH
// ============================================================================

#[get("/google")]
async fn google_auth_url(auth_service: web::Data<AuthService>) -> HttpResponse {
    // Generate OAuth URL for Google
    let client_id = &auth_service.google_client_id;
    let redirect_uri = format!("{}/auth/google/callback", auth_service.frontend_url);
    
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline&prompt=consent",
        client_id, redirect_uri
    );
    
    success_response(serde_json::json!({
        "auth_url": auth_url
    }))
}

#[get("/google/callback")]
async fn google_callback(
    query: web::Query<GoogleOAuthCallbackRequest>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let ip = extract_ip(&req);
    let user_agent = extract_user_agent(&req);

    match auth_service.google_oauth(&query.code, ip, user_agent.as_deref()).await {
        Ok(auth_response) => {
            // Redirect to frontend with tokens
            let redirect_url = format!(
                "{}/auth/callback?access_token={}&refresh_token={}",
                auth_service.frontend_url,
                auth_response.tokens.access_token,
                auth_response.tokens.refresh_token
            );
            HttpResponse::Found()
                .append_header(("Location", redirect_url))
                .finish()
        }
        Err(e) => {
            // Redirect to error page
            let error_url = format!(
                "{}/auth/error?message={}",
                auth_service.frontend_url,
                urlencoding::encode(&e.to_string())
            );
            HttpResponse::Found()
                .append_header(("Location", error_url))
                .finish()
        }
    }
}

// ============================================================================
// TOKEN MANAGEMENT
// ============================================================================

#[post("/refresh")]
async fn refresh_token(
    body: web::Json<RefreshTokenRequest>,
    auth_service: web::Data<AuthService>,
) -> HttpResponse {
    match auth_service.refresh_token(body.into_inner()).await {
        Ok(tokens) => success_response(tokens),
        Err(e) => e.into_response(),
    }
}

#[post("/logout")]
async fn logout(
    req: HttpRequest,
    auth_service: web::Data<AuthService>,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "UNAUTHORIZED",
            "Authentication required"
        )),
    };

    // Extract token from Authorization header
    let token = extract_token(&req).unwrap_or_default();

    match auth_service.logout(user_id, &token).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success(MessageResponse {
            message: "Logged out successfully".to_string(),
        })),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// PASSWORD RESET
// ============================================================================

#[post("/forgot-password")]
async fn request_password_reset(
    body: web::Json<PasswordResetRequest>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let ip = extract_ip(&req);
    let user_agent = extract_user_agent(&req);

    match auth_service.request_password_reset(body.into_inner(), ip, user_agent.as_deref()).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

#[post("/reset-password")]
async fn reset_password(
    body: web::Json<PasswordUpdateRequest>,
    auth_service: web::Data<AuthService>,
) -> HttpResponse {
    match auth_service.reset_password(body.into_inner()).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

#[post("/change-password")]
async fn change_password(
    body: web::Json<ChangePasswordRequest>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "UNAUTHORIZED",
            "Authentication required"
        )),
    };

    let ip = extract_ip(&req);
    let user_agent = extract_user_agent(&req);

    match auth_service.change_password(user_id, body.into_inner(), ip, user_agent.as_deref()).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// EMAIL VERIFICATION
// ============================================================================

#[post("/verify-email")]
async fn verify_email(
    body: web::Json<EmailVerificationRequest>,
    auth_service: web::Data<AuthService>,
) -> HttpResponse {
    match auth_service.verify_email(body.into_inner()).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

#[post("/resend-verification")]
async fn resend_verification(
    body: web::Json<ResendVerificationRequest>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> HttpResponse {
    let ip = extract_ip(&req);
    let user_agent = extract_user_agent(&req);

    match auth_service.resend_verification(body.into_inner(), ip, user_agent.as_deref()).await {
        Ok(response) => success_response(response),
        Err(e) => e.into_response(),
    }
}

// ============================================================================
// STATUS
// ============================================================================

#[get("/status")]
async fn get_auth_status(
    req: HttpRequest,
    auth_service: web::Data<AuthService>,
    _jwt: web::Data<crate::utils::Jwt>,
) -> HttpResponse {
    // Try to get user_id from middleware first
    if let Some(user_id) = get_user_id(&req) {
        match auth_service.get_user_response(user_id).await {
            Ok(user) => success_response(serde_json::json!({
                "authenticated": true,
                "user": user
            })),
            Err(_) => success_response(serde_json::json!({
                "authenticated": false,
                "user": null
            })),
        }
    } else {
        success_response(serde_json::json!({
            "authenticated": false,
            "user": null
        }))
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn get_user_id(req: &HttpRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}

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
