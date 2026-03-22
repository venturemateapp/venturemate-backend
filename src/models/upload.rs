use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Upload {
    pub id: Uuid,
    pub business_id: Option<Uuid>,
    pub user_id: Uuid,
    pub original_name: String,
    pub storage_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Value,
    pub visibility: String,
    pub share_token: Option<String>,
    pub share_expires_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub blob_id: Option<Uuid>,
    pub is_blob_stored: bool,
    pub document_type: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UploadFolder {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UploadBrandAsset {
    pub id: Uuid,
    pub business_id: Uuid,
    pub asset_type: String,
    pub file_url: String,
    pub thumbnail_url: Option<String>,
    pub file_size: Option<i64>,
    pub format: Option<String>,
    pub dimensions: Option<Value>,
    pub variant: Option<String>,
    pub ai_job_id: Option<Uuid>,
    pub generation_params: Option<Value>,
    pub is_selected: bool,
    pub selected_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: Value,
    pub blob_id: Option<Uuid>,
    pub thumbnail_blob_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrandColorPalette {
    pub id: Uuid,
    pub business_id: Uuid,
    pub palette: Value,
    pub ai_generated: bool,
    pub ai_job_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// Document Vault Models

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub business_id: Option<Uuid>,
    pub user_id: Uuid,
    pub blob_id: Option<Uuid>,
    pub name: String,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub folder_id: Option<Uuid>,
    pub description: Option<String>,
    pub document_type: Option<String>,
    pub visibility: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FolderResponse {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub document_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentTag {
    pub id: Uuid,
    pub business_id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentShareResponse {
    pub id: Uuid,
    pub upload_id: Uuid,
    pub share_token: String,
    pub allow_download: bool,
    pub allow_preview: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_downloads: Option<i32>,
    pub download_count: i32,
    pub created_at: DateTime<Utc>,
}

// Request/Response structs

#[derive(Debug, Clone, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUploadRequest {
    pub name: Option<String>,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadDocumentRequest {
    pub folder_id: Option<Uuid>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShareDocumentRequest {
    pub expiry_days: Option<i32>,
    pub password: Option<String>,
    pub allow_download: Option<bool>,
    pub allow_preview: Option<bool>,
    pub max_downloads: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDocumentTagRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessShareRequest {
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UploadResponse {
    pub id: Uuid,
    pub original_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub folder_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub visibility: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentListResponse {
    pub documents: Vec<DocumentResponse>,
    pub folders: Vec<FolderResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShareLinkResponse {
    pub share_token: String,
    pub share_url: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<Upload> for UploadResponse {
    fn from(upload: Upload) -> Self {
        Self {
            id: upload.id,
            original_name: upload.original_name,
            file_size: upload.file_size.unwrap_or(0),
            mime_type: upload.mime_type.unwrap_or_default(),
            folder_id: upload.folder_id,
            tags: serde_json::from_value(upload.tags).unwrap_or_default(),
            visibility: upload.visibility,
            created_at: upload.created_at,
        }
    }
}

impl From<UploadFolder> for FolderResponse {
    fn from(folder: UploadFolder) -> Self {
        Self {
            id: folder.id,
            business_id: folder.business_id,
            name: folder.name,
            parent_id: folder.parent_id,
            document_count: 0, // Would need to query separately
            created_at: folder.created_at,
        }
    }
}
