// Phase 3: Credit Scoring Models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// CREDIT SCORE MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct CreditScore {
    pub id: Uuid,
    pub business_id: Uuid,
    pub overall_score: i32, // 300-850
    pub score_grade: String, // A, B, C, D, E, F
    pub risk_level: String, // low, moderate, high, very_high
    
    // Component scores
    pub payment_history_score: i32,
    pub financial_stability_score: i32,
    pub business_viability_score: i32,
    pub compliance_score: i32,
    pub market_position_score: i32,
    
    // Score breakdown
    pub score_breakdown: Value,
    pub factors_positive: Value,
    pub factors_negative: Value,
    
    // Credit limits and offers
    pub suggested_credit_limit: Option<i64>,
    pub currency: String,
    
    pub calculated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct CreditScoreHistory {
    pub id: Uuid,
    pub business_id: Uuid,
    pub score: i32,
    pub score_grade: String,
    pub change_from_previous: i32,
    pub reason: Option<String>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct CreditReport {
    pub id: Uuid,
    pub business_id: Uuid,
    pub report_type: String, // full, summary, investor
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub access_count: i32,
    pub report_data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct FinancingOffer {
    pub id: Uuid,
    pub provider_name: String,
    pub provider_type: String, // bank, fintech, investor
    pub offer_type: String, // loan, credit_line, invoice_financing, revenue_based
    pub title: String,
    pub description: Option<String>,
    pub min_amount: Option<i64>,
    pub max_amount: i64,
    pub currency: String,
    pub interest_rate_min: Option<f64>,
    pub interest_rate_max: Option<f64>,
    pub term_months_min: Option<i32>,
    pub term_months_max: Option<i32>,
    pub requirements: Value,
    pub required_credit_score_min: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct FinancingApplication {
    pub id: Uuid,
    pub business_id: Uuid,
    pub offer_id: Uuid,
    pub requested_amount: i64,
    pub status: String, // draft, submitted, under_review, approved, rejected, funded
    pub application_data: Value,
    pub submitted_at: Option<DateTime<Utc>>,
    pub decision_at: Option<DateTime<Utc>>,
    pub decision_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// REQUEST/RESPONSE MODELS
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct CalculateCreditScoreRequest {
    pub force_recalculate: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplyForFinancingRequest {
    pub offer_id: Uuid,
    pub requested_amount: i64,
    pub application_data: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditScoreResponse {
    pub business_id: Uuid,
    pub overall_score: i32,
    pub score_grade: String,
    pub risk_level: String,
    pub components: CreditComponents,
    pub factors: CreditFactors,
    pub suggested_credit_limit: Option<i64>,
    pub currency: String,
    pub calculated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditComponents {
    pub payment_history: ComponentDetail,
    pub financial_stability: ComponentDetail,
    pub business_viability: ComponentDetail,
    pub compliance: ComponentDetail,
    pub market_position: ComponentDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentDetail {
    pub score: i32,
    pub max_score: i32,
    pub weight: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditFactors {
    pub positive: Vec<String>,
    pub negative: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinancingOfferResponse {
    pub id: Uuid,
    pub provider_name: String,
    pub provider_type: String,
    pub offer_type: String,
    pub title: String,
    pub description: Option<String>,
    pub amount_range: String,
    pub interest_rate_range: String,
    pub term_range: String,
    pub eligibility: OfferEligibility,
}

#[derive(Debug, Clone, Serialize)]
pub struct OfferEligibility {
    pub is_eligible: bool,
    pub required_score: Option<i32>,
    pub current_score: i32,
    pub meets_requirements: bool,
    pub missing_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditDashboard {
    pub current_score: CreditScoreResponse,
    pub score_history: Vec<ScoreHistoryPoint>,
    pub available_offers: Vec<FinancingOfferResponse>,
    pub active_applications: Vec<ApplicationSummary>,
    pub credit_utilization: Option<CreditUtilization>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreHistoryPoint {
    pub score: i32,
    pub grade: String,
    pub change: i32,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationSummary {
    pub id: Uuid,
    pub provider_name: String,
    pub offer_type: String,
    pub requested_amount: i64,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreditUtilization {
    pub total_limit: i64,
    pub used_amount: i64,
    pub available_amount: i64,
    pub utilization_percentage: f64,
}
