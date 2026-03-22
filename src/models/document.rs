use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub document_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_url: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub version: i32,
    pub is_public: bool,
    pub metadata: Option<Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDocument {
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub document_type: String,
    pub title: String,
    pub description: Option<String>,
    pub file_url: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub is_public: Option<bool>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDocument {
    pub title: Option<String>,
    pub description: Option<String>,
    pub file_url: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub is_public: Option<bool>,
    pub metadata: Option<Value>,
    pub status: Option<String>,
}
