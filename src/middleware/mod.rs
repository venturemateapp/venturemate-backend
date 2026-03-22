use actix_web::{dev::ServiceRequest, Error};

pub mod auth;

/// Request logging middleware setup
pub fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

/// Validator function for actix-web-httpauth
pub async fn validator(
    req: ServiceRequest,
    credentials: actix_web_httpauth::extractors::bearer::BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // TODO: Implement JWT validation
    let _token = credentials.token();
    Ok(req)
}
