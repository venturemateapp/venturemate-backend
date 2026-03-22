//! Data Room Service
//! 
//! Manages secure investor data rooms with file storage,
// access logs, and sharing capabilities.

use crate::utils::{AppError, Result, hash_password, verify_password};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_ENGINE};
use crate::models::documents::{
    AccessDataRoomRequest, AccessLogEntry, AddDataRoomFileRequest, CreateDataRoomRequest,
    DataRoom, DataRoomAccessLogsResponse, DataRoomAccessResponse, DataRoomFile,
    DataRoomFileResponse, DataRoomFolder, DataRoomResponse, DataRoomSummary,
    ShareDataRoomRequest, ShareDataRoomResponse,
};
use sqlx::PgPool;

use tracing::info;
use uuid::Uuid;

pub struct DataRoomService {
    db: PgPool,
}

impl DataRoomService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create a new data room
    pub async fn create_data_room(
        &self,
        user_id: Uuid,
        req: CreateDataRoomRequest,
    ) -> Result<DataRoomResponse> {
        info!("Creating data room for business: {}", req.business_id);

        // Verify business exists and user has access
        let business: Option<(String,)> = sqlx::query_as(
            "SELECT name FROM businesses WHERE id = $1 AND user_id = $2"
        )
        .bind(req.business_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some((business_name,)) = business else {
            return Err(AppError::NotFound("Business not found".to_string()));
        };

        let data_room_id = Uuid::new_v4();
        let default_name = format!("{} - Investor Data Room", business_name);
        let name = if req.name.is_empty() { default_name } else { req.name };

        sqlx::query(
            "INSERT INTO data_rooms (id, business_id, name, description, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, true, NOW(), NOW())"
        )
        .bind(data_room_id)
        .bind(req.business_id)
        .bind(&name)
        .bind(&req.description)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Auto-add generated documents to the data room
        self.auto_populate_data_room(data_room_id, req.business_id).await?;

        info!("Data room created: {} for business: {}", data_room_id, req.business_id);

        Ok(DataRoomResponse {
            id: data_room_id,
            business_id: req.business_id,
            name,
            description: req.description.clone(),
            shareable_link: None,
            password_protected: false,
            expires_at: None,
            access_count: 0,
            download_limit: None,
            watermark_enabled: false,
            is_active: true,
            file_count: 0,
            created_at: chrono::Utc::now(),
            document_count: None,
            view_count: None,
            is_public: None,
            access_url: None,
        })
    }

    /// Get data room by ID
    pub async fn get_data_room(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
    ) -> Result<DataRoomResponse> {
        let data_room: Option<DataRoom> = sqlx::query_as(
            "SELECT dr.* FROM data_rooms dr
             JOIN businesses b ON dr.business_id = b.id
             WHERE dr.id = $1 AND b.user_id = $2"
        )
        .bind(data_room_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(dr) = data_room else {
            return Err(AppError::NotFound("Data room not found".to_string()));
        };

        let file_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM data_room_files WHERE data_room_id = $1"
        )
        .bind(data_room_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(DataRoomResponse {
            id: dr.id,
            business_id: dr.business_id,
            name: dr.name,
            description: dr.description,
            shareable_link: dr.shareable_link,
            password_protected: dr.password_protected,
            expires_at: dr.expires_at,
            access_count: dr.access_count,
            download_limit: dr.download_limit,
            watermark_enabled: dr.watermark_enabled,
            is_active: dr.is_active,
            file_count,
            created_at: dr.created_at,
            document_count: None,
            view_count: None,
            is_public: None,
            access_url: None,
        })
    }

    /// List data rooms for a business
    pub async fn list_data_rooms(
        &self,
        user_id: Uuid,
        business_id: Uuid,
    ) -> Result<Vec<DataRoomResponse>> {
        // Verify ownership
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM businesses WHERE id = $1 AND user_id = $2)"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        if !exists {
            return Err(AppError::NotFound("Business not found".to_string()));
        }

        let data_rooms: Vec<DataRoomSummary> = sqlx::query_as(
            "SELECT dr.*, b.name as business_name,
                (SELECT COUNT(*) FROM data_room_files drf WHERE drf.data_room_id = dr.id) as file_count,
                (SELECT COUNT(*) FROM data_room_access_logs dral WHERE dral.data_room_id = dr.id) as access_count
             FROM data_rooms dr
             JOIN businesses b ON dr.business_id = b.id
             WHERE dr.business_id = $1 AND b.user_id = $2
             ORDER BY dr.created_at DESC"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(data_rooms.into_iter().map(|dr| DataRoomResponse {
            id: dr.id,
            business_id: dr.business_id,
            name: dr.name,
            description: dr.description,
            shareable_link: dr.shareable_link,
            password_protected: dr.password_protected,
            expires_at: dr.expires_at,
            access_count: dr.access_count,
            download_limit: dr.download_limit,
            watermark_enabled: dr.watermark_enabled,
            is_active: dr.is_active,
            file_count: dr.file_count,
            created_at: dr.created_at,
            document_count: None,
            view_count: None,
            is_public: None,
            access_url: None,
        }).collect())
    }

    /// Update data room
    pub async fn update_data_room(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        is_active: Option<bool>,
    ) -> Result<DataRoomResponse> {
        let data_room: Option<DataRoom> = sqlx::query_as(
            "SELECT dr.* FROM data_rooms dr
             JOIN businesses b ON dr.business_id = b.id
             WHERE dr.id = $1 AND b.user_id = $2"
        )
        .bind(data_room_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(mut dr) = data_room else {
            return Err(AppError::NotFound("Data room not found".to_string()));
        };

        if let Some(n) = name {
            dr.name = n;
        }
        if let Some(d) = description {
            dr.description = Some(d);
        }
        if let Some(a) = is_active {
            dr.is_active = a;
        }

        sqlx::query(
            "UPDATE data_rooms SET name = $1, description = $2, is_active = $3, updated_at = NOW()
             WHERE id = $4"
        )
        .bind(&dr.name)
        .bind(&dr.description)
        .bind(dr.is_active)
        .bind(data_room_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        self.get_data_room(user_id, data_room_id).await
    }

    /// Delete data room
    pub async fn delete_data_room(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
    ) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM data_rooms dr USING businesses b 
             WHERE dr.id = $1 AND dr.business_id = b.id AND b.user_id = $2"
        )
        .bind(data_room_id)
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Data room not found".to_string()));
        }

        Ok(())
    }

    /// Share data room (create shareable link)
    pub async fn share_data_room(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
        req: ShareDataRoomRequest,
    ) -> Result<ShareDataRoomResponse> {
        let data_room: Option<DataRoom> = sqlx::query_as(
            "SELECT dr.* FROM data_rooms dr
             JOIN businesses b ON dr.business_id = b.id
             WHERE dr.id = $1 AND b.user_id = $2"
        )
        .bind(data_room_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(mut dr) = data_room else {
            return Err(AppError::NotFound("Data room not found".to_string()));
        };

        // Generate shareable link
        let share_token = format!("dr-{}", &Uuid::new_v4().to_string().replace("-", "").to_lowercase()[..16]);
        let share_link = format!("https://app.venturemate.com/d/{}?utm_source=investor_data_room", share_token);

        dr.shareable_link = Some(share_link.clone());
        dr.password_protected = req.password.is_some();
        dr.password_hash = match req.password {
            Some(p) => hash_password(&p).ok(),
            None => None,
        };
        dr.expires_at = req.expires_in_days.map(|days| chrono::Utc::now() + chrono::Duration::days(days as i64));
        dr.download_limit = req.download_limit;
        dr.watermark_enabled = req.watermark_text.is_some();
        dr.watermark_text = req.watermark_text;

        sqlx::query(
            "UPDATE data_rooms SET
                shareable_link = $1,
                password_protected = $2,
                password_hash = $3,
                expires_at = $4,
                download_limit = $5,
                watermark_enabled = $6,
                watermark_text = $7,
                updated_at = NOW()
             WHERE id = $8"
        )
        .bind(&dr.shareable_link)
        .bind(dr.password_protected)
        .bind(&dr.password_hash)
        .bind(dr.expires_at)
        .bind(dr.download_limit)
        .bind(dr.watermark_enabled)
        .bind(&dr.watermark_text)
        .bind(data_room_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(ShareDataRoomResponse {
            share_link,
            expires_at: dr.expires_at,
            password_protected: dr.password_protected,
        })
    }

    /// Access data room (public/investor access)
    pub async fn access_data_room(
        &self,
        share_token: &str,
        req: AccessDataRoomRequest,
        ip_addr: Option<String>,
        user_agent: Option<&str>,
    ) -> Result<DataRoomAccessResponse> {
        let share_link = format!("https://app.venturemate.com/d/{}", share_token);

        let data_room: Option<DataRoom> = sqlx::query_as(
            "SELECT * FROM data_rooms WHERE shareable_link = $1 AND is_active = true"
        )
        .bind(&share_link)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(dr) = data_room else {
            return Err(AppError::NotFound("Data room not found or inactive".to_string()));
        };

        // Check expiration
        if let Some(expires) = dr.expires_at {
            if expires < chrono::Utc::now() {
                return Err(AppError::Unauthorized("Data room link has expired".to_string()));
            }
        }

        // Check password
        if dr.password_protected {
            let Some(password) = req.password else {
                return Err(AppError::Unauthorized("Password required".to_string()));
            };

            let Some(hash) = dr.password_hash else {
                return Err(AppError::Unauthorized("Invalid password".to_string()));
            };

            match verify_password(&password, &hash) {
                Ok(true) => {},
                _ => return Err(AppError::Unauthorized("Invalid password".to_string())),
            }
        }

        // Increment access count
        sqlx::query(
            "UPDATE data_rooms SET access_count = access_count + 1 WHERE id = $1"
        )
        .bind(dr.id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Log access
        sqlx::query(
            "INSERT INTO data_room_access_logs (data_room_id, ip_address, user_agent, email, action, created_at)
             VALUES ($1, $2, $3, $4, 'viewed', NOW())"
        )
        .bind(dr.id)
        .bind(ip_addr)
        .bind(user_agent)
        .bind(req.email)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Get files
        let files: Vec<DataRoomFile> = sqlx::query_as(
            "SELECT * FROM data_room_files WHERE data_room_id = $1 ORDER BY folder, file_name"
        )
        .bind(dr.id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        let file_responses = files.into_iter().map(|f| DataRoomFileResponse {
            id: f.id,
            folder: f.folder,
            file_name: f.file_name,
            file_mime_type: f.file_mime_type.unwrap_or_default(),
            file_size: f.file_size.unwrap_or(0),
            version: f.version,
            description: f.description,
            download_url: Some(format!("/api/v1/data-room/{}/files/{}/download", dr.id, f.id)),
            created_at: f.created_at,
        }).collect();

        Ok(DataRoomAccessResponse {
            data_room_id: dr.id,
            name: dr.name,
            description: dr.description,
            files: file_responses,
            watermark_enabled: dr.watermark_enabled,
        })
    }

    /// Add file to data room
    pub async fn add_file(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
        req: AddDataRoomFileRequest,
    ) -> Result<DataRoomFileResponse> {
        // Verify ownership
        let data_room: Option<DataRoom> = sqlx::query_as(
            "SELECT dr.* FROM data_rooms dr
             JOIN businesses b ON dr.business_id = b.id
             WHERE dr.id = $1 AND b.user_id = $2"
        )
        .bind(data_room_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(_dr) = data_room else {
            return Err(AppError::NotFound("Data room not found".to_string()));
        };

        // Validate folder
        let folder_enum = req.folder.parse::<DataRoomFolder>()
            .map_err(AppError::Validation)?;

        // Decode base64 file data
        let file_data = BASE64_ENGINE.decode(&req.file_data)
            .map_err(|_| AppError::Validation("Invalid file data".to_string()))?;

        let file_id = Uuid::new_v4();
        let file_size = file_data.len() as i64;

        sqlx::query(
            "INSERT INTO data_room_files (id, data_room_id, folder, file_name, file_data, file_mime_type, file_size, version, description, uploaded_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9, NOW(), NOW())"
        )
        .bind(file_id)
        .bind(data_room_id)
        .bind(folder_enum.as_str())
        .bind(&req.file_name)
        .bind(&file_data)
        .bind(&req.mime_type)
        .bind(file_size)
        .bind(&req.description)
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(DataRoomFileResponse {
            id: file_id,
            folder: folder_enum.as_str().to_string(),
            file_name: req.file_name,
            file_mime_type: req.mime_type,
            file_size,
            version: 1,
            description: req.description,
            download_url: Some(format!("/api/v1/data-room/{}/files/{}/download", data_room_id, file_id)),
            created_at: chrono::Utc::now(),
        })
    }

    /// Download file from data room
    pub async fn download_file(
        &self,
        _user_id: Option<Uuid>, // Optional for public access
        data_room_id: Uuid,
        file_id: Uuid,
    ) -> Result<(String, Vec<u8>)> { // (filename, content)
        let file: Option<DataRoomFile> = sqlx::query_as(
            "SELECT * FROM data_room_files WHERE id = $1 AND data_room_id = $2"
        )
        .bind(file_id)
        .bind(data_room_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(file) = file else {
            return Err(AppError::NotFound("File not found".to_string()));
        };

        let content = file.file_data.ok_or_else(|| {
            AppError::Internal("File content not found".to_string())
        })?;

        Ok((file.file_name, content))
    }

    /// Delete file from data room
    pub async fn delete_file(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
        file_id: Uuid,
    ) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM data_room_files drf USING data_rooms dr, businesses b
             WHERE drf.id = $1 AND drf.data_room_id = $2
             AND drf.data_room_id = dr.id AND dr.business_id = b.id AND b.user_id = $3"
        )
        .bind(file_id)
        .bind(data_room_id)
        .bind(user_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("File not found".to_string()));
        }

        Ok(())
    }

    /// Get access logs for a data room
    pub async fn get_access_logs(
        &self,
        user_id: Uuid,
        data_room_id: Uuid,
    ) -> Result<DataRoomAccessLogsResponse> {
        // Verify ownership
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM data_rooms dr 
             JOIN businesses b ON dr.business_id = b.id
             WHERE dr.id = $1 AND b.user_id = $2)"
        )
        .bind(data_room_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        if !exists {
            return Err(AppError::NotFound("Data room not found".to_string()));
        }

        let logs: Vec<(Uuid, Option<String>, Option<String>, String, Option<String>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT dral.id, dral.ip_address, dral.email, dral.action, drf.file_name, dral.created_at
             FROM data_room_access_logs dral
             LEFT JOIN data_room_files drf ON dral.file_id = drf.id
             WHERE dral.data_room_id = $1
             ORDER BY dral.created_at DESC"
        )
        .bind(data_room_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(DataRoomAccessLogsResponse {
            access_logs: logs.into_iter().map(|(id, ip, email, action, file_name, created_at)| AccessLogEntry {
                id,
                ip_address: ip.map(|i| i.to_string()),
                email,
                action,
                file_name,
                created_at,
            }).collect(),
        })
    }

    /// Auto-populate data room with generated documents
    async fn auto_populate_data_room(
        &self,
        data_room_id: Uuid,
        business_id: Uuid,
    ) -> Result<()> {
        // Get generated documents for the business
        let documents: Vec<(Uuid, String, Option<Vec<u8>>, Option<String>)> = sqlx::query_as(
            "SELECT id, document_name, file_data, file_format FROM generated_documents 
             WHERE business_id = $1 AND status = 'ready'"
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        for (_doc_id, doc_name, file_data, file_format) in documents {
            if let Some(data) = file_data {
                let folder = match doc_name.to_lowercase().as_str() {
                    n if n.contains("business plan") => DataRoomFolder::BusinessPlan,
                    n if n.contains("pitch deck") => DataRoomFolder::PitchDeck,
                    n if n.contains("executive summary") => DataRoomFolder::ExecutiveSummary,
                    n if n.contains("financial") => DataRoomFolder::Financials,
                    _ => DataRoomFolder::Other,
                };

                let _ = sqlx::query(
                    "INSERT INTO data_room_files (id, data_room_id, folder, file_name, file_data, file_mime_type, file_size, version, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, 1, NOW(), NOW())"
                )
                .bind(Uuid::new_v4())
                .bind(data_room_id)
                .bind(folder.as_str())
                .bind(format!("{}.{}", doc_name, file_format.unwrap_or_else(|| "pdf".to_string())))
                .bind(&data)
                .bind("application/pdf")
                .bind(data.len() as i64)
                .execute(&self.db)
                .await;
            }
        }

        Ok(())
    }
}

