//! Marketplace API Handlers

use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::marketplace::{
    CreateOrderRequest, GenerateAiContentRequest, ListServiceListingsRequest, ScheduleAiContentRequest,
    SendProviderMessageRequest, SubmitReviewRequest, UpdateContentRequest,
};
use crate::models::ApiResponse;
use crate::services::MarketplaceService;
use crate::utils::get_user_id;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/marketplace")
            .service(list_service_listings)
            .service(create_order)
            .service(list_orders)
            .service(submit_review)
            .service(get_messages)
            .service(send_message)
            .service(generate_ai_content)
            .service(get_content_calendar)
            .service(update_content)
            .service(schedule_content)
    );
}

/// GET /api/v1/marketplace/listings
#[get("/listings")]
async fn list_service_listings(
    service: web::Data<MarketplaceService>,
    query: web::Query<ListServiceListingsRequest>,
) -> HttpResponse {
    match service.list_listings(query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/marketplace/orders
#[post("/orders")]
async fn create_order(
    service: web::Data<MarketplaceService>,
    req: web::Json<CreateOrderRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.create_order(user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/marketplace/orders/{business_id}
#[get("/orders/{business_id}")]
async fn list_orders(
    service: web::Data<MarketplaceService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.list_orders(business_id.into_inner()).await {
        Ok(orders) => HttpResponse::Ok().json(ApiResponse::success(orders)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/marketplace/orders/{order_id}/review
#[post("/orders/{order_id}/review")]
async fn submit_review(
    service: web::Data<MarketplaceService>,
    order_id: web::Path<Uuid>,
    req: web::Json<SubmitReviewRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.submit_review(order_id.into_inner(), req.rating, req.review_text.clone()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/marketplace/orders/{order_id}/messages
#[get("/orders/{order_id}/messages")]
async fn get_messages(
    service: web::Data<MarketplaceService>,
    order_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.get_messages(order_id.into_inner()).await {
        Ok(messages) => HttpResponse::Ok().json(ApiResponse::success(messages)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/marketplace/orders/{order_id}/messages
#[post("/orders/{order_id}/messages")]
async fn send_message(
    service: web::Data<MarketplaceService>,
    order_id: web::Path<Uuid>,
    req: web::Json<SendProviderMessageRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.send_message(order_id.into_inner(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/marketplace/ai-content/generate
#[post("/ai-content/generate")]
async fn generate_ai_content(
    service: web::Data<MarketplaceService>,
    req: web::Json<GenerateAiContentRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.generate_content(req.into_inner()).await {
        Ok(response) => HttpResponse::Accepted().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/marketplace/ai-content/{business_id}
#[get("/ai-content/{business_id}")]
async fn get_content_calendar(
    service: web::Data<MarketplaceService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let _user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.get_content_calendar(business_id.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// PUT /api/v1/marketplace/ai-content/{content_id}
#[put("/ai-content/{content_id}")]
async fn update_content(
    service: web::Data<MarketplaceService>,
    content_id: web::Path<Uuid>,
    req: web::Json<UpdateContentRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.update_content(content_id.into_inner(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/marketplace/ai-content/{content_id}/schedule
#[post("/ai-content/{content_id}/schedule")]
async fn schedule_content(
    service: web::Data<MarketplaceService>,
    content_id: web::Path<Uuid>,
    req: web::Json<ScheduleAiContentRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", "Authentication required")),
    };

    match service.schedule_content(content_id.into_inner(), user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}
