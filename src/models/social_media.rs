use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================
// SOCIAL MEDIA ACCOUNT MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct SocialMediaAccount {
    pub id: Uuid,
    pub business_id: Uuid,
    pub user_id: Uuid,
    pub platform: String, // instagram, twitter, linkedin, facebook, tiktok
    pub account_handle: Option<String>,
    pub account_url: Option<String>,
    pub status: String,
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub follower_count: Option<i32>,
    pub post_count: Option<i32>,
    pub engagement_rate: Option<f64>,
    pub ai_content_enabled: bool,
    pub content_tone: Option<String>,
    pub posting_schedule: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct ContentCalendarItem {
    pub id: Uuid,
    pub business_id: Uuid,
    pub social_account_id: Option<Uuid>,
    pub content_type: String, // post, story, reel, thread
    pub status: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub ai_generated_content: Value,
    pub media_urls: Value,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub likes: i32,
    pub comments: i32,
    pub shares: i32,
    pub impressions: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// REQUEST/RESPONSE MODELS
// ============================================

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ConnectSocialAccountRequest {
    #[validate(length(min = 1, message = "Platform is required"))]
    pub platform: String,
    pub auth_code: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateContentRequest {
    pub social_account_id: Option<Uuid>,
    #[validate(length(min = 1, message = "Content type is required"))]
    pub content_type: String,
    pub topic: Option<String>,
    pub tone: Option<String>,
    pub ai_generate: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleContentRequest {
    pub scheduled_at: DateTime<Utc>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SocialAccountResponse {
    pub id: Uuid,
    pub platform: String,
    pub account_handle: Option<String>,
    pub account_url: Option<String>,
    pub status: String,
    pub follower_count: Option<i32>,
    pub engagement_rate: Option<f64>,
    pub ai_content_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentCalendarResponse {
    pub id: Uuid,
    pub platform: String,
    pub content_type: String,
    pub status: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub metrics: ContentMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContentMetrics {
    pub likes: i32,
    pub comments: i32,
    pub shares: i32,
    pub impressions: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiGeneratedPost {
    pub content: String,
    pub hashtags: Vec<String>,
    pub suggested_images: Vec<String>,
    pub best_posting_time: String,
    pub predicted_engagement: String,
}

// ============================================
// MARKETPLACE MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceService {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub service_type: String,
    pub title: String,
    pub description: Option<String>,
    pub pricing_type: String,
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub currency: String,
    pub deliverables: Value,
    pub timeline_days: Option<i32>,
    pub rating: Option<f64>,
    pub review_count: i32,
    pub completed_projects: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceBooking {
    pub id: Uuid,
    pub business_id: Uuid,
    pub service_id: Uuid,
    pub requester_id: Uuid,
    pub status: String,
    pub requirements: Option<String>,
    pub agreed_price: Option<i64>,
    pub timeline_days: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateMarketplaceServiceRequest {
    #[validate(length(min = 1, message = "Service type is required"))]
    pub service_type: String,
    #[validate(length(min = 3, message = "Title must be at least 3 characters"))]
    pub title: String,
    pub description: String,
    pub pricing_type: String, // fixed, hourly, package
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub deliverables: Vec<String>,
    pub timeline_days: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookServiceRequest {
    pub service_id: Uuid,
    pub requirements: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceServiceResponse {
    pub id: Uuid,
    pub provider: ServiceProviderInfo,
    pub service_type: String,
    pub title: String,
    pub description: Option<String>,
    pub pricing: PricingInfo,
    pub rating: Option<f64>,
    pub review_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceProviderInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar: Option<String>,
    pub completed_projects: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PricingInfo {
    pub pricing_type: String,
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub currency: String,
}
