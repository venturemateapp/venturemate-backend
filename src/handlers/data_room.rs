//! Data Room API Handlers
//! 
//! API endpoints for secure investor data rooms.

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use tracing::{error, info};
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::models::documents::{AccessDataRoomRequest, AddDataRoomFileRequest, CreateDataRoomRequest, DataRoomAccessResponse, ShareDataRoomRequest};
use crate::services::DataRoomService;
use crate::utils::get_user_id;

/// Configure data room routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/data-rooms")
            // Owner endpoints
            .service(create_data_room)
            .service(get_data_room)
            .service(list_data_rooms)
            .service(update_data_room)
            .service(delete_data_room)
            .service(share_data_room)
            .service(get_data_room_access_logs)
            .service(add_file_to_data_room)
            .service(delete_file_from_data_room)
            // Public/investor endpoints
            .service(access_shared_data_room)
            .service(download_data_room_file)
    );
}

// =============================================================================
// Owner Endpoints
// =============================================================================

/// POST /api/v1/data-rooms - Create data room
#[post("")]
async fn create_data_room(
    service: web::Data<DataRoomService>,
    req: web::Json<CreateDataRoomRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    info!("Creating data room for user: {}", user_id);

    match service.create_data_room(user_id, req.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(ApiResponse::success(response)),
        Err(e) => {
            error!("Failed to create data room: {}", e);
            e.into_response()
        }
    }
}

/// GET /api/v1/data-rooms/{data_room_id} - Get data room details
#[get("/{data_room_id}")]
async fn get_data_room(
    service: web::Data<DataRoomService>,
    data_room_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.get_data_room(user_id, data_room_id.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/data-rooms/business/{business_id} - List business data rooms
#[get("/business/{business_id}")]
async fn list_data_rooms(
    service: web::Data<DataRoomService>,
    business_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.list_data_rooms(user_id, business_id.into_inner()).await {
        Ok(rooms) => HttpResponse::Ok().json(ApiResponse::success(rooms)),
        Err(e) => e.into_response(),
    }
}

/// PUT /api/v1/data-rooms/{data_room_id} - Update data room
#[put("/{data_room_id}")]
async fn update_data_room(
    service: web::Data<DataRoomService>,
    data_room_id: web::Path<Uuid>,
    req: web::Json<UpdateDataRoomRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.update_data_room(
        user_id,
        data_room_id.into_inner(),
        req.name.clone(),
        req.description.clone(),
        req.is_active,
    ).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// DELETE /api/v1/data-rooms/{data_room_id} - Delete data room
#[delete("/{data_room_id}")]
async fn delete_data_room(
    service: web::Data<DataRoomService>,
    data_room_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.delete_data_room(user_id, data_room_id.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/data-rooms/{data_room_id}/share - Share data room
#[post("/{data_room_id}/share")]
async fn share_data_room(
    service: web::Data<DataRoomService>,
    data_room_id: web::Path<Uuid>,
    req: web::Json<ShareDataRoomRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    info!("Sharing data room: {}", data_room_id);

    match service.share_data_room(user_id, data_room_id.into_inner(), req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/data-rooms/{data_room_id}/access-logs - Get access logs
#[get("/{data_room_id}/access-logs")]
async fn get_data_room_access_logs(
    service: web::Data<DataRoomService>,
    data_room_id: web::Path<Uuid>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.get_access_logs(user_id, data_room_id.into_inner()).await {
        Ok(logs) => HttpResponse::Ok().json(ApiResponse::success(logs)),
        Err(e) => e.into_response(),
    }
}

/// POST /api/v1/data-rooms/{data_room_id}/files - Add file to data room
#[post("/{data_room_id}/files")]
async fn add_file_to_data_room(
    service: web::Data<DataRoomService>,
    data_room_id: web::Path<Uuid>,
    req: web::Json<AddDataRoomFileRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    match service.add_file(user_id, data_room_id.into_inner(), req.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// DELETE /api/v1/data-rooms/{data_room_id}/files/{file_id} - Delete file
#[delete("/{data_room_id}/files/{file_id}")]
async fn delete_file_from_data_room(
    service: web::Data<DataRoomService>,
    path: web::Path<(Uuid, Uuid)>,
    http_req: HttpRequest,
) -> HttpResponse {
    let user_id = match get_user_id(&http_req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "UNAUTHORIZED",
                "Authentication required",
            ))
        }
    };

    let (data_room_id, file_id) = path.into_inner();
    
    match service.delete_file(user_id, data_room_id, file_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.into_response(),
    }
}

// =============================================================================
// Public/Investor Endpoints
// =============================================================================

/// POST /api/v1/data-rooms/access/{share_token} - Access shared data room
#[post("/access/{share_token}")]
async fn access_shared_data_room(
    service: web::Data<DataRoomService>,
    share_token: web::Path<String>,
    req: web::Json<AccessDataRoomRequest>,
    http_req: HttpRequest,
) -> HttpResponse {
    let ip_addr = http_req.peer_addr().map(|addr| addr.ip());
    let user_agent = http_req.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok());

    match service.access_data_room(
        &share_token.into_inner(),
        req.into_inner(),
        ip_addr,
        user_agent,
    ).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

/// GET /api/v1/data-rooms/{data_room_id}/files/{file_id}/download - Download file
#[get("/{data_room_id}/files/{file_id}/download")]
async fn download_data_room_file(
    service: web::Data<DataRoomService>,
    path: web::Path<(Uuid, Uuid)>,
    http_req: HttpRequest,
) -> HttpResponse {
    // Check for optional auth (for owner access)
    let user_id = get_user_id(&http_req);
    let (data_room_id, file_id) = path.into_inner();

    match service.download_file(user_id, data_room_id, file_id).await {
        Ok((filename, content)) => {
            let mime_type = match filename.split('.').last() {
                Some("pdf") => "application/pdf",
                Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                _ => "application/octet-stream",
            };

            HttpResponse::Ok()
                .content_type(mime_type)
                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .body(content)
        }
        Err(e) => e.into_response(),
    }
}

// =============================================================================
// Request Types
// =============================================================================

#[derive(Debug, serde::Deserialize)]
struct UpdateDataRoomRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_active: Option<bool>,
}
