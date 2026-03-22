use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("External API error: {0}")]
    ExternalApi(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Rate limited: {0}")]
    RateLimited(String),
    
    #[error("Too many requests: {0}")]
    RateLimit(String),
}

impl AppError {
    /// Convert to HTTP response
    pub fn error_response(&self) -> HttpResponse {
        ResponseError::error_response(self)
    }
    
    /// Alias for error_response for convenience
    pub fn into_response(&self) -> HttpResponse {
        self.error_response()
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::ExternalApi(_) => StatusCode::BAD_GATEWAY,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_message = self.to_string();
        
        // Extract the error code from the variant name
        let error_code = match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::ExternalApi(_) => "EXTERNAL_API_ERROR",
            AppError::Conflict(_) => "CONFLICT",
            AppError::RateLimited(_) => "RATE_LIMITED",
            AppError::RateLimit(_) => "RATE_LIMIT",
        };
        
        HttpResponse::build(status).json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": error_message,
            },
            "data": null
        }))
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
