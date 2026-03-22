pub mod ai_conversation;
pub mod ai_generation;
pub mod ai_startup_engine;
pub mod auth;
pub mod banking;
pub mod branding;
pub mod business;
pub mod business_idea;
pub mod cofounder;
pub mod credit_score;
pub mod crm;
pub mod document;
pub mod documents;
pub mod health_score;
pub mod investor;
pub mod marketplace;
pub mod onboarding;
pub mod onboarding_wizard;
pub mod project;
pub mod recommendations;
pub mod social_media;
pub mod startup_stack;
pub mod subscription;
pub mod upload;
pub mod user;
pub mod website;

// Re-exports for convenience - some modules have conflicting names
// Use explicit imports: `use crate::models::branding::TypeName`
pub use ai_conversation::*;
// pub use ai_generation::*;  // Has conflicts with documents
pub use ai_startup_engine::*;
pub use auth::*;
pub use banking::*;
// pub use branding::*;  // Use explicit imports
pub use business::*;
pub use business_idea::*;
pub use cofounder::*;
pub use credit_score::*;
pub use crm::*;
pub use document::*;
// pub use documents::*;  // Has conflicts with ai_generation and investor
pub use health_score::*;
pub use investor::*;
pub use marketplace::*;
pub use onboarding::*;
pub use onboarding_wizard::*;
pub use project::*;
pub use recommendations::*;
pub use social_media::*;

// Explicit type aliases to resolve ambiguities
pub use ai_conversation::SendMessageRequest;
pub use ai_conversation::ChatMessageResponse;
pub use marketplace::MarketplaceMessageResponse;
pub use marketplace::SendProviderMessageRequest;
pub use startup_stack::*;
pub use subscription::*;
pub use upload::*;
pub use user::*;
pub use website::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Common fields that most database entities have
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BaseEntity {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination parameters for list endpoints
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        let per_page = self.per_page.unwrap_or(20).max(1).min(100);
        (page - 1) * per_page
    }

    pub fn limit(&self) -> i64 {
        self.per_page.unwrap_or(20).max(1).min(100)
    }
}

/// Generic paginated response
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            data,
            meta: PaginationMeta {
                total,
                page,
                per_page,
                total_pages,
            },
        }
    }
}

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
        }
    }

    pub fn error_with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
                details: Some(details),
            }),
        }
    }
}
