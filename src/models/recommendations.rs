//! Smart Recommendations Engine Models
//!
//! Models for AI-powered personalized recommendations
//! that guide founders toward startup success.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// Recommendation
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Recommendation {
    pub id: Uuid,
    pub business_id: Uuid,
    pub recommendation_type: String,
    pub trigger_source: String,
    pub title: String,
    pub description: String,
    pub impact_description: Option<String>,
    pub cta_text: Option<String>,
    pub cta_link: Option<String>,
    pub action_type: Option<String>,
    pub priority: String,
    pub status: String,
    pub priority_score: i32,
    pub has_financial_impact: bool,
    pub unblocks_features: bool,
    pub is_time_sensitive: bool,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub acted_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub id: Uuid,
    pub recommendation_type: String,
    pub title: String,
    pub description: String,
    pub impact_description: Option<String>,
    pub cta_text: Option<String>,
    pub cta_link: Option<String>,
    pub priority: String,
    pub priority_label: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Recommendation Action Log
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RecommendationAction {
    pub id: Uuid,
    pub recommendation_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Recommendation Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationType {
    Compliance,
    Revenue,
    MarketFit,
    Team,
    Operations,
    Timing,
    Behavioral,
}

impl RecommendationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compliance => "compliance",
            Self::Revenue => "revenue",
            Self::MarketFit => "market_fit",
            Self::Team => "team",
            Self::Operations => "operations",
            Self::Timing => "timing",
            Self::Behavioral => "behavioral",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Compliance => "Compliance",
            Self::Revenue => "Revenue",
            Self::MarketFit => "Market Fit",
            Self::Team => "Team",
            Self::Operations => "Operations",
            Self::Timing => "Timing",
            Self::Behavioral => "Behavioral",
        }
    }
}

// =============================================================================
// Priority
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationPriority {
    High,
    Medium,
    Low,
}

impl RecommendationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::High => "🔴",
            Self::Medium => "🟡",
            Self::Low => "🟢",
        }
    }
}

// =============================================================================
// Status
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationStatus {
    Pending,
    Acted,
    Dismissed,
    Expired,
}

impl RecommendationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Acted => "acted",
            Self::Dismissed => "dismissed",
            Self::Expired => "expired",
        }
    }
}

// =============================================================================
// Requests & Responses
// =============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ListRecommendationsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListRecommendationsResponse {
    pub recommendations: Vec<RecommendationResponse>,
    pub dismissed_count: i64,
    pub total_pending: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DismissRecommendationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DismissRecommendationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActOnRecommendationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActOnRecommendationResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefreshRecommendationsResponse {
    pub new_recommendations_count: i64,
    pub message: String,
}

// =============================================================================
// Trigger Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationTrigger {
    pub trigger_type: String,
    pub condition: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum TriggerCondition {
    Timing { days_since: i32, milestone: String },
    HealthScore { component: String, threshold: i32, operator: String },
    Component { component: String, score: i32 },
    Behavioral { action: String, count: i32 },
}

