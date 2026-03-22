//! AI Startup Engine Models
//! Complete implementation per specification

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// GENERATION LOGS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GenerationLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Option<Uuid>,
    pub input_data: Value,
    pub onboarding_session_id: Option<Uuid>,
    pub ai_model: String,
    pub prompt_sent: Option<String>,
    pub raw_ai_response: Option<String>,
    pub parsed_output: Option<Value>,
    pub processing_time_ms: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub estimated_cost: Option<f64>,
    pub status: String, // 'processing', 'completed', 'failed', 'partial'
    pub error_message: Option<String>,
    pub blueprint: Option<Value>,
    pub confidence_overall: Option<f64>,
    pub confidence_industry: Option<f64>,
    pub confidence_revenue: Option<f64>,
    pub confidence_name: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGenerationLogRequest {
    pub user_id: Uuid,
    pub business_id: Option<Uuid>,
    pub input_data: Value,
    pub onboarding_session_id: Option<Uuid>,
    pub ai_model: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateGenerationLogRequest {
    pub status: Option<String>,
    pub prompt_sent: Option<String>,
    pub raw_ai_response: Option<String>,
    pub parsed_output: Option<Value>,
    pub processing_time_ms: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub estimated_cost: Option<f64>,
    pub error_message: Option<String>,
    pub blueprint: Option<Value>,
    pub confidence_overall: Option<f64>,
    pub confidence_industry: Option<f64>,
    pub confidence_revenue: Option<f64>,
    pub confidence_name: Option<f64>,
}

// ============================================================================
// AI VALIDATION LOGS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiValidationLog {
    pub id: Uuid,
    pub generation_log_id: Uuid,
    pub field_name: String,
    pub original_value: Option<String>,
    pub corrected_value: Option<String>,
    pub validation_rule: Option<String>,
    pub action_taken: Option<String>, // 'auto_fixed', 'flagged_for_review', 'accepted'
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateValidationLogRequest {
    pub generation_log_id: Uuid,
    pub field_name: String,
    pub original_value: Option<String>,
    pub corrected_value: Option<String>,
    pub validation_rule: Option<String>,
    pub action_taken: String,
}

// ============================================================================
// INDUSTRY CLASSIFICATION CACHE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndustryClassificationCache {
    pub id: Uuid,
    pub keywords_hash: String,
    pub keywords_text: Option<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub confidence_score: f64,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCacheEntryRequest {
    pub keywords_hash: String,
    pub keywords_text: Option<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub confidence_score: f64,
}

// ============================================================================
// REGULATORY REQUIREMENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RegulatoryRequirement {
    pub id: Uuid,
    pub country_code: String,
    pub country_name: String,
    pub requirement_type: String,
    pub requirement_name: String,
    pub description: Option<String>,
    pub applicable_industries: Value,
    pub applicable_business_types: Value,
    pub estimated_time_days: Option<i32>,
    pub estimated_cost_min: Option<f64>,
    pub estimated_cost_max: Option<f64>,
    pub currency: Option<String>,
    pub required_documents: Value,
    pub authority_name: Option<String>,
    pub authority_website: Option<String>,
    pub authority_contact_email: Option<String>,
    pub authority_contact_phone: Option<String>,
    pub is_mandatory: bool,
    pub priority: i32,
    pub condition_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

// ============================================================================
// INDUSTRY DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndustryDefinition {
    pub id: Uuid,
    pub industry_code: String,
    pub industry_name: String,
    pub description: Option<String>,
    pub classification_keywords: Value,
    pub primary_revenue_models: Value,
    pub secondary_revenue_models: Value,
    pub typical_startup_costs: Option<Value>,
    pub average_time_to_revenue_months: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubIndustryDefinition {
    pub id: Uuid,
    pub industry_code: String,
    pub sub_industry_code: String,
    pub sub_industry_name: String,
    pub description: Option<String>,
    pub classification_keywords: Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// AI PROCESSING REQUESTS/RESPONSES
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessStartupRequest {
    pub business_id: Option<Uuid>,
    pub onboarding_data: OnboardingData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnboardingData {
    pub business_idea: String,
    pub country: String,
    pub founder_type: String,
    pub optional_context: Option<OptionalContext>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptionalContext {
    pub target_customers: Option<String>,
    pub industry: Option<String>,
    pub revenue_model: Option<String>,
    pub problem_statement: Option<String>,
    pub solution_description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessStartupResponse {
    pub generation_id: Uuid,
    pub status: String,
    pub estimated_time: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStatusResponse {
    pub generation_id: Uuid,
    pub status: String,
    pub blueprint: Option<StartupBlueprint>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegenerateFieldRequest {
    pub startup_id: Uuid,
    pub field: String,
    pub context: Option<String>,
}

// ============================================================================
// STARTUP BLUEPRINT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupBlueprint {
    pub business_identity: BusinessIdentity,
    pub market_intelligence: MarketIntelligence,
    pub business_model: BusinessModel,
    pub compliance_requirements: ComplianceRequirements,
    pub ai_confidence: AiConfidence,
    pub suggested_next_steps: Vec<String>,
    pub generation_metadata: GenerationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessIdentity {
    pub business_name: String,
    pub alternative_names: Vec<String>,
    pub tagline: String,
    pub elevator_pitch: String,
    pub mission_statement: String,
    pub vision_statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIntelligence {
    pub industry: String,
    pub sub_industry: Option<String>,
    pub value_proposition: String,
    pub problem_statement: String,
    pub solution_description: String,
    pub target_customers: String,
    pub target_customer_description: String,
    pub market_size_estimate: String,
    pub competitive_advantage: String,
    pub key_challenges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessModel {
    pub primary_revenue_model: String,
    pub primary_model_description: String,
    pub secondary_revenue_models: Vec<RevenueModel>,
    pub pricing_suggestions: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueModel {
    pub model: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirements {
    pub country: String,
    pub registrations: Vec<RegistrationRequirement>,
    pub total_estimated_timeline: i32,
    pub total_estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequirement {
    pub name: String,
    pub authority: String,
    pub timeline_days: i32,
    pub cost_estimate: f64,
    pub priority: i32,
    pub documents_required: Vec<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfidence {
    pub overall_score: f32,
    pub industry_classification: f32,
    pub revenue_model: f32,
    pub business_name: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub model_used: String,
    pub processing_time_ms: i32,
    pub tokens_used: i32,
    pub generated_at: DateTime<Utc>,
}

// ============================================================================
// AI PROMPT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct BusinessAnalysisPrompt {
    pub business_idea: String,
    pub country: String,
    pub founder_type: String,
    pub target_customers: Option<String>,
    pub industry_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessAnalysisResponse {
    pub business_name: String,
    pub alternative_names: Vec<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub value_proposition: String,
    pub problem_statement: String,
    pub solution_description: String,
    pub target_customers: String,
    pub target_customer_description: String,
    pub suggested_revenue_models: Vec<String>,
    pub market_size_estimate: String,
    pub competitive_advantage: String,
    pub key_challenges: Vec<String>,
    pub suggested_next_steps: Vec<String>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnhancementPrompt {
    pub business_name: String,
    pub industry: String,
    pub value_proposition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementResponse {
    pub tagline: String,
    pub elevator_pitch: String,
    pub mission_statement: String,
    pub vision_statement: String,
    pub key_metrics: Vec<String>,
    pub risk_factors: Vec<String>,
    pub growth_strategy: String,
    pub team_needs: Vec<String>,
}

// ============================================================================
// CACHE QUERY RESULT
// ============================================================================

#[derive(Debug, Clone)]
pub struct CacheQueryResult {
    pub found: bool,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub confidence: Option<f32>,
}

// ============================================================================
// FALLBACK TEMPLATES
// ============================================================================

pub const FALLBACK_INDUSTRIES: &[&str] = &[
    "Fintech", "Agritech", "Healthtech", "Edtech", "E-commerce", 
    "SaaS", "Logistics", "Marketplace", "Media", "CleanTech", "PropTech", "Other"
];

pub const FALLBACK_REVENUE_MODELS: &[&str] = &[
    "Subscription", "Transaction Fee", "Commission", "Advertising", 
    "Freemium", "Licensing", "SaaS", "Marketplace Fee", "Product Sales", "Services"
];
