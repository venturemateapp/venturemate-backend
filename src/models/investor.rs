// Phase 3: Investor Matchmaking Models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================
// INVESTOR PROFILE MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct InvestorProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub investor_type: String, // angel, vc, family_office, corporate, accelerator
    pub firm_name: Option<String>,
    pub bio: Option<String>,
    pub website_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub location: Option<String>,
    pub preferred_countries: Value,
    pub investment_stage: Value, // pre_seed, seed, series_a, series_b, growth
    pub check_size_min: Option<i64>,
    pub check_size_max: Option<i64>,
    pub currency: String,
    pub preferred_industries: Value,
    pub past_investments: Value,
    pub thesis: Option<String>,
    pub value_add: Option<String>, // what they bring beyond money
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct InvestorMatch {
    pub id: Uuid,
    pub business_id: Uuid,
    pub investor_id: Uuid,
    pub match_score: i32, // 0-100
    pub match_reasons: Value,
    pub status: String, // pending, viewed, interested, passed, connected, pitched, invested
    pub business_pitch: Option<String>,
    pub investor_notes: Option<String>,
    pub meeting_scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct InvestorDataRoom {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub access_code: Option<String>,
    pub is_public: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub view_count: i32,
    pub download_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct InvestorDataRoomDocument {
    pub id: Uuid,
    pub data_room_id: Uuid,
    pub document_id: Uuid,
    pub folder_path: Option<String>,
    pub order_index: i32,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct InvestorDataRoomAccess {
    pub id: Uuid,
    pub data_room_id: Uuid,
    pub investor_id: Option<Uuid>,
    pub email: Option<String>,
    pub access_type: String, // view, download
    granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

// ============================================
// REQUEST/RESPONSE MODELS
// ============================================

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateInvestorProfileRequest {
    #[validate(length(min = 1))]
    pub investor_type: String,
    pub firm_name: Option<String>,
    pub bio: String,
    pub website_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub location: Option<String>,
    pub preferred_countries: Vec<String>,
    pub investment_stage: Vec<String>,
    pub check_size_min: Option<i64>,
    pub check_size_max: Option<i64>,
    pub currency: Option<String>,
    pub preferred_industries: Vec<String>,
    pub thesis: Option<String>,
    pub value_add: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchInvestorsRequest {
    pub stages: Option<Vec<String>>,
    pub industries: Option<Vec<String>>,
    pub countries: Option<Vec<String>>,
    pub min_check_size: Option<i64>,
    pub max_check_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitPitchRequest {
    pub investor_id: Uuid,
    pub pitch_message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvestorCreateDataRoomRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddDocumentToDataRoomRequest {
    pub document_id: Uuid,
    pub folder_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvestorProfileResponse {
    pub id: Uuid,
    pub investor_type: String,
    pub firm_name: Option<String>,
    pub bio: Option<String>,
    pub website_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub location: Option<String>,
    pub investment_stage: Vec<String>,
    pub check_size_range: String,
    pub preferred_industries: Vec<String>,
    pub thesis: Option<String>,
    pub value_add: Option<String>,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvestorMatchResponse {
    pub id: Uuid,
    pub investor: InvestorProfileResponse,
    pub match_score: i32,
    pub match_reasons: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvestorDataRoomResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub document_count: i64,
    pub view_count: i32,
    pub is_public: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchmakingStats {
    pub total_investors: i64,
    pub matched_investors: i64,
    pub pending_pitches: i64,
    pub interested_investors: i64,
    pub meetings_scheduled: i64,
    pub match_rate: f64,
}
