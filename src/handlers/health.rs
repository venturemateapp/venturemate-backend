use crate::services::{health::HealthService, AppState};
use crate::utils::ResponseBuilder;
use actix_web::{web, HttpResponse};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
}

async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    let service = HealthService::new(state.db.clone());
    
    match service.check().await {
        Ok(status) => ResponseBuilder::ok(status),
        Err(e) => ResponseBuilder::internal_error(e.to_string()),
    }
}
