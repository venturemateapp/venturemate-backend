use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpMessage, HttpResponse,
};
use std::future::{ready, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::utils::Jwt;

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        
        Box::pin(async move {
            // Extract token from Authorization header
            let token = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            tracing::debug!("AuthMiddleware: token present = {}", token.is_some());

            if let Some(token) = token {
                // Get JWT from app data
                if let Some(jwt) = req.app_data::<actix_web::web::Data<Jwt>>() {
                    tracing::debug!("AuthMiddleware: JWT app_data found");
                    match jwt.extract_user_id(token) {
                        Ok(user_id) => {
                            tracing::debug!("AuthMiddleware: user_id extracted = {}", user_id);
                            // Store user_id in request extensions
                            req.extensions_mut().insert::<Uuid>(user_id);
                            return service.call(req).await;
                        }
                        Err(e) => {
                            tracing::warn!("AuthMiddleware: token validation failed: {} (token: {}...)", e, &token[..20.min(token.len())]);
                            // Return unauthorized response
                            let (req_parts, _) = req.into_parts();
                            let response = HttpResponse::Unauthorized()
                                .json(serde_json::json!({
                                    "success": false,
                                    "error": {
                                        "code": "UNAUTHORIZED",
                                        "message": "Invalid or expired token"
                                    }
                                }));
                            return Ok(ServiceResponse::new(req_parts, response));
                        }
                    }
                } else {
                    tracing::error!("AuthMiddleware: JWT app_data NOT found!");
                }
            } else {
                tracing::debug!("AuthMiddleware: No Bearer token in Authorization header");
            }

            // Return unauthorized response for missing/invalid token
            let (req_parts, _) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "success": false,
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "Missing or invalid authorization token"
                    }
                }));
            Ok(ServiceResponse::new(req_parts, response))
        })
    }
}

/// Extract user_id from request extensions
pub fn get_user_id(req: &actix_web::HttpRequest) -> Option<Uuid> {
    req.extensions().get::<Uuid>().copied()
}
