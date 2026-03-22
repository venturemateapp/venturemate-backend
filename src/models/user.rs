use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Core user account information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub country_code: String,
    pub timezone: String,
    pub google_id: Option<String>,
    pub current_subscription_id: Option<Uuid>,
    pub status: String, // 'active', 'suspended', 'deleted'
    pub metadata: Value,
    pub onboarding_completed: bool,
    pub onboarding_step: Option<String>,
    
    // Security fields per spec
    pub failed_login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub consent_tracking: Value,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Extended user profile information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    
    // Personal Info
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub city: Option<String>,
    
    // Professional Info
    pub job_title: Option<String>,
    pub company_name: Option<String>,
    pub industry: Option<String>,
    pub years_of_experience: Option<i32>,
    pub founder_type: Option<String>, // 'solo_founder', 'co_founder', 'team_member'
    pub startup_experience_level: Option<String>, // 'first_time', 'experienced', 'serial'
    
    // Preferences
    pub language_preference: String, // default 'en'
    pub email_notifications_enabled: bool,
    pub marketing_emails_enabled: bool,
    pub profile_visibility: String, // 'public', 'private', 'connections_only'
    
    // Avatar stored as BLOB (BYTEA)
    pub avatar_data: Option<Vec<u8>>,
    pub avatar_mime_type: Option<String>,
    pub avatar_updated_at: Option<DateTime<Utc>>,
    
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Session for JWT token management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub refresh_token_hash: String,
    pub device_fingerprint: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Active session info for display to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub is_current: bool,
}

/// Password reset token
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PasswordReset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Email verification token
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Audit log entry for security events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub event_category: String, // 'auth', 'security', 'profile', 'business', 'system'
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub description: Option<String>,
    pub metadata: Value,
    pub severity: String, // 'info', 'warning', 'critical'
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// REQUEST DTOs
// ============================================================================

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,
    #[validate(length(equal = 2, message = "Country code must be 2 characters"))]
    pub country_code: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub country_code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    #[serde(skip)]
    pub ip_address: Option<String>,
    #[serde(skip)]
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PasswordUpdateRequest {
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailVerificationRequest {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    
    // Extended profile fields
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub city: Option<String>,
    pub job_title: Option<String>,
    pub company_name: Option<String>,
    pub industry: Option<String>,
    pub years_of_experience: Option<i32>,
    pub founder_type: Option<String>,
    pub startup_experience_level: Option<String>,
    pub language_preference: Option<String>,
    pub email_notifications_enabled: Option<bool>,
    pub marketing_emails_enabled: Option<bool>,
    pub profile_visibility: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAvatarRequest {
    pub avatar_data: Vec<u8>, // Base64 decoded to bytes
    pub mime_type: String,
}

// ============================================================================
// RESPONSE DTOs
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>, // Data URI for avatar if available
    pub email_verified: bool,
    pub phone: Option<String>,
    pub country_code: String,
    pub timezone: String,
    pub subscription_tier: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub onboarding_completed: bool,
    pub businesses_count: i64,
    
    // Extended profile
    pub job_title: Option<String>,
    pub company_name: Option<String>,
    pub industry: Option<String>,
    pub profile_visibility: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub city: Option<String>,
    pub job_title: Option<String>,
    pub company_name: Option<String>,
    pub industry: Option<String>,
    pub years_of_experience: Option<i32>,
    pub founder_type: Option<String>,
    pub startup_experience_level: Option<String>,
    pub language_preference: String,
    pub email_notifications_enabled: bool,
    pub marketing_emails_enabled: bool,
    pub profile_visibility: String,
    pub has_avatar: bool,
    pub avatar_mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub tokens: TokenPair,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationStatusResponse {
    pub email: String,
    pub verified: bool,
    pub resent_at: Option<DateTime<Utc>>,
    pub can_resend_at: Option<DateTime<Utc>>,
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            avatar_url: None, // Will be populated by service with avatar data
            email_verified: user.email_verified_at.is_some(),
            phone: user.phone,
            country_code: user.country_code,
            timezone: user.timezone,
            subscription_tier: None, // Will be populated by service
            created_at: user.created_at,
            updated_at: user.updated_at,
            onboarding_completed: user.onboarding_completed,
            businesses_count: 0, // Will be populated by service
            job_title: None,
            company_name: None,
            industry: None,
            profile_visibility: "private".to_string(),
        }
    }
}

impl From<UserProfile> for ProfileResponse {
    fn from(profile: UserProfile) -> Self {
        Self {
            id: profile.id,
            user_id: profile.user_id,
            date_of_birth: profile.date_of_birth,
            city: profile.city,
            job_title: profile.job_title,
            company_name: profile.company_name,
            industry: profile.industry,
            years_of_experience: profile.years_of_experience,
            founder_type: profile.founder_type,
            startup_experience_level: profile.startup_experience_level,
            language_preference: profile.language_preference,
            email_notifications_enabled: profile.email_notifications_enabled,
            marketing_emails_enabled: profile.marketing_emails_enabled,
            profile_visibility: profile.profile_visibility,
            has_avatar: profile.avatar_data.is_some(),
            avatar_mime_type: profile.avatar_mime_type,
        }
    }
}

// ============================================================================
// EVENT TYPES FOR AUDIT LOGS
// ============================================================================

pub const AUDIT_EVENT_REGISTRATION: &str = "user_registration";
pub const AUDIT_EVENT_LOGIN: &str = "user_login";
pub const AUDIT_EVENT_LOGIN_FAILED: &str = "login_failed";
pub const AUDIT_EVENT_LOGOUT: &str = "user_logout";
pub const AUDIT_EVENT_PASSWORD_CHANGED: &str = "password_changed";
pub const AUDIT_EVENT_PASSWORD_RESET_REQUEST: &str = "password_reset_request";
pub const AUDIT_EVENT_PASSWORD_RESET: &str = "password_reset";
pub const AUDIT_EVENT_EMAIL_VERIFIED: &str = "email_verified";
pub const AUDIT_EVENT_EMAIL_VERIFICATION_SENT: &str = "email_verification_sent";
pub const AUDIT_EVENT_PROFILE_UPDATED: &str = "profile_updated";
pub const AUDIT_EVENT_AVATAR_UPDATED: &str = "avatar_updated";
pub const AUDIT_EVENT_ACCOUNT_LOCKED: &str = "account_locked";
pub const AUDIT_EVENT_SESSION_REVOKED: &str = "session_revoked";
pub const AUDIT_EVENT_OAUTH_LOGIN: &str = "oauth_login";

pub const AUDIT_CATEGORY_AUTH: &str = "auth";
pub const AUDIT_CATEGORY_SECURITY: &str = "security";
pub const AUDIT_CATEGORY_PROFILE: &str = "profile";
pub const AUDIT_CATEGORY_BUSINESS: &str = "business";
pub const AUDIT_CATEGORY_SYSTEM: &str = "system";

pub const AUDIT_SEVERITY_INFO: &str = "info";
pub const AUDIT_SEVERITY_WARNING: &str = "warning";
pub const AUDIT_SEVERITY_CRITICAL: &str = "critical";
