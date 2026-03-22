//! Media Marketplace Models
//!
//! Models for the freelancer marketplace and AI content generation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Service Listings
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceListing {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub service_category: String,
    pub service_name: String,
    pub description: Option<String>,
    pub pricing: Option<serde_json::Value>,
    pub delivery_time_days: Option<i32>,
    pub portfolio_urls: Option<serde_json::Value>,
    pub rating: Option<rust_decimal::Decimal>,
    pub review_count: i32,
    pub status: String,
    pub is_verified: bool,
    pub featured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceListingResponse {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub provider_avatar: Option<String>,
    pub service_category: String,
    pub service_name: String,
    pub description: String,
    pub pricing: ServicePricingInfo,
    pub delivery_time_days: i32,
    pub portfolio_urls: Vec<String>,
    pub rating: f32,
    pub review_count: i32,
    pub is_verified: bool,
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePricingInfo {
    pub base_price: rust_decimal::Decimal,
    pub currency: String,
    pub price_tiers: Vec<PriceTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTier {
    pub name: String,
    pub price: rust_decimal::Decimal,
    pub description: String,
    pub delivery_days: i32,
}

// =============================================================================
// Marketplace Orders
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceOrder {
    pub id: Uuid,
    pub business_id: Uuid,
    pub service_id: Uuid,
    pub buyer_id: Uuid,
    pub provider_id: Uuid,
    pub requirements: Option<String>,
    pub attachments: Option<serde_json::Value>,
    pub total_amount: rust_decimal::Decimal,
    pub currency: String,
    pub status: String,
    pub delivery_date: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub business_id: Uuid,
    pub service: OrderServiceInfo,
    pub buyer_id: Uuid,
    pub provider_id: Uuid,
    pub requirements: String,
    pub total_amount: rust_decimal::Decimal,
    pub currency: String,
    pub status: String,
    pub delivery_date: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderServiceInfo {
    pub id: Uuid,
    pub name: String,
    pub category: String,
    pub provider_name: String,
}

// =============================================================================
// Marketplace Reviews
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceReview {
    pub id: Uuid,
    pub order_id: Uuid,
    pub business_id: Uuid,
    pub provider_id: Uuid,
    pub rating: i32,
    pub review_text: Option<String>,
    pub is_public: bool,
    pub response_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub rating: i32,
    pub review_text: Option<String>,
    pub is_public: bool,
    pub response_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Marketplace Messages
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketplaceMessage {
    pub id: Uuid,
    pub order_id: Uuid,
    pub sender_id: Uuid,
    pub message: String,
    pub attachment_url: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceMessageResponse {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_name: String,
    pub message: String,
    pub attachment_url: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// AI Content Generation
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiContent {
    pub id: Uuid,
    pub business_id: Uuid,
    pub content_type: String,
    pub platform: Option<String>,
    pub generated_content: String,
    pub image_url: Option<String>,
    pub hashtags: Option<serde_json::Value>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub status: String,
    pub generation_params: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContentResponse {
    pub id: Uuid,
    pub content_type: String,
    pub platform: Option<String>,
    pub generated_content: String,
    pub image_url: Option<String>,
    pub hashtags: Vec<String>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Service Categories
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceCategory {
    LogoDesign,
    SocialMedia,
    AdManagement,
    Copywriting,
    WebDesign,
    VideoProduction,
    BusinessPlan,
    PitchDeck,
}

impl ServiceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LogoDesign => "logo_design",
            Self::SocialMedia => "social_media",
            Self::AdManagement => "ad_management",
            Self::Copywriting => "copywriting",
            Self::WebDesign => "web_design",
            Self::VideoProduction => "video_production",
            Self::BusinessPlan => "business_plan",
            Self::PitchDeck => "pitch_deck",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::LogoDesign => "Logo Design",
            Self::SocialMedia => "Social Media",
            Self::AdManagement => "Ad Management",
            Self::Copywriting => "Copywriting",
            Self::WebDesign => "Web Design",
            Self::VideoProduction => "Video Production",
            Self::BusinessPlan => "Business Plan",
            Self::PitchDeck => "Pitch Deck",
        }
    }

    pub fn price_range(&self) -> &'static str {
        match self {
            Self::LogoDesign => "$50-500",
            Self::SocialMedia => "$100-1000/month",
            Self::AdManagement => "$200-2000/month",
            Self::Copywriting => "$100-500",
            Self::WebDesign => "$200-2000",
            Self::VideoProduction => "$300-3000",
            Self::BusinessPlan => "$150-1000",
            Self::PitchDeck => "$150-1000",
        }
    }
}

impl std::str::FromStr for ServiceCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "logo_design" => Ok(Self::LogoDesign),
            "social_media" => Ok(Self::SocialMedia),
            "ad_management" => Ok(Self::AdManagement),
            "copywriting" => Ok(Self::Copywriting),
            "web_design" => Ok(Self::WebDesign),
            "video_production" => Ok(Self::VideoProduction),
            "business_plan" => Ok(Self::BusinessPlan),
            "pitch_deck" => Ok(Self::PitchDeck),
            _ => Err(format!("Unknown service category: {}", s)),
        }
    }
}

// =============================================================================
// Order Status
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    InProgress,
    Delivered,
    Completed,
    Cancelled,
    Disputed,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Delivered => "delivered",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Disputed => "disputed",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Delivered => "Delivered",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Disputed => "Disputed",
        }
    }
}

