use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BusinessIdea {
    pub id: Uuid,
    pub user_id: Uuid,
    pub idea_text: String,
    pub business_name: Option<String>,
    pub industry: Option<String>,
    pub target_customer: Option<String>,
    pub value_proposition: Option<String>,
    pub revenue_model: Option<String>,
    pub operating_model: Option<String>,
    pub funding_stage: Option<String>,
    pub ai_analysis: Option<Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBusinessIdea {
    pub user_id: Uuid,
    pub idea_text: String,
    pub business_name: Option<String>,
    pub industry: Option<String>,
    pub target_customer: Option<String>,
    pub value_proposition: Option<String>,
    pub revenue_model: Option<String>,
    pub operating_model: Option<String>,
    pub funding_stage: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBusinessIdea {
    pub idea_text: Option<String>,
    pub business_name: Option<String>,
    pub industry: Option<String>,
    pub target_customer: Option<String>,
    pub value_proposition: Option<String>,
    pub revenue_model: Option<String>,
    pub operating_model: Option<String>,
    pub funding_stage: Option<String>,
    pub ai_analysis: Option<Value>,
    pub status: Option<String>,
}
