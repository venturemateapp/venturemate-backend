// Auth-related models complementing user.rs

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // User ID
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub jti: String,      // JWT ID (for token revocation)
    pub session_id: String, // Session ID for revocation support
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenData {
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
}

/// Device fingerprint for session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub user_agent: String,
    pub ip_address: String,
}

// ============================================================================
// GOOGLE OAUTH MODELS
// ============================================================================

/// Google OAuth callback request
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleOAuthCallbackRequest {
    pub code: String,
    pub state: String,
}

/// Google user info from ID token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,           // Google's unique user ID
    pub email: String,
    pub email_verified: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

/// OAuth state for CSRF protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub redirect_url: String,
    pub created_at: DateTime<Utc>,
}

/// OAuth token response from Google
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

// ============================================================================
// EMAIL VERIFICATION MODELS
// ============================================================================

/// Email verification token (already in user.rs, keeping for compatibility)
pub use super::user::EmailVerificationToken;

/// Email verification response
#[derive(Debug, Clone, Serialize)]
pub struct VerifyEmailResponse {
    pub success: bool,
    pub message: String,
    pub redirect_to: Option<String>,
}

// ============================================================================
// RATE LIMITING MODELS
// ============================================================================

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: i32,
    pub reset_at: DateTime<Utc>,
    pub blocked_until: Option<DateTime<Utc>>,
}

/// Rate limit log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RateLimitLog {
    pub id: Uuid,
    pub identifier: String,
    pub identifier_type: String,
    pub action: String,
    pub ip_address: Option<IpNetwork>,
    pub user_agent: Option<String>,
    pub allowed: bool,
    pub blocked_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_attempts: i32,
    pub window_seconds: i64,
    pub block_duration_seconds: i64,
}

impl RateLimitConfig {
    /// Login attempts: 5 per 5 minutes
    pub fn login() -> Self {
        Self {
            max_attempts: 5,
            window_seconds: 300, // 5 minutes
            block_duration_seconds: 1800, // 30 minutes
        }
    }
    
    /// Registration: 3 per hour per IP
    pub fn registration() -> Self {
        Self {
            max_attempts: 3,
            window_seconds: 3600, // 1 hour
            block_duration_seconds: 3600, // 1 hour
        }
    }
    
    /// Password reset: 3 per hour per email
    pub fn password_reset() -> Self {
        Self {
            max_attempts: 3,
            window_seconds: 3600, // 1 hour
            block_duration_seconds: 3600, // 1 hour
        }
    }
    
    /// Email verification resend: 3 per hour per email
    pub fn verification_resend() -> Self {
        Self {
            max_attempts: 3,
            window_seconds: 3600, // 1 hour
            block_duration_seconds: 3600, // 1 hour
        }
    }
}

// ============================================================================
// SECURITY EVENTS
// ============================================================================

/// Security alert types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAlertType {
    MultipleFailedLogins,
    ImpossibleTravel,
    PasswordResetFlood,
    SuspiciousUserAgent,
    AccountLocked,
    SessionHijackingAttempt,
}

/// Security alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub alert_type: SecurityAlertType,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub description: String,
    pub severity: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// AUTH ERRORS
// ============================================================================

/// Auth-specific error types
#[derive(Debug, Clone)]
pub enum AuthErrorType {
    InvalidCredentials,
    AccountLocked,
    AccountSuspended,
    EmailNotVerified,
    TokenExpired,
    TokenInvalid,
    TokenRevoked,
    RateLimited,
    UserNotFound,
    EmailExists,
    WeakPassword,
    InvalidRequest,
    ServerError,
}

impl AuthErrorType {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "Invalid email or password",
            Self::AccountLocked => "Account is temporarily locked due to too many failed attempts",
            Self::AccountSuspended => "Account has been suspended",
            Self::EmailNotVerified => "Please verify your email before logging in",
            Self::TokenExpired => "Token has expired",
            Self::TokenInvalid => "Invalid token",
            Self::TokenRevoked => "Token has been revoked",
            Self::RateLimited => "Too many attempts. Please try again later",
            Self::UserNotFound => "User not found",
            Self::EmailExists => "An account with this email already exists",
            Self::WeakPassword => "Password does not meet security requirements",
            Self::InvalidRequest => "Invalid request",
            Self::ServerError => "An error occurred. Please try again",
        }
    }
    
    pub fn status_code(&self) -> u16 {
        match self {
            Self::InvalidCredentials => 401,
            Self::AccountLocked => 423,
            Self::AccountSuspended => 403,
            Self::EmailNotVerified => 403,
            Self::TokenExpired => 401,
            Self::TokenInvalid => 401,
            Self::TokenRevoked => 401,
            Self::RateLimited => 429,
            Self::UserNotFound => 404,
            Self::EmailExists => 409,
            Self::WeakPassword => 400,
            Self::InvalidRequest => 400,
            Self::ServerError => 500,
        }
    }
}
