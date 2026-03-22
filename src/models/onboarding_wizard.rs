//! Onboarding Wizard Models
//! Complete implementation per specification document

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================
// 1. ONBOARDING ANSWERS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OnboardingAnswer {
    pub id: Uuid,
    pub user_id: Uuid,
    pub startup_id: Option<Uuid>,
    pub session_id: Uuid,
    pub step_number: i32,
    pub question_key: String,
    pub answer_value: Option<String>,
    pub answer_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SaveStepAnswersRequest {
    pub session_id: Uuid,
    #[validate(range(min = 1, max = 5))]
    pub step: i32,
    pub answers: StepAnswers,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StepAnswers {
    Step1(CountrySelectionAnswers),
    Step2(FounderTypeAnswers),
    Step3(BusinessIdeaAnswers),
    Step4(BusinessContextAnswers),
    Step5(ReviewAnswers),
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CountrySelectionAnswers {
    #[validate(length(min = 1, message = "Country is required"))]
    pub country: String,
    pub secondary_countries: Option<Vec<String>>,
    pub has_physical_presence: Option<bool>,
    pub is_digital_only: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct FounderTypeAnswers {
    #[validate(length(min = 1, message = "Founder type is required"))]
    pub founder_type: String, // "solo" or "team"
    pub team_size: Option<i32>,
    pub cofounders: Option<Vec<CofounderInput>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CofounderInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub full_name: String,
    pub role: String,
    #[validate(range(min = 0.0, max = 100.0, message = "Equity must be between 0 and 100"))]
    pub equity_percentage: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct BusinessIdeaAnswers {
    #[validate(length(min = 50, message = "Business idea must be at least 50 characters"))]
    #[validate(length(max = 5000, message = "Business idea must not exceed 5000 characters"))]
    pub business_idea: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BusinessContextAnswers {
    pub target_customers: Option<String>,
    pub b2b_segment: Option<String>,
    pub revenue_model: Option<Vec<String>>,
    pub current_stage: Option<String>,
    pub industry: Option<Vec<String>>,
    pub funding_status: Option<String>,
    pub funding_amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReviewAnswers {
    pub confirmed: bool,
    pub terms_accepted: bool,
}

// ============================================
// 2. FOUNDERS (Co-founders for Team Type)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Founder {
    pub id: Uuid,
    pub startup_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub invited_by: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub equity_percentage: Option<f64>,
    pub status: String,
    pub invitation_token: Option<String>,
    pub invitation_sent_at: Option<DateTime<Utc>>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub declined_at: Option<DateTime<Utc>>,
    pub declined_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct InviteCofounderRequest {
    #[validate(email)]
    pub email: String,
    pub full_name: String,
    pub role: String,
    #[validate(range(min = 0.0, max = 100.0))]
    pub equity: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchInviteCofoundersRequest {
    pub startup_id: Uuid,
    pub cofounders: Vec<InviteCofounderRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FounderInvitationResponse {
    pub founder_id: Uuid,
    pub email: String,
    pub status: String,
    pub invitation_token: String,
    pub invitation_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FounderResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub equity_percentage: Option<f64>,
    pub status: String,
    pub joined_at: Option<DateTime<Utc>>,
}

// ============================================
// 3. BUSINESS IDEAS (Versioned)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WizardBusinessIdea {
    pub id: Uuid,
    pub startup_id: Option<Uuid>,
    pub user_id: Uuid,
    pub session_id: Option<Uuid>,
    pub raw_idea_text: String,
    pub processed_idea_text: Option<String>,
    pub ai_enhanced_version: Option<String>,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub target_customers: Option<Value>,
    pub revenue_model: Option<Value>,
    pub current_stage: Option<String>,
    pub funding_status: Option<String>,
    pub version: i32,
    pub parent_version_id: Option<Uuid>,
    pub is_active: bool,
    pub language_detected: Option<String>,
    pub keywords: Option<Value>,
    pub complexity_score: Option<i32>,
    pub viability_score: Option<i32>,
    pub flagged_for_review: bool,
    pub flag_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateWizardBusinessIdeaRequest {
    #[validate(length(min = 50, max = 5000))]
    pub raw_idea_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WizardBusinessIdeaResponse {
    pub id: Uuid,
    pub raw_idea_text: String,
    pub processed_idea_text: Option<String>,
    pub ai_enhanced_version: Option<String>,
    pub industry: Option<String>,
    pub version: i32,
    pub is_active: bool,
    pub viability_score: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// 4. SUPPORTED COUNTRIES
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SupportedCountry {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub code_3: String,
    pub currency: String,
    pub currency_symbol: Option<String>,
    pub is_active: bool,
    pub supports_banking: bool,
    pub supports_investor_matching: bool,
    pub supports_marketplace: bool,
    pub regulatory_complexity_score: Option<i32>,
    pub available_services: Value,
    pub region: Option<String>,
    pub sub_region: Option<String>,
    pub flag_emoji: Option<String>,
    pub phone_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CountryResponse {
    pub name: String,
    pub code: String,
    pub currency: String,
    pub currency_symbol: Option<String>,
    pub flag_emoji: Option<String>,
    pub regulatory_complexity_score: Option<i32>,
    pub available_services: Value,
    pub supports_banking: bool,
    pub supports_investor_matching: bool,
    pub supports_marketplace: bool,
}

impl From<SupportedCountry> for CountryResponse {
    fn from(country: SupportedCountry) -> Self {
        Self {
            name: country.name,
            code: country.code,
            currency: country.currency,
            currency_symbol: country.currency_symbol,
            flag_emoji: country.flag_emoji,
            regulatory_complexity_score: country.regulatory_complexity_score,
            available_services: country.available_services,
            supports_banking: country.supports_banking,
            supports_investor_matching: country.supports_investor_matching,
            supports_marketplace: country.supports_marketplace,
        }
    }
}

// ============================================
// 5. WIZARD FLOW RESPONSES
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct StartOnboardingRequest {
    pub source: Option<String>, // where user came from
}

#[derive(Debug, Clone, Serialize)]
pub struct StartOnboardingResponse {
    pub session_id: Uuid,
    pub current_step: i32,
    pub progress_percentage: i32,
    pub first_step: StepContent,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepContent {
    pub step_number: i32,
    pub title: String,
    pub description: String,
    pub helper_text: Option<String>,
    pub fields: Vec<StepField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepField {
    pub key: String,
    pub label: String,
    pub field_type: String, // text, textarea, select, multiselect, radio, checkbox
    pub required: bool,
    pub placeholder: Option<String>,
    pub options: Option<Vec<FieldOption>>,
    pub validation: Option<FieldValidation>,
    pub tooltip: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldValidation {
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveStepResponse {
    pub success: bool,
    pub next_step: Option<i32>,
    pub progress_percentage: i32,
    pub next_step_content: Option<StepContent>,
    pub validation_errors: Option<Vec<ValidationError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteOnboardingResponse {
    pub startup_id: Uuid,
    pub startup_name: String,
    pub dashboard_url: String,
    pub processing_status: String,
    pub estimated_completion_seconds: i32,
}

// ============================================
// 6. ONBOARDING ANALYTICS
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct TrackOnboardingEventRequest {
    pub session_id: Uuid,
    pub event_type: String,
    pub step_number: Option<i32>,
    pub event_data: Option<Value>,
    pub time_spent_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingStats {
    pub total_started: i64,
    pub total_completed: i64,
    pub completion_rate: f64,
    pub avg_completion_time_seconds: Option<i64>,
    pub avg_time_per_step: Vec<StepTimeStats>,
    pub abandonment_by_step: Vec<StepAbandonmentStats>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepTimeStats {
    pub step_number: i32,
    pub avg_time_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepAbandonmentStats {
    pub step_number: i32,
    pub abandonment_count: i64,
    pub abandonment_rate: f64,
}

// ============================================
// 7. HELPER TYPES FOR WIZARD
// ============================================

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingSummary {
    pub session_id: Uuid,
    pub current_step: i32,
    pub country: Option<String>,
    pub founder_type: Option<String>,
    pub business_idea_preview: Option<String>,
    pub cofounders: Vec<FounderResponse>,
    pub can_complete: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResumeOnboardingRequest {
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResumeOnboardingResponse {
    pub session_id: Uuid,
    pub last_completed_step: i32,
    pub current_step: i32,
    pub progress_percentage: i32,
    pub saved_answers: Value,
    pub welcome_back_message: String,
}
