use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiGenerationJob {
    pub id: Uuid,
    pub business_id: Uuid,
    pub user_id: Uuid,
    pub job_type: String,
    pub status: String,
    pub progress: i32,
    pub input_params: Value,
    pub prompt_version: Option<String>,
    pub result: Option<Value>,
    pub output_urls: Value,
    pub ai_model: Option<String>,
    pub token_usage: Option<i32>,
    pub cost: Option<f64>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub webhook_url: Option<String>,
    pub webhook_delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GeneratedDocument {
    pub id: Uuid,
    pub business_id: Uuid,
    pub job_id: Option<Uuid>,
    pub document_type: String,
    pub version: i32,
    pub content: Option<Value>,
    pub file_url: Option<String>,
    pub file_size: Option<i64>,
    pub file_format: Option<String>,
    pub metadata: Value,
    pub ai_model: Option<String>,
    pub token_usage: Option<i32>,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiPrompt {
    pub id: Uuid,
    pub prompt_id: String,
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub user_template: String,
    pub variables: Value,
    pub output_schema: Option<Value>,
    pub model_config: Value,
    pub is_active: bool,
    pub use_count: i32,
    pub avg_tokens: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

// Request/Response structs

#[derive(Debug, Clone, Deserialize)]
pub struct AIGenerateBusinessPlanRequest {
    pub template: Option<String>,
    pub sections: Option<Vec<String>>,
    pub language: Option<String>,
    pub include_financials: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AIGeneratePitchDeckRequest {
    pub template: Option<String>,
    pub audience: Option<String>,
    pub slides_count: Option<i32>,
    pub include_financials: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateLogoRequest {
    pub style_preferences: Vec<String>,
    pub color_preferences: Vec<String>,
    pub concept_keywords: Vec<String>,
    pub variations_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateColorPaletteRequest {
    pub base_color: Option<String>,
    pub mood: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegenerateSectionRequest {
    pub document_type: String,
    pub section: String,
    pub instructions: String,
    pub tone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectLogoRequest {
    pub logo_id: Uuid,
    pub generate_variants: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationJobResponse {
    pub id: Uuid,
    pub job_type: String,
    pub status: String,
    pub progress: i32,
    pub result: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub token_usage: Option<i32>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateGenerationJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub estimated_duration: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogoOptionResponse {
    pub id: Uuid,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub style: String,
    pub colors: Vec<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogoOptionsResponse {
    pub options: Vec<LogoOptionResponse>,
    pub can_generate_more: bool,
    pub generations_remaining: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiColorPalette {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub neutral: String,
    pub background: String,
    pub text: String,
    pub success: String,
    pub warning: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiColorPaletteResponse {
    pub palette: AiColorPalette,
    pub ai_recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrandGuidelinesResponse {
    pub colors: AiColorPalette,
    pub typography: Typography,
    pub logo_usage: LogoUsage,
    pub messaging: Messaging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub headings: String,
    pub body: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoUsage {
    pub minimum_size: String,
    pub clearspace: String,
    pub dos: Vec<String>,
    pub donts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Messaging {
    pub tone: String,
    pub taglines: Vec<String>,
    pub key_messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessPlanContent {
    pub id: Uuid,
    pub business_id: Uuid,
    pub version: i32,
    pub content: Value,
    pub generated_at: DateTime<Utc>,
    pub ai_model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketingMaterialRequest {
    pub materials: Vec<MarketingMaterial>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketingMaterial {
    pub material_type: String,
    pub size: Option<String>,
    pub platform: Option<String>,
    pub orientation: Option<String>,
}

impl From<AiGenerationJob> for GenerationJobResponse {
    fn from(job: AiGenerationJob) -> Self {
        Self {
            id: job.id,
            job_type: job.job_type,
            status: job.status,
            progress: job.progress,
            result: job.result,
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            token_usage: job.token_usage,
            cost: job.cost,
        }
    }
}
