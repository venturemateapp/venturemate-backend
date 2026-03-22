use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// AI CONVERSATION MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiConversation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Option<Uuid>,
    pub session_type: String, // onboarding, business_plan, branding, fundraising, etc.
    pub status: String,
    pub context: Value,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiChatMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String, // user, assistant, system
    pub content: String,
    pub ai_model: Option<String>,
    pub tokens_used: Option<i32>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

// ============================================
// REQUEST/RESPONSE MODELS
// ============================================

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateConversationRequest {
    pub business_id: Option<Uuid>,
    #[validate(length(min = 1, message = "Session type is required"))]
    pub session_type: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct SendMessageRequest {
    #[validate(length(min = 1, message = "Message content is required"))]
    pub content: String,
    pub context_updates: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub session_type: String,
    pub status: String,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageResponse {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub ai_model: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatWithAiResponse {
    pub message: ChatMessageResponse,
    pub ai_response: ChatMessageResponse,
    pub actions: Vec<AiAction>,
    pub context_updates: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAction {
    pub action_type: String, // generate_business_plan, create_business, etc.
    pub params: Value,
    pub description: String,
}

// ============================================
// AI CONTENT GENERATION MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiGeneratedContent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Option<Uuid>,
    pub content_type: String, // business_plan, pitch_deck, brand_strategy, etc.
    pub status: String,
    pub title: Option<String>,
    pub content: Value,
    pub raw_content: Option<String>,
    pub ai_model: Option<String>,
    pub generation_params: Value,
    pub tokens_used: Option<i32>,
    pub generation_time_ms: Option<i32>,
    pub version: i32,
    pub parent_version_id: Option<Uuid>,
    pub user_rating: Option<i32>,
    pub user_feedback: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct GenerateContentRequest {
    pub business_id: Option<Uuid>,
    #[validate(length(min = 1, message = "Content type is required"))]
    pub content_type: String, // business_plan, pitch_deck, brand_strategy, financial_model
    pub title: Option<String>,
    pub params: Value, // Generation parameters
    pub context: Option<String>, // Additional context
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegenerateContentRequest {
    pub content_id: Uuid,
    pub section: Option<String>,
    pub feedback: Option<String>,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedContentResponse {
    pub id: Uuid,
    pub content_type: String,
    pub status: String,
    pub title: Option<String>,
    pub content: Value,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentGenerationStatus {
    pub id: Uuid,
    pub status: String,
    pub progress: i32, // 0-100
    pub estimated_seconds_remaining: Option<i32>,
}

// ============================================
// AI INTENT & ENTITY MODELS (for chat)
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIntent {
    pub intent: String, // create_business, generate_plan, ask_question, etc.
    pub confidence: f32,
    pub entities: Vec<AiEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEntity {
    pub entity_type: String, // business_name, industry, country, etc.
    pub value: String,
    pub start_pos: Option<usize>,
    pub end_pos: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContext {
    pub current_stage: String,
    pub collected_data: Value,
    pub pending_questions: Vec<String>,
    pub suggested_actions: Vec<String>,
}

use validator::Validate;
