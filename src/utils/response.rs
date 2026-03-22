use actix_web::HttpResponse;
use serde::Serialize;

use crate::models::ApiResponse;

/// Build success responses
pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn ok<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse::success(data))
    }

    pub fn created<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Created().json(ApiResponse::success(data))
    }

    pub fn no_content() -> HttpResponse {
        HttpResponse::NoContent().finish()
    }

    pub fn accepted<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Accepted().json(ApiResponse::success(data))
    }

    pub fn bad_request(message: impl Into<String>) -> HttpResponse {
        HttpResponse::BadRequest().json(ApiResponse::<()>::error("BAD_REQUEST", message))
    }

    pub fn bad_request_with_data<T: Serialize>(message: impl Into<String>, data: T) -> HttpResponse {
        HttpResponse::BadRequest().json(ApiResponse::<T>::error_with_details("BAD_REQUEST", message, serde_json::json!(data)))
    }

    pub fn validation_error(message: impl Into<String>) -> HttpResponse {
        HttpResponse::BadRequest().json(ApiResponse::<()>::error("VALIDATION_ERROR", message))
    }



    pub fn unauthorized(message: impl Into<String>) -> HttpResponse {
        HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", message))
    }

    pub fn forbidden(message: impl Into<String>) -> HttpResponse {
        HttpResponse::Forbidden().json(ApiResponse::<()>::error("FORBIDDEN", message))
    }

    pub fn not_found(message: impl Into<String>) -> HttpResponse {
        HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", message))
    }

    pub fn conflict(message: impl Into<String>) -> HttpResponse {
        HttpResponse::Conflict().json(ApiResponse::<()>::error("CONFLICT", message))
    }

    pub fn internal_error(message: impl Into<String>) -> HttpResponse {
        HttpResponse::InternalServerError().json(ApiResponse::<()>::error("INTERNAL_ERROR", message))
    }
}

/// Create a success response
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(data))
}

/// Create an error response
pub fn error_response(code: &str, message: &str) -> HttpResponse {
    let status_code = match code {
        "BAD_REQUEST" | "VALIDATION_ERROR" => 400,
        "UNAUTHORIZED" => 401,
        "FORBIDDEN" => 403,
        "NOT_FOUND" => 404,
        "CONFLICT" => 409,
        "RATE_LIMITED" => 429,
        _ => 500,
    };

    let response = ApiResponse::<()>::error(code, message);
    
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status_code).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    )
    .json(response)
}
