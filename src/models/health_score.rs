//! Health Score Models
//!
//! Models for the Startup Health Score™ system that tracks
//! startup readiness across multiple dimensions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Health Score
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HealthScore {
    pub id: Uuid,
    pub business_id: Uuid,
    pub overall_score: i32,
    pub compliance_score: i32,
    pub revenue_score: i32,
    pub market_fit_score: i32,
    pub team_score: i32,
    pub operations_score: i32,
    pub funding_readiness_score: i32,
    pub score_breakdown: Option<serde_json::Value>,
    pub contributing_factors: Option<serde_json::Value>,
    pub recommendations_count: i32,
    pub calculated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreResponse {
    pub id: Uuid,
    pub business_id: Uuid,
    pub overall_score: i32,
    pub status: String,
    pub trend: String,
    pub components: HealthScoreComponents,
    pub contributing_factors: ContributingFactors,
    pub recommendations_count: i32,
    pub calculated_at: DateTime<Utc>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScoreComponents {
    pub compliance: ComponentScore,
    pub revenue: ComponentScore,
    pub market_fit: ComponentScore,
    pub team: ComponentScore,
    pub operations: ComponentScore,
    pub funding_readiness: ComponentScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScore {
    pub score: i32,
    pub weight: f32,
    pub breakdown: Option<serde_json::Value>,
    pub max_score: Option<i32>,
    pub grade: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContributingFactors {
    pub positive: Vec<String>,
    pub negative: Vec<String>,
}

// =============================================================================
// Health Score History
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HealthScoreHistory {
    pub id: Uuid,
    pub business_id: Uuid,
    pub overall_score: i32,
    pub compliance_score: i32,
    pub revenue_score: i32,
    pub market_fit_score: i32,
    pub team_score: i32,
    pub operations_score: i32,
    pub funding_readiness_score: i32,
    pub calculated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Market Fit Analysis
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketFitAnalysis {
    pub id: Uuid,
    pub business_id: Uuid,
    pub analysis_type: String,
    pub analyzed_content: Option<String>,
    pub content_url: Option<String>,
    pub ai_analysis: Option<serde_json::Value>,
    pub score_contribution: i32,
    pub analyzed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsiteAnalysisResult {
    pub clarity_score: i32,
    pub design_score: i32,
    pub messaging_score: i32,
    pub trust_score: i32,
    pub overall_score: i32,
    pub recommendations: Vec<String>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
}

// =============================================================================
// Health Score Calculation Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthScoreComponent {
    Compliance,
    Revenue,
    MarketFit,
    Team,
    Operations,
    FundingReadiness,
}

impl HealthScoreComponent {
    pub fn weight(&self) -> f32 {
        match self {
            Self::Compliance => 0.25,
            Self::Revenue => 0.25,
            Self::MarketFit => 0.20,
            Self::Team => 0.15,
            Self::Operations => 0.15,
            Self::FundingReadiness => 0.0, // Not included in overall
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Compliance => "Compliance",
            Self::Revenue => "Revenue",
            Self::MarketFit => "Market Fit",
            Self::Team => "Team",
            Self::Operations => "Operations",
            Self::FundingReadiness => "Funding Readiness",
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

pub fn get_score_status(score: i32) -> &'static str {
    match score {
        81..=100 => "excellent",
        61..=80 => "good",
        41..=60 => "fair",
        21..=40 => "critical",
        _ => "just_started",
    }
}

pub fn get_score_emoji(score: i32) -> &'static str {
    match score {
        81..=100 => "🟢",
        61..=80 => "🟡",
        41..=60 => "🟠",
        21..=40 => "🔴",
        _ => "⚫",
    }
}

pub fn get_score_label(score: i32) -> &'static str {
    match score {
        81..=100 => "Excellent",
        61..=80 => "Good",
        41..=60 => "Fair",
        21..=40 => "Critical",
        _ => "Just Started",
    }
}

// =============================================================================
// Requests & Responses
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct CalculateHealthScoreRequest {
    pub business_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_recalculate: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthScoreBreakdown {
    pub overall: i32,
    pub compliance: i32,
    pub revenue: i32,
    pub market_fit: i32,
    pub team: i32,
    pub operations: i32,
    pub funding_readiness: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthComponents {
    pub compliance: HealthComponentDetail,
    pub revenue: HealthComponentDetail,
    pub market_fit: HealthComponentDetail,
    pub team: HealthComponentDetail,
    pub operations: HealthComponentDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthComponentDetail {
    pub score: i32,
    pub max_score: i32,
    pub weight: f32,
    pub grade: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshHealthScoreRequest {
    pub business_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>, // 'all' or specific component
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthScoreRefreshResponse {
    pub calculation_id: Uuid,
    pub status: String,
    pub estimated_seconds: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetHealthScoreHistoryRequest {
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthScoreHistoryResponse {
    pub history: Vec<HealthScoreHistoryPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthScoreHistoryPoint {
    pub date: DateTime<Utc>,
    pub overall_score: i32,
    pub compliance_score: i32,
    pub revenue_score: i32,
    pub market_fit_score: i32,
    pub team_score: i32,
    pub operations_score: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyzeWebsiteRequest {
    pub business_id: Uuid,
    pub website_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeWebsiteResponse {
    pub analysis_id: Uuid,
    pub status: String,
    pub estimated_seconds: i32,
}

// =============================================================================
// Score Breakdown Structures
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScoreBreakdown {
    pub business_registration: i32,
    pub tax_id: i32,
    pub industry_licenses: i32,
    pub document_vault: i32,
    pub legal_structure: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueScoreBreakdown {
    pub bank_account_connected: i32,
    pub payment_gateway: i32,
    pub invoices_created: i32,
    pub revenue_generated: i32,
    pub financial_projections: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketFitScoreBreakdown {
    pub website_quality: i32,
    pub brand_identity: i32,
    pub marketing_copy: i32,
    pub social_media_presence: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamScoreBreakdown {
    pub founder_completeness: i32,
    pub key_roles_filled: i32,
    pub advisory_board: i32,
    pub team_documents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsScoreBreakdown {
    pub crm_setup: i32,
    pub document_management: i32,
    pub tools_integration: i32,
    pub processes_defined: i32,
}

// =============================================================================
// Calculation Log
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HealthScoreCalculation {
    pub id: Uuid,
    pub business_id: Uuid,
    pub calculation_type: String,
    pub component_calculated: Option<String>,
    pub old_score: Option<i32>,
    pub new_score: Option<i32>,
    pub calculation_details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
