// Phase 2: CRM Models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct Contact {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub contact_type: String, // lead, customer, partner, investor
    pub status: String, // new, contacted, qualified, proposal, closed_won, closed_lost
    pub source: Option<String>,
    pub notes: Option<String>,
    pub tags: Value,
    pub custom_fields: Value,
    pub last_contacted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct Deal {
    pub id: Uuid,
    pub business_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: String,
    pub stage: String, // prospecting, qualification, proposal, negotiation, closed_won, closed_lost
    pub probability: i32, // 0-100
    pub expected_close_date: Option<DateTime<Utc>>,
    pub actual_close_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub custom_fields: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct Activity {
    pub id: Uuid,
    pub business_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub activity_type: String, // call, email, meeting, task, note
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateContactRequest {
    #[validate(length(min = 1))]
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub contact_type: String,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateDealRequest {
    #[validate(length(min = 1))]
    pub title: String,
    pub contact_id: Option<Uuid>,
    pub description: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub stage: String,
    pub probability: Option<i32>,
    pub expected_close_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateActivityRequest {
    pub contact_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub activity_type: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrmDashboardStats {
    pub total_contacts: i64,
    pub total_deals: i64,
    pub deals_by_stage: Vec<StageCount>,
    pub total_pipeline_value: f64,
    pub recent_activities: Vec<Activity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageCount {
    pub stage: String,
    pub count: i64,
    pub total_value: f64,
}
