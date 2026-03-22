use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Business {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub slug: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub country_code: String,
    pub city: Option<String>,
    pub status: String,
    pub stage: String,
    pub legal_structure: Option<String>,
    pub registration_number: Option<String>,
    pub founded_date: Option<NaiveDate>,
    pub tax_id: Option<String>,
    pub logo_url: Option<String>,
    pub brand_colors: Value,
    pub website_url: Option<String>,
    pub custom_domain: Option<String>,
    pub health_score: Option<i32>,
    pub health_score_updated_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub settings: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBusinessRequest {
    #[validate(length(min = 1, message = "Business name is required"))]
    pub name: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    #[validate(length(min = 1, message = "Industry is required"))]
    pub industry: String,
    pub sub_industry: Option<String>,
    #[validate(length(equal = 2, message = "Country code must be 2 characters"))]
    pub country_code: String,
    pub city: Option<String>,
    pub legal_structure: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBusinessRequest {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
    pub sub_industry: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub stage: Option<String>,
    pub legal_structure: Option<String>,
    pub registration_number: Option<String>,
    pub founded_date: Option<NaiveDate>,
    pub tax_id: Option<String>,
    pub website_url: Option<String>,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub status: String,
    pub stage: String,
    pub country_code: String,
    pub city: Option<String>,
    pub health_score: Option<i32>,
    pub health_score_breakdown: Option<crate::models::HealthScoreBreakdown>,
    pub logo_url: Option<String>,
    pub brand_colors: Value,
    pub website_url: Option<String>,
    pub custom_domain: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub team: Vec<BusinessMemberResponse>,
}



#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BusinessMember {
    pub id: Uuid,
    pub business_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub permissions: Value,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
    pub invitation_accepted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessMemberResponse {
    pub user_id: Uuid,
    pub role: String,
    pub name: String,
    pub email: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Industry {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub ai_prompt_context: Option<String>,
    pub common_business_models: Value,
    pub created_at: DateTime<Utc>,
}

impl From<Business> for BusinessResponse {
    fn from(business: Business) -> Self {
        Self {
            id: business.id,
            name: business.name,
            slug: business.slug,
            tagline: business.tagline,
            description: business.description,
            industry: business.industry,
            sub_industry: business.sub_industry,
            status: business.status,
            stage: business.stage,
            country_code: business.country_code,
            city: business.city,
            health_score: business.health_score,
            health_score_breakdown: None, // Populated by service
            logo_url: business.logo_url,
            brand_colors: business.brand_colors,
            website_url: business.website_url,
            custom_domain: business.custom_domain,
            created_at: business.created_at,
            updated_at: business.updated_at,
            team: vec![], // Populated by service
        }
    }
}

// Checklist models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChecklistCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub country_code: Option<String>,
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub country_code: Option<String>,
    pub order_index: i32,
    pub estimated_duration_minutes: Option<i32>,
    pub required_for_stage: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BusinessChecklistProgress {
    pub id: Uuid,
    pub business_id: Uuid,
    pub checklist_item_id: Uuid,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub notes: Option<String>,
    pub attachments: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistCategoryResponse {
    pub name: String,
    pub progress: i32,
    pub items: Vec<ChecklistItemResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecklistItemResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub completed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChecklistItemRequest {
    pub completed: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusinessChecklistResponse {
    pub overall_progress: i32,
    pub categories: Vec<ChecklistCategoryResponse>,
}
