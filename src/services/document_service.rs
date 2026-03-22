use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::models::upload::{
    DocumentResponse, FolderResponse, ShareDocumentRequest,
    DocumentShareResponse, DocumentTag, CreateDocumentTagRequest,
};
use crate::services::file_storage_service::{FileStorageService, ALL_DOCUMENTS};
use crate::utils::{AppError, Result};

/// Document Vault Service - manages files stored as DB blobs
pub struct DocumentService {
    db: PgPool,
    storage: FileStorageService,
}

impl DocumentService {
    pub fn new(db: PgPool) -> Self {
        let storage = FileStorageService::new(db.clone());
        Self { db, storage }
    }

    // ============================================================================
    // UPLOADS / DOCUMENTS
    // ============================================================================

    /// Upload a document
    pub async fn upload_document(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        data: Vec<u8>,
        original_name: &str,
        mime_type: &str,
        folder_id: Option<Uuid>,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<DocumentResponse> {
        // Validate file type
        if !ALL_DOCUMENTS.contains(&mime_type) && !mime_type.starts_with("image/") {
            return Err(AppError::Validation(format!(
                "File type '{}' not supported",
                mime_type
            )));
        }

        // Store file as blob
        let blob = self.storage.store_file(
            data,
            original_name,
            mime_type,
            None, // No expiration for documents
        ).await?;

        // Determine document type from mime
        let doc_type = document_type_from_mime(mime_type);

        // Create upload record
        let doc = sqlx::query_as::<_, DocumentResponse>(
            r#"
            INSERT INTO uploads (
                business_id, user_id, blob_id, original_name, 
                mime_type, file_size, folder_id, description, document_type,
                is_blob_stored, visibility
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, 'private')
            RETURNING 
                id, business_id, user_id, blob_id, original_name as name,
                mime_type, file_size, folder_id, description, document_type,
                visibility, created_at, updated_at
            "#
        )
        .bind(business_id)
        .bind(user_id)
        .bind(blob.id)
        .bind(original_name)
        .bind(mime_type)
        .bind(blob.file_size)
        .bind(folder_id)
        .bind(description)
        .bind(doc_type)
        .fetch_one(&self.db)
        .await?;

        // Add tags if provided
        for tag_name in tags {
            self.add_tag_to_document(doc.id, business_id, &tag_name).await?;
        }

        Ok(doc)
    }

    /// Get document by ID
    pub async fn get_document(&self, document_id: Uuid, user_id: Uuid) -> Result<DocumentResponse> {
        // Verify access
        let doc = sqlx::query_as::<_, DocumentResponse>(
            r#"
            SELECT 
                u.id, u.business_id, u.user_id, u.blob_id, u.original_name as name,
                u.mime_type, u.file_size, u.folder_id, u.description, u.document_type,
                u.visibility, u.created_at, u.updated_at
            FROM uploads u
            JOIN businesses b ON u.business_id = b.id
            LEFT JOIN business_members bm ON b.id = bm.business_id AND bm.user_id = $2
            WHERE u.id = $1 
              AND u.deleted_at IS NULL
              AND (b.owner_id = $2 OR bm.user_id = $2 OR u.visibility = 'public')
            "#
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        Ok(doc)
    }

    /// Get document data (binary)
    pub async fn get_document_data(&self, document_id: Uuid, user_id: Uuid) -> Result<(DocumentResponse, Vec<u8>)> {
        let doc = self.get_document(document_id, user_id).await?;
        
        let data = match doc.blob_id {
            Some(blob_id) => self.storage.get_file_data(blob_id).await?,
            None => return Err(AppError::Internal("Document has no blob".to_string())),
        };

        Ok((doc, data))
    }

    /// List documents for a business
    pub async fn list_documents(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        folder_id: Option<Uuid>,
        _document_type: Option<String>,
        _search: Option<String>,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<DocumentResponse>> {
        let offset = (page - 1) * per_page;

        // Simplified query without dynamic binding for now
        let docs = if let Some(fid) = folder_id {
            sqlx::query_as::<_, DocumentResponse>(
                r#"
                SELECT DISTINCT
                    u.id, u.business_id, u.user_id, u.blob_id, u.original_name as name,
                    u.mime_type, u.file_size, u.folder_id, u.description, u.document_type,
                    u.visibility, u.created_at, u.updated_at
                FROM uploads u
                JOIN businesses b ON u.business_id = b.id
                LEFT JOIN business_members bm ON b.id = bm.business_id AND bm.user_id = $1
                WHERE u.business_id = $2 
                  AND u.folder_id = $3
                  AND u.deleted_at IS NULL
                  AND (b.owner_id = $1 OR bm.user_id = $1)
                ORDER BY u.created_at DESC
                LIMIT $4 OFFSET $5
                "#
            )
            .bind(user_id)
            .bind(business_id)
            .bind(fid)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, DocumentResponse>(
                r#"
                SELECT DISTINCT
                    u.id, u.business_id, u.user_id, u.blob_id, u.original_name as name,
                    u.mime_type, u.file_size, u.folder_id, u.description, u.document_type,
                    u.visibility, u.created_at, u.updated_at
                FROM uploads u
                JOIN businesses b ON u.business_id = b.id
                LEFT JOIN business_members bm ON b.id = bm.business_id AND bm.user_id = $1
                WHERE u.business_id = $2 
                  AND u.folder_id IS NULL
                  AND u.deleted_at IS NULL
                  AND (b.owner_id = $1 OR bm.user_id = $1)
                ORDER BY u.created_at DESC
                LIMIT $3 OFFSET $4
                "#
            )
            .bind(user_id)
            .bind(business_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        Ok(docs)
    }

    /// Update document metadata
    pub async fn update_document(
        &self,
        document_id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        folder_id: Option<Uuid>,
        description: Option<String>,
    ) -> Result<DocumentResponse> {
        // Verify ownership
        self.verify_document_owner(document_id, user_id).await?;

        let doc = sqlx::query_as::<_, DocumentResponse>(
            r#"
            UPDATE uploads
            SET 
                original_name = COALESCE($1, original_name),
                folder_id = $2,
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $4
            RETURNING 
                id, business_id, user_id, blob_id, original_name as name,
                mime_type, file_size, folder_id, description, document_type,
                visibility, created_at, updated_at
            "#
        )
        .bind(name)
        .bind(folder_id)
        .bind(description)
        .bind(document_id)
        .fetch_one(&self.db)
        .await?;

        Ok(doc)
    }

    /// Delete document (soft delete)
    pub async fn delete_document(&self, document_id: Uuid, user_id: Uuid) -> Result<()> {
        // Verify ownership
        self.verify_document_owner(document_id, user_id).await?;

        // Get blob_id for potential cleanup
        let blob_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT blob_id FROM uploads WHERE id = $1"
        )
        .bind(document_id)
        .fetch_one(&self.db)
        .await?;

        // Soft delete document
        sqlx::query("UPDATE uploads SET deleted_at = NOW() WHERE id = $1")
            .bind(document_id)
            .execute(&self.db)
            .await?;

        // Optionally soft delete blob (hard delete can be done by cleanup job)
        if let Some(bid) = blob_id {
            self.storage.delete_file(bid).await?;
        }

        Ok(())
    }

    // ============================================================================
    // FOLDERS
    // ============================================================================

    /// Create folder
    pub async fn create_folder(
        &self,
        business_id: Uuid,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> Result<FolderResponse> {
        let folder = sqlx::query_as::<_, FolderResponse>(
            r#"
            INSERT INTO upload_folders (business_id, name, parent_id)
            VALUES ($1, $2, $3)
            RETURNING 
                id, business_id, name, parent_id,
                (SELECT COUNT(*) FROM uploads WHERE folder_id = upload_folders.id AND deleted_at IS NULL) as document_count,
                created_at
            "#
        )
        .bind(business_id)
        .bind(name)
        .bind(parent_id)
        .fetch_one(&self.db)
        .await?;

        Ok(folder)
    }

    /// List folders
    pub async fn list_folders(
        &self,
        business_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<FolderResponse>> {
        let folders = sqlx::query_as::<_, FolderResponse>(
            r#"
            SELECT 
                f.id, f.business_id, f.name, f.parent_id,
                (SELECT COUNT(*) FROM uploads WHERE folder_id = f.id AND deleted_at IS NULL) as document_count,
                f.created_at
            FROM upload_folders f
            WHERE f.business_id = $1 AND f.parent_id IS NOT DISTINCT FROM $2
            ORDER BY f.name
            "#
        )
        .bind(business_id)
        .bind(parent_id)
        .fetch_all(&self.db)
        .await?;

        Ok(folders)
    }

    /// Delete folder (must be empty)
    pub async fn delete_folder(&self, folder_id: Uuid, _user_id: Uuid) -> Result<()> {
        // Check if folder is empty
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM uploads WHERE folder_id = $1 AND deleted_at IS NULL"
        )
        .bind(folder_id)
        .fetch_one(&self.db)
        .await?;

        if count > 0 {
            return Err(AppError::Validation("Folder is not empty".to_string()));
        }

        sqlx::query("DELETE FROM upload_folders WHERE id = $1")
            .bind(folder_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ============================================================================
    // SHARING
    // ============================================================================

    /// Share document
    pub async fn share_document(
        &self,
        document_id: Uuid,
        user_id: Uuid,
        req: ShareDocumentRequest,
    ) -> Result<DocumentShareResponse> {
        // Verify ownership
        self.verify_document_owner(document_id, user_id).await?;

        // Generate share token
        let token = generate_share_token();

        // Hash password if provided
        let password_hash = match req.password {
            Some(pwd) if !pwd.is_empty() => {
                Some(hash_password(&pwd)?)
            }
            _ => None,
        };

        let expires_at = req.expiry_days.map(|days| Utc::now() + Duration::days(days as i64));

        let share = sqlx::query_as::<_, DocumentShareResponse>(
            r#"
            INSERT INTO document_shares (
                upload_id, share_token, password_hash, allow_download, 
                allow_preview, expires_at, max_downloads, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, upload_id, share_token, allow_download, allow_preview,
                expires_at, max_downloads, download_count, created_at
            "#
        )
        .bind(document_id)
        .bind(&token)
        .bind(password_hash)
        .bind(req.allow_download.unwrap_or(true))
        .bind(req.allow_preview.unwrap_or(true))
        .bind(expires_at)
        .bind(req.max_downloads)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(share)
    }

    /// Access shared document
    pub async fn access_shared_document(
        &self,
        token: &str,
        password: Option<String>,
    ) -> Result<(DocumentResponse, Vec<u8>)> {
        // Get share info
        let share = sqlx::query(
            r#"
            SELECT 
                ds.id, ds.upload_id, ds.password_hash, ds.allow_download,
                ds.allow_preview, ds.expires_at, ds.max_downloads, ds.download_count
            FROM document_shares ds
            WHERE ds.share_token = $1
              AND (ds.expires_at IS NULL OR ds.expires_at > NOW())
            "#
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Share not found or expired".to_string()))?;

        let share_id: Uuid = share.get("id");
        let upload_id: Uuid = share.get("upload_id");
        let password_hash: Option<String> = share.get("password_hash");
        let max_downloads: Option<i32> = share.get("max_downloads");
        let download_count: i32 = share.get("download_count");
        let allow_download: bool = share.get("allow_download");

        // Check password
        if let Some(hash) = password_hash {
            let pwd = password.ok_or_else(|| AppError::Unauthorized("Password required".to_string()))?;
            if !verify_password(&pwd, &hash)? {
                return Err(AppError::Unauthorized("Invalid password".to_string()));
            }
        }

        // Check download limit
        if let Some(max) = max_downloads {
            if download_count >= max {
                return Err(AppError::Forbidden("Download limit reached".to_string()));
            }
        }

        if !allow_download {
            return Err(AppError::Forbidden("Downloads not allowed for this share".to_string()));
        }

        // Update download count
        sqlx::query(
            "UPDATE document_shares SET download_count = download_count + 1, last_accessed_at = NOW() WHERE id = $1"
        )
        .bind(share_id)
        .execute(&self.db)
        .await?;

        // Get document data
        let doc = sqlx::query_as::<_, DocumentResponse>(
            r#"
            SELECT 
                u.id, u.business_id, u.user_id, u.blob_id, u.original_name as name,
                u.mime_type, u.file_size, u.folder_id, u.description, u.document_type,
                u.visibility, u.created_at, u.updated_at
            FROM uploads u
            WHERE u.id = $1 AND u.deleted_at IS NULL
            "#
        )
        .bind(upload_id)
        .fetch_one(&self.db)
        .await?;

        let data = match doc.blob_id {
            Some(blob_id) => self.storage.get_file_data(blob_id).await?,
            None => return Err(AppError::Internal("Document has no data".to_string())),
        };

        Ok((doc, data))
    }

    // ============================================================================
    // TAGS
    // ============================================================================

    /// Create tag
    pub async fn create_tag(
        &self,
        business_id: Uuid,
        req: CreateDocumentTagRequest,
    ) -> Result<DocumentTag> {
        let tag = sqlx::query_as::<_, DocumentTag>(
            r#"
            INSERT INTO document_tags (business_id, name, color)
            VALUES ($1, $2, $3)
            ON CONFLICT (business_id, name) DO UPDATE SET color = EXCLUDED.color
            RETURNING id, business_id, name, color, created_at
            "#
        )
        .bind(business_id)
        .bind(&req.name)
        .bind(req.color.unwrap_or_else(|| "#6366F1".to_string()))
        .fetch_one(&self.db)
        .await?;

        Ok(tag)
    }

    /// List tags
    pub async fn list_tags(&self, business_id: Uuid) -> Result<Vec<DocumentTag>> {
        let tags = sqlx::query_as::<_, DocumentTag>(
            "SELECT id, business_id, name, color, created_at FROM document_tags WHERE business_id = $1 ORDER BY name"
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(tags)
    }

    /// Add tag to document
    async fn add_tag_to_document(&self, document_id: Uuid, business_id: Uuid, tag_name: &str) -> Result<()> {
        // Get or create tag
        let tag: DocumentTag = sqlx::query_as(
            r#"
            INSERT INTO document_tags (business_id, name)
            VALUES ($1, $2)
            ON CONFLICT (business_id, name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id, business_id, name, color, created_at
            "#
        )
        .bind(business_id)
        .bind(tag_name)
        .fetch_one(&self.db)
        .await?;

        // Link tag to document
        sqlx::query(
            r#"
            INSERT INTO upload_tag_links (upload_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(document_id)
        .bind(tag.id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ============================================================================
    // DOCUMENT TEMPLATES
    // ============================================================================

    /// List available templates
    pub async fn list_templates(&self, country_code: Option<String>) -> Result<Vec<serde_json::Value>> {
        let templates = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT json_build_object(
                'id', id,
                'name', name,
                'description', description,
                'category', category,
                'country_code', country_code
            ) as template
            FROM document_templates
            WHERE is_active = true
              AND (country_code IS NULL OR country_code = $1)
            ORDER BY name
            "#
        )
        .bind(country_code)
        .fetch_all(&self.db)
        .await?;

        Ok(templates.into_iter().map(|t| t.0).collect())
    }

    // ============================================================================
    // HELPERS
    // ============================================================================

    async fn verify_document_owner(&self, document_id: Uuid, user_id: Uuid) -> Result<()> {
        let has_access = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM uploads u
                JOIN businesses b ON u.business_id = b.id
                WHERE u.id = $1 
                  AND (b.owner_id = $2 OR u.user_id = $2)
                  AND u.deleted_at IS NULL
            )
            "#
        )
        .bind(document_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !has_access {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(())
    }
}

/// Generate random share token
fn generate_share_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Hash password for share protection
fn hash_password(password: &str) -> Result<String> {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Password hash failed: {}", e)))
}

/// Verify share password
fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid hash: {}", e)))?;

    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Determine document type from MIME type
fn document_type_from_mime(mime: &str) -> Option<String> {
    match mime {
        "application/pdf" => Some("pdf".to_string()),
        "text/plain" => Some("text".to_string()),
        "image/jpeg" | "image/png" | "image/gif" | "image/webp" => Some("image".to_string()),
        "application/msword" | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => Some("document".to_string()),
        "application/vnd.ms-excel" | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some("spreadsheet".to_string()),
        _ => None,
    }
}
