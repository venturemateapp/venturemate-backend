use crate::db::DbPool;
use crate::config::Config;
use std::sync::Arc;

pub mod ai_conversation_service;
pub mod ai_service;
pub mod ai_startup_engine_service;
pub mod auth_service;
pub mod branding_service;
pub mod business_service;
pub mod cofounder_service;
pub mod dashboard_service;
pub mod data_room_service;
pub mod document_generation_service;
pub mod document_service;
pub mod email_service;
pub mod file_storage_service;
pub mod health;
pub mod health_score_service;
pub mod marketplace_service;
pub mod onboarding_service;
pub mod onboarding_wizard_service;
pub mod recommendations_service;
pub mod startup_stack_service;
pub mod subscription_service;
pub mod user_service;
pub mod website_service;

pub use ai_conversation_service::AiConversationService;
pub use ai_service::AIService;
pub use ai_startup_engine_service::AiStartupEngineService;
pub use auth_service::AuthService;
pub use branding_service::BrandingService;
pub use business_service::BusinessService;
pub use cofounder_service::CofounderService;
pub use dashboard_service::DashboardService;
pub use data_room_service::DataRoomService;
pub use document_generation_service::DocumentGenerationService;
pub use document_service::DocumentService;
pub use email_service::EmailService;
pub use file_storage_service::FileStorageService;
pub use health::HealthService;
pub use health_score_service::HealthScoreService;
pub use marketplace_service::MarketplaceService;
pub use onboarding_service::OnboardingService;
pub use onboarding_wizard_service::OnboardingWizardService;
pub use recommendations_service::RecommendationsService;
pub use startup_stack_service::StartupStackService;
pub use subscription_service::SubscriptionService;
pub use user_service::UserService;
pub use website_service::WebsiteService;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: DbPool, config: Config) -> Self {
        Self {
            db,
            config: Arc::new(config),
        }
    }
}