// =============================================================================
// AI Content Status
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiContentStatus {
    Draft,
    Approved,
    Scheduled,
    Published,
}

impl AiContentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Approved => "approved",
            Self::Scheduled => "scheduled",
            Self::Published => "published",
        }
    }
}

// =============================================================================
// Requests & Responses
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ListServiceListingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_rating: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<rust_decimal::Decimal>,
    #[serde(default = "default_featured")]
    pub featured_only: bool,
}

fn default_featured() -> bool {
    false
}

#[derive(Debug, Clone, Serialize)]
pub struct ListServiceListingsResponse {
    pub listings: Vec<ServiceListingResponse>,
    pub categories: Vec<CategoryInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CategoryInfo {
    pub code: String,
    pub name: String,
    pub price_range: String,
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub service_id: Uuid,
    pub requirements: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderResponse {
    pub order_id: Uuid,
    pub status: String,
    pub total_amount: rust_decimal::Decimal,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitReviewRequest {
    pub rating: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendProviderMessageRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment_url: Option<String>,
}



#[derive(Debug, Clone, Deserialize)]
pub struct GenerateAiContentRequest {
    pub business_id: Uuid,
    pub content_type: String,
    #[serde(default = "default_days")]
    pub days: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

fn default_days() -> i32 {
    30
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateAiContentResponse {
    pub generation_id: Uuid,
    pub status: String,
    pub estimated_seconds: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiContentCalendarResponse {
    pub content_calendar: Vec<AiContentCalendarItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiContentCalendarItem {
    pub id: Uuid,
    pub date: String,
    pub platform: String,
    pub content_type: String,
    pub content: String,
    pub hashtags: Vec<String>,
    pub image_url: Option<String>,
    pub status: String,
    pub scheduled_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateContentRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashtags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleAiContentRequest {
    pub scheduled_date: DateTime<Utc>,
}

// =============================================================================
// Social Content Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    SocialPost,
    AdCopy,
    BlogPost,
    EmailCopy,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SocialPost => "social_post",
            Self::AdCopy => "ad_copy",
            Self::BlogPost => "blog_post",
            Self::EmailCopy => "email_copy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialPlatform {
    Instagram,
    Twitter,
    LinkedIn,
    Facebook,
}

impl SocialPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Instagram => "instagram",
            Self::Twitter => "twitter",
            Self::LinkedIn => "linkedin",
            Self::Facebook => "facebook",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Instagram => "Instagram",
            Self::Twitter => "Twitter",
            Self::LinkedIn => "LinkedIn",
            Self::Facebook => "Facebook",
        }
    }
}

// =============================================================================
// AI Content Generation Prompts
// =============================================================================

pub fn build_social_content_prompt(
    business_name: &str,
    industry: &str,
    tagline: &str,
    target_audience: &str,
    products: &str,
    days: i32,
) -> String {
    format!(
        r#"Generate {} days of social media content for a startup.

Business: {}
Industry: {}
Tagline: {}
Target Audience: {}
Key Products/Services: {}

Generate content for:
- Instagram (visual-focused, hashtags)
- Twitter/X (short, engaging)
- LinkedIn (professional, thought leadership)
- Facebook (community-focused)

For each day, provide:
1. Platform
2. Post type (educational, promotional, engagement, behind_the_scenes)
3. Caption text (with emojis)
4. Hashtags (5-10 relevant)
5. Image description (what to visually show)
6. Optimal posting time

Mix content types across the {} days:
- 40% Educational/Value (tips, insights, how-to)
- 30% Engagement (questions, polls, user-generated)
- 20% Promotional (product features, offers)
- 10% Behind-the-scenes/Culture

Return as JSON array with {} entries."#,
        days, business_name, industry, tagline, target_audience, products, days, days
    )
}

pub fn build_ad_copy_prompt(
    business_name: &str,
    industry: &str,
    target_audience: &str,
    product_benefits: &str,
    call_to_action: &str,
) -> String {
    format!(
        r#"Generate ad copy for a startup.

Business: {}
Industry: {}
Target Audience: {}
Product Benefits: {}
Call to Action: {}

Generate:
1. Headline variations (5 options, max 30 chars each)
2. Primary text (3 variations, max 125 chars each)
3. Description (2 variations, max 30 chars each)
4. Call-to-action button text (3 options)

Make copy compelling, benefit-focused, and action-oriented."#,
        business_name, industry, target_audience, product_benefits, call_to_action
    )
}
