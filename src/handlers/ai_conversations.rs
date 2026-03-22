use actix_web::{get, post, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::{
    CreateConversationRequest, GenerateContentRequest, RegenerateContentRequest,
    SendMessageRequest, PaginationParams,
};
use crate::services::AiConversationService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ai")
            .service(create_conversation)
            .service(list_conversations)
            .service(get_conversation)
            .service(send_message)
            .service(get_messages)
            .service(generate_content)
            .service(regenerate_content)
            .service(get_generated_content)
            .service(calculate_health_score)
            .service(get_recommendations)
            .service(dismiss_recommendation)
    );
}

// ============================================
// AI CONVERSATIONS
// ============================================

#[post("/conversations")]
async fn create_conversation(
    req: web::Json<CreateConversationRequest>,
    ai_service: web::Data<AiConversationService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.create_conversation(user_id, req.into_inner()).await {
        Ok(conversation) => ResponseBuilder::ok(conversation),
        Err(e) => e.error_response(),
    }
}

#[get("/conversations")]
async fn list_conversations(
    query: web::Query<PaginationParams>,
    ai_service: web::Data<AiConversationService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.list_conversations(user_id, query.page.unwrap_or(1), query.per_page.unwrap_or(20)).await {
        Ok(conversations) => ResponseBuilder::ok(conversations),
        Err(e) => e.error_response(),
    }
}

#[get("/conversations/{id}")]
async fn get_conversation(
    path: web::Path<Uuid>,
    ai_service: web::Data<AiConversationService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.get_conversation(path.into_inner(), user_id).await {
        Ok(conversation) => ResponseBuilder::ok(conversation),
        Err(e) => e.error_response(),
    }
}

#[post("/conversations/{id}/messages")]
async fn send_message(
    path: web::Path<Uuid>,
    req: web::Json<SendMessageRequest>,
    ai_service: web::Data<AiConversationService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.send_message(path.into_inner(), user_id, req.into_inner()).await {
        Ok(response) => ResponseBuilder::ok(response),
        Err(e) => e.error_response(),
    }
}

#[get("/conversations/{id}/messages")]
async fn get_messages(
    path: web::Path<Uuid>,
    query: web::Query<PaginationParams>,
    ai_service: web::Data<AiConversationService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.get_messages(path.into_inner(), user_id, query.page.unwrap_or(1), query.per_page.unwrap_or(50)).await {
        Ok(messages) => ResponseBuilder::ok(messages),
        Err(e) => e.error_response(),
    }
}

// ============================================
// AI CONTENT GENERATION
// ============================================

#[post("/generate")]
async fn generate_content(
    req: web::Json<GenerateContentRequest>,
    ai_service: web::Data<AiConversationService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.generate_content(user_id, req.into_inner()).await {
        Ok(content) => ResponseBuilder::ok(content),
        Err(e) => e.error_response(),
    }
}

#[post("/generate/{id}/regenerate")]
async fn regenerate_content(
    path: web::Path<Uuid>,
    req: web::Json<RegenerateContentRequest>,
    ai_service: web::Data<AiConversationService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.regenerate_content(user_id, path.into_inner(), req.into_inner()).await {
        Ok(content) => ResponseBuilder::ok(content),
        Err(e) => e.error_response(),
    }
}

#[get("/content")]
async fn get_generated_content(
    query: web::Query<std::collections::HashMap<String, String>>,
    ai_service: web::Data<AiConversationService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let business_id = query.get("business_id").and_then(|v| Uuid::parse_str(v).ok());
    let content_type = query.get("content_type").cloned();

    match ai_service.list_generated_content(user_id, business_id, content_type).await {
        Ok(content) => ResponseBuilder::ok(content),
        Err(e) => e.error_response(),
    }
}

// ============================================
// HEALTH SCORE & RECOMMENDATIONS
// ============================================

#[post("/health-score")]
async fn calculate_health_score(
    req: web::Json<crate::models::CalculateHealthScoreRequest>,
    ai_service: web::Data<AiConversationService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    // Get user's business
    match ai_service.calculate_health_score(user_id, req.force_recalculate.unwrap_or(false)).await {
        Ok(score) => ResponseBuilder::ok(score),
        Err(e) => e.error_response(),
    }
}

#[get("/recommendations")]
async fn get_recommendations(
    ai_service: web::Data<AiConversationService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.get_recommendations(user_id).await {
        Ok(recommendations) => ResponseBuilder::ok(recommendations),
        Err(e) => e.error_response(),
    }
}

#[post("/recommendations/{id}/dismiss")]
async fn dismiss_recommendation(
    path: web::Path<Uuid>,
    req: web::Json<crate::models::DismissRecommendationRequest>,
    ai_service: web::Data<AiConversationService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match ai_service.dismiss_recommendation(user_id, path.into_inner(), req.reason.clone().unwrap_or_default()).await {
        Ok(_) => ResponseBuilder::ok(serde_json::json!({"message": "Recommendation dismissed"})),
        Err(e) => e.error_response(),
    }
}
