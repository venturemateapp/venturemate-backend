use actix_web::web;
use crate::middleware::auth::AuthMiddleware;

pub mod ai_conversations;
pub mod ai_generation;
pub mod ai_startup_engine;
pub mod auth;
pub mod banking;
pub mod branding;
pub mod businesses;
pub mod cofounder;
pub mod credit;
pub mod crm;
pub mod dashboard;
pub mod data_room;
pub mod documents;
pub mod health;
pub mod health_score;
pub mod investors;
pub mod marketplace;
pub mod onboarding;
pub mod onboarding_wizard;
pub mod recommendations;
pub mod social;
pub mod startup_stack;
pub mod subscriptions;
pub mod users;
pub mod websites;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Build all protected routes first as a separate config
    let protected = web::scope("")
        .wrap(AuthMiddleware)
        .configure(users::configure)
        .configure(onboarding::configure)
        .configure(onboarding_wizard::configure)
        .configure(businesses::configure)
        .configure(ai_generation::configure)
        .configure(ai_conversations::configure)
        .configure(ai_startup_engine::configure)
        .configure(branding::configure)
        .configure(documents::configure)
        .configure(data_room::configure)
        .configure(health_score::configure)
        .configure(recommendations::configure)
        .configure(marketplace::configure)
        .configure(cofounder::configure)
        .configure(subscriptions::configure)
        .configure(websites::configure)
        .configure(crm::configure)
        .configure(banking::configure)
        .configure(investors::configure)
        .configure(credit::configure)
        .configure(social::configure)
        .configure(startup_stack::configure)
        .service(dashboard::get_dashboard)
        .service(dashboard::get_quick_actions)
        .service(dashboard::get_activity_feed);
    
    // Single /api/v1 scope with both public and protected
    cfg.service(
        web::scope("/api/v1")
            // Public routes first
            .configure(health::routes)
            .configure(auth::configure)
            // Protected routes (with middleware)
            .service(protected)
    );
}