// =============================================================================
// AI Content Generation for Recommendations
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationContentRequest {
    pub startup_name: String,
    pub industry: String,
    pub business_stage: String,
    pub trigger_type: String,
    pub trigger_context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationContent {
    pub title: String,
    pub description: String,
    pub impact: String,
    pub call_to_action: String,
}

// =============================================================================
// Recommendation Templates
// =============================================================================

#[derive(Debug, Clone)]
pub struct RecommendationTemplate {
    pub trigger_condition: TriggerCondition,
    pub title_template: String,
    pub description_template: String,
    pub cta_text: String,
    pub cta_link: String,
    pub priority: RecommendationPriority,
    pub recommendation_type: RecommendationType,
}

// Pre-defined templates for common recommendations
pub fn get_recommendation_templates() -> Vec<RecommendationTemplate> {
    vec![
        // Compliance recommendations
        RecommendationTemplate {
            trigger_condition: TriggerCondition::Timing { days_since: 7, milestone: "startup_created".to_string() },
            title_template: "Start your business registration".to_string(),
            description_template: "It's been a week since you started. Business registration is the foundation for everything else—bank accounts, contracts, and credibility.".to_string(),
            cta_text: "Start Registration".to_string(),
            cta_link: "/compliance/registration".to_string(),
            priority: RecommendationPriority::High,
            recommendation_type: RecommendationType::Compliance,
        },
        RecommendationTemplate {
            trigger_condition: TriggerCondition::HealthScore { component: "compliance".to_string(), threshold: 50, operator: "<".to_string() },
            title_template: "Complete your compliance requirements".to_string(),
            description_template: "Your compliance score is below 50. Address the missing items to avoid legal issues and unlock funding opportunities.".to_string(),
            cta_text: "View Checklist".to_string(),
            cta_link: "/compliance/checklist".to_string(),
            priority: RecommendationPriority::High,
            recommendation_type: RecommendationType::Compliance,
        },
        // Revenue recommendations
        RecommendationTemplate {
            trigger_condition: TriggerCondition::Timing { days_since: 7, milestone: "bank_connected".to_string() },
            title_template: "Set up payment processing".to_string(),
            description_template: "You have a bank account connected. Now add a payment gateway to start accepting payments from customers.".to_string(),
            cta_text: "Set Up Payments".to_string(),
            cta_link: "/finance/payment-gateway".to_string(),
            priority: RecommendationPriority::Medium,
            recommendation_type: RecommendationType::Revenue,
        },
        RecommendationTemplate {
            trigger_condition: TriggerCondition::HealthScore { component: "revenue".to_string(), threshold: 30, operator: "<".to_string() },
            title_template: "Start monetizing your business".to_string(),
            description_template: "Your revenue setup is incomplete. Based on your industry, here are 3 ways to start generating revenue now.".to_string(),
            cta_text: "View Options".to_string(),
            cta_link: "/revenue/models".to_string(),
            priority: RecommendationPriority::High,
            recommendation_type: RecommendationType::Revenue,
        },
        // Market fit recommendations
        RecommendationTemplate {
            trigger_condition: TriggerCondition::Timing { days_since: 14, milestone: "website_published".to_string() },
            title_template: "Add analytics to your website".to_string(),
            description_template: "Your website has been live for 2 weeks. Add Google Analytics to track visitors and understand your audience.".to_string(),
            cta_text: "Add Analytics".to_string(),
            cta_link: "/website/analytics".to_string(),
            priority: RecommendationPriority::Medium,
            recommendation_type: RecommendationType::MarketFit,
        },
        RecommendationTemplate {
            trigger_condition: TriggerCondition::HealthScore { component: "market_fit".to_string(), threshold: 40, operator: "<".to_string() },
            title_template: "Improve your website messaging".to_string(),
            description_template: "Your market fit score indicates your website could better communicate your value proposition. Here are specific improvements.".to_string(),
            cta_text: "View Analysis".to_string(),
            cta_link: "/market-fit/website-review".to_string(),
            priority: RecommendationPriority::Medium,
            recommendation_type: RecommendationType::MarketFit,
        },
        // Team recommendations
        RecommendationTemplate {
            trigger_condition: TriggerCondition::Component { component: "team".to_string(), score: 60 },
            title_template: "Complete your team profiles".to_string(),
            description_template: "Investors want to know who's behind the business. Complete your team profiles to increase credibility.".to_string(),
            cta_text: "Update Team".to_string(),
            cta_link: "/team/profiles".to_string(),
            priority: RecommendationPriority::Medium,
            recommendation_type: RecommendationType::Team,
        },
        // Operations recommendations
        RecommendationTemplate {
            trigger_condition: TriggerCondition::Timing { days_since: 30, milestone: "business_plan_generated".to_string() },
            title_template: "Update your business plan".to_string(),
            description_template: "Your business plan is 30 days old. Your business has likely evolved—update it with latest metrics and learnings.".to_string(),
            cta_text: "Update Plan".to_string(),
            cta_link: "/documents/business-plan".to_string(),
            priority: RecommendationPriority::Low,
            recommendation_type: RecommendationType::Operations,
        },
    ]
}

// =============================================================================
// Metrics
// =============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationsMetrics {
    pub total_recommendations: i64,
    pub pending_count: i64,
    pub acted_count: i64,
    pub dismissed_count: i64,
    pub click_through_rate: f32,
    pub action_completion_rate: f32,
    pub average_time_to_action_hours: f32,
}

// =============================================================================
// AI Conversation Service Types (needed for compatibility)
// =============================================================================

/// SmartRecommendation - alias for Recommendation for compatibility
pub type SmartRecommendation = Recommendation;

/// RecommendationContext - context for recommendation display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationContext {
    pub business_stage: String,
    pub industry: String,
    pub trigger: String,
}


