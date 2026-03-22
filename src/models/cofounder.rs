use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================
// CO-FOUNDER PROFILE MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CofounderProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub skills: Value,
    pub expertise_areas: Value,
    pub experience_level: Option<String>,
    pub availability_hours: Option<i32>,
    pub commitment_type: Option<String>,
    pub equity_expectation_min: Option<i32>,
    pub equity_expectation_max: Option<i32>,
    pub looking_for_skills: Value,
    pub looking_for_commitment: Option<String>,
    pub preferred_industries: Value,
    pub bio: Option<String>,
    pub linkedin_url: Option<String>,
    pub portfolio_url: Option<String>,
    pub location: Option<String>,
    pub remote_ok: bool,
    pub match_score: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CofounderMatch {
    pub id: Uuid,
    pub user_id_1: Uuid,
    pub user_id_2: Uuid,
    pub match_score: i32,
    pub match_reasons: Value,
    pub status: String,
    pub initiated_by: Option<Uuid>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// REQUEST/RESPONSE MODELS
// ============================================

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateCofounderProfileRequest {
    #[validate(length(min = 1, message = "At least one skill is required"))]
    pub skills: Vec<String>,
    pub expertise_areas: Vec<String>,
    pub experience_level: String,
    pub availability_hours: i32,
    pub commitment_type: String,
    pub equity_expectation_min: Option<i32>,
    pub equity_expectation_max: Option<i32>,
    pub looking_for_skills: Vec<String>,
    pub looking_for_commitment: Option<String>,
    pub preferred_industries: Vec<String>,
    pub bio: Option<String>,
    pub linkedin_url: Option<String>,
    pub portfolio_url: Option<String>,
    pub location: Option<String>,
    pub remote_ok: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCofounderProfileRequest {
    pub skills: Option<Vec<String>>,
    pub expertise_areas: Option<Vec<String>>,
    pub experience_level: Option<String>,
    pub availability_hours: Option<i32>,
    pub commitment_type: Option<String>,
    pub equity_expectation_min: Option<i32>,
    pub equity_expectation_max: Option<i32>,
    pub looking_for_skills: Option<Vec<String>>,
    pub bio: Option<String>,
    pub linkedin_url: Option<String>,
    pub portfolio_url: Option<String>,
    pub location: Option<String>,
    pub remote_ok: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SendMatchRequest {
    #[validate(length(min = 1, message = "User ID is required"))]
    pub user_id: String, // UUID as string
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RespondToMatchRequest {
    pub accept: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CofounderProfileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub skills: Vec<String>,
    pub expertise_areas: Vec<String>,
    pub experience_level: String,
    pub availability_hours: i32,
    pub commitment_type: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub remote_ok: bool,
    pub match_score: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CofounderMatchResponse {
    pub id: Uuid,
    pub matched_user: MatchedUserInfo,
    pub match_score: i32,
    pub match_reasons: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedUserInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub skills: Vec<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchCofounderRequest {
    pub skills: Option<Vec<String>>,
    pub location: Option<String>,
    pub remote_ok: Option<bool>,
    pub commitment_type: Option<String>,
}
