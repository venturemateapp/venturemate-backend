use actix_cors::Cors;
use actix_web::{http, middleware as actix_middleware, web, App, HttpServer};
use bcknd::{
    config::Config,
    db::init_db,
    handlers,
    middleware,
    services::{
        ai_conversation_service::AiConversationService,
        ai_service::AIService,
        ai_startup_engine_service::AiStartupEngineService,
        auth_service::AuthService,
        branding_service::BrandingService,
        business_service::BusinessService,
        cofounder_service::CofounderService,
        data_room_service::DataRoomService,
        document_generation_service::DocumentGenerationService,
        document_service::DocumentService,
        email_service::EmailService,
        file_storage_service::FileStorageService,
        health_score_service::HealthScoreService,
        marketplace_service::MarketplaceService,
        onboarding_service::OnboardingService,
        onboarding_wizard_service::OnboardingWizardService,
        recommendations_service::RecommendationsService,
        startup_stack_service::StartupStackService,
        subscription_service::SubscriptionService,
        user_service::UserService,
        website_service::WebsiteService,
        AppState,
    },
    utils::Jwt,
};
use tracing::info;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    middleware::setup_logging();

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize database connection
    let db_pool = init_db(&config.database_url).await?;
    info!("Database connected successfully");

    // Create application state
    let app_state = web::Data::new(AppState::new(db_pool.clone(), config.clone()));

    // Create services
    let jwt = web::Data::new(Jwt::new(&config.jwt_secret));
    
    // Initialize email service
    let email_service = EmailService::new(db_pool.clone()).await;
    
    // Create auth service with email service
    let auth_service = web::Data::new(
        AuthService::new(
            db_pool.clone(),
            &config.jwt_secret,
            &config.google_client_id.clone().unwrap_or_default(),
            &config.google_client_secret.clone().unwrap_or_default(),
            &config.frontend_url.clone().unwrap_or_else(|| "http://localhost:5173".to_string()),
        ).with_email_service(email_service)
    );
    let user_service = web::Data::new(UserService::new(db_pool.clone()));
    let onboarding_service = web::Data::new(OnboardingService::new(db_pool.clone()));
    let onboarding_wizard_service = web::Data::new(OnboardingWizardService::new(db_pool.clone()));
    let business_service = web::Data::new(BusinessService::new(db_pool.clone()));
    let ai_service = web::Data::new(AIService::new(
        &config.anthropic_api_key,
        db_pool.clone(),
    ));
    let startup_stack_service = web::Data::new(StartupStackService::new(db_pool.clone()));
    let subscription_service = web::Data::new(SubscriptionService::new(
        db_pool.clone(),
        &config.stripe_secret_key.clone().unwrap_or_default(),
    ));
    let document_service = web::Data::new(DocumentService::new(db_pool.clone()));
    let website_service = web::Data::new(WebsiteService::new(db_pool.clone()));
    let file_storage_service = web::Data::new(FileStorageService::new(db_pool.clone()));
    let ai_conversation_service = web::Data::new(AiConversationService::new(db_pool.clone()));
    let ai_startup_engine_service = web::Data::new(AiStartupEngineService::new(
        db_pool.clone(),
        AIService::new(&config.anthropic_api_key, db_pool.clone()),
    ));
    let cofounder_service = web::Data::new(CofounderService::new(db_pool.clone()));
    let branding_service = web::Data::new(BrandingService::new(
        db_pool.clone(),
        std::sync::Arc::new(AIService::new(&config.anthropic_api_key, db_pool.clone())),
    ));
    let document_generation_service = web::Data::new(DocumentGenerationService::new(
        db_pool.clone(),
        std::sync::Arc::new(AIService::new(&config.anthropic_api_key, db_pool.clone())),
    ));
    let data_room_service = web::Data::new(DataRoomService::new(db_pool.clone()));
    let health_score_service = web::Data::new(HealthScoreService::new(
        db_pool.clone(),
        std::sync::Arc::new(AIService::new(&config.anthropic_api_key, db_pool.clone())),
    ));
    let recommendations_service = web::Data::new(RecommendationsService::new(
        db_pool.clone(),
        std::sync::Arc::new(AIService::new(&config.anthropic_api_key, db_pool.clone())),
    ));
    let marketplace_service = web::Data::new(MarketplaceService::new(
        db_pool.clone(),
        std::sync::Arc::new(AIService::new(&config.anthropic_api_key, db_pool.clone())),
    ));

    // Get server address
    let server_addr = config.server_addr();
    info!("Starting server at http://{}", server_addr);

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .app_data(jwt.clone())
            .app_data(auth_service.clone())
            .app_data(user_service.clone())
            .app_data(onboarding_service.clone())
            .app_data(onboarding_wizard_service.clone())
            .app_data(business_service.clone())
            .app_data(ai_service.clone())
            .app_data(startup_stack_service.clone())
            .app_data(subscription_service.clone())
            .app_data(document_service.clone())
            .app_data(website_service.clone())
            .app_data(file_storage_service.clone())
            .app_data(ai_conversation_service.clone())
            .app_data(ai_startup_engine_service.clone())
            .app_data(cofounder_service.clone())
            .app_data(branding_service.clone())
            .app_data(document_generation_service.clone())
            .app_data(data_room_service.clone())
            .app_data(health_score_service.clone())
            .app_data(recommendations_service.clone())
            .app_data(marketplace_service.clone())
            .wrap(cors)
            .wrap(actix_middleware::Logger::default())
            .wrap(actix_middleware::Compress::default())
            .configure(handlers::configure_routes)
    })
    .bind(&server_addr)?
    .run()
    .await?;

    Ok(())
}
