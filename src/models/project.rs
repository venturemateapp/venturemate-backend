use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub status: String,
    pub priority: i32,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub assigned_to: Option<Uuid>,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProject {
    pub user_id: Uuid,
    pub business_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<Uuid>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProject {
    pub title: Option<String>,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub due_date: Option<NaiveDate>,
    pub assigned_to: Option<Uuid>,
    pub metadata: Option<Value>,
}
