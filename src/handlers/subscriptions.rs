use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};

use crate::models::{CreateSubscriptionRequest, UpdateSubscriptionRequest};
use crate::services::subscription_service::SubscriptionService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subscriptions")
            .service(get_plans)
            .service(get_current_subscription)
            .service(create_subscription)
            .service(cancel_subscription)
            .service(get_invoices)
            .service(get_payment_methods),
    );
}

#[get("/plans")]
async fn get_plans(
    subscription_service: web::Data<SubscriptionService>,
) -> HttpResponse {
    match subscription_service.get_plans().await {
        Ok(plans) => ResponseBuilder::ok(plans),
        Err(e) => e.error_response(),
    }
}

#[get("/me")]
async fn get_current_subscription(
    subscription_service: web::Data<SubscriptionService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match subscription_service.get_user_subscription(user_id).await {
        Ok(Some(sub)) => ResponseBuilder::ok(sub),
        Ok(None) => ResponseBuilder::ok(serde_json::json!({
            "plan": "free",
            "status": "active"
        })),
        Err(e) => e.error_response(),
    }
}

#[post("")]
async fn create_subscription(
    req: web::Json<CreateSubscriptionRequest>,
    subscription_service: web::Data<SubscriptionService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match subscription_service.create_subscription(user_id, req.into_inner()).await {
        Ok(sub) => ResponseBuilder::created(sub),
        Err(e) => e.error_response(),
    }
}

#[delete("")]
async fn cancel_subscription(
    query: web::Query<UpdateSubscriptionRequest>,
    subscription_service: web::Data<SubscriptionService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let cancel_at_period_end = query.cancel_at_period_end.unwrap_or(true);

    match subscription_service.cancel_subscription(user_id, cancel_at_period_end).await {
        Ok(_) => ResponseBuilder::ok(serde_json::json!({ 
            "message": "Subscription cancelled" 
        })),
        Err(e) => e.error_response(),
    }
}

#[get("/invoices")]
async fn get_invoices(
    subscription_service: web::Data<SubscriptionService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match subscription_service.get_invoices(user_id).await {
        Ok(invoices) => ResponseBuilder::ok(invoices),
        Err(e) => e.error_response(),
    }
}

#[get("/payment-methods")]
async fn get_payment_methods(
    subscription_service: web::Data<SubscriptionService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match subscription_service.get_payment_methods(user_id).await {
        Ok(methods) => ResponseBuilder::ok(methods),
        Err(e) => e.error_response(),
    }
}
