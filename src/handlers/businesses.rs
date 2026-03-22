use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::models::{
    CreateBusinessRequest, PaginationParams, UpdateBusinessRequest, UpdateChecklistItemRequest,
};
use crate::services::business_service::BusinessService;
use crate::utils::{get_user_id, ResponseBuilder};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/businesses")
            // Register static paths BEFORE dynamic paths like /{id}
            .service(list_businesses)
            .service(create_business)
            .service(list_industries)  // Must be before /{id}
            // Nested resources under /{business_id}
            .service(get_checklist)
            .service(update_checklist_item)
            .service(get_business)
            .service(update_business)
            .service(delete_business)

    );
}

#[get("")]
async fn list_businesses(
    query: web::Query<PaginationParams>,
    business_service: web::Data<BusinessService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match business_service.list_for_user(user_id, page, per_page).await {
        Ok((businesses, total)) => {
            let response = crate::models::PaginatedResponse::new(businesses, total, page, per_page);
            ResponseBuilder::ok(response)
        }
        Err(e) => e.error_response(),
    }
}

#[post("")]
async fn create_business(
    req: web::Json<CreateBusinessRequest>,
    business_service: web::Data<BusinessService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match business_service.create(user_id, req.into_inner()).await {
        Ok(business) => ResponseBuilder::created(business),
        Err(e) => e.error_response(),
    }
}

#[get("/{id}")]
async fn get_business(
    path: web::Path<Uuid>,
    business_service: web::Data<BusinessService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match business_service.get_by_id(path.into_inner(), user_id).await {
        Ok(business) => ResponseBuilder::ok(business),
        Err(e) => e.error_response(),
    }
}

#[put("/{id}")]
async fn update_business(
    path: web::Path<Uuid>,
    req: web::Json<UpdateBusinessRequest>,
    business_service: web::Data<BusinessService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match business_service.update(path.into_inner(), user_id, req.into_inner()).await {
        Ok(business) => ResponseBuilder::ok(business),
        Err(e) => e.error_response(),
    }
}

#[delete("/{id}")]
async fn delete_business(
    path: web::Path<Uuid>,
    business_service: web::Data<BusinessService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match business_service.delete(path.into_inner(), user_id).await {
        Ok(_) => ResponseBuilder::no_content(),
        Err(e) => e.error_response(),
    }
}

#[get("/{id}/checklist")]
async fn get_checklist(
    path: web::Path<Uuid>,
    business_service: web::Data<BusinessService>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    match business_service.get_checklist(path.into_inner(), user_id).await {
        Ok(checklist) => ResponseBuilder::ok(checklist),
        Err(e) => e.error_response(),
    }
}

#[put("/{id}/checklist/{item_id}")]
async fn update_checklist_item(
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateChecklistItemRequest>,
    business_service: web::Data<BusinessService>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => return ResponseBuilder::unauthorized("Not authenticated"),
    };

    let (business_id, item_id) = path.into_inner();

    match business_service.update_checklist_item(business_id, item_id, user_id, req.into_inner()).await {
        Ok(_) => ResponseBuilder::ok(serde_json::json!({ "message": "Checklist item updated" })),
        Err(e) => e.error_response(),
    }
}

#[get("/industries")]
async fn list_industries(
    business_service: web::Data<BusinessService>,
) -> HttpResponse {
    match business_service.list_industries().await {
        Ok(industries) => ResponseBuilder::ok(industries),
        Err(e) => e.error_response(),
    }
}
