use sqlx::{PgPool, Row};
use uuid::Uuid;
use sha2::{Sha256, Digest};

use crate::utils::{AppError, Result};

/// File stored as blob in PostgreSQL
#[derive(Debug, Clone)]
pub struct FileBlob {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub data: Vec<u8>,
    pub is_compressed: bool,
}

/// File storage service - stores files directly in PostgreSQL as BYTEA
pub struct FileStorageService {
    db: PgPool,
    max_file_size: usize, // 50MB default
}

impl FileStorageService {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            max_file_size: 50 * 1024 * 1024, // 50MB
        }
    }

    /// Store a file as blob in the database
    pub async fn store_file(
        &self,
        data: Vec<u8>,
        original_name: &str,
        mime_type: &str,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<FileBlob> {
        // Validate file size
        if data.len() > self.max_file_size {
            return Err(AppError::Validation(format!(
                "File too large: {} bytes (max: {} bytes)",
                data.len(),
                self.max_file_size
            )));
        }

        // Calculate hash for deduplication
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let file_hash = format!("{:x}", hasher.finalize());

        // Check if file already exists (deduplication)
        let existing = sqlx::query_as::<_, FileBlob>(
            r#"
            SELECT id, original_name, mime_type, file_size, data, is_compressed
            FROM file_blobs
            WHERE file_hash = $1 AND deleted_at IS NULL
            "#
        )
        .bind(&file_hash)
        .fetch_optional(&self.db)
        .await?;

        if let Some(blob) = existing {
            // Update access count for existing file
            sqlx::query(
                "UPDATE file_blobs SET access_count = access_count + 1, last_accessed_at = NOW() WHERE id = $1"
            )
            .bind(blob.id)
            .execute(&self.db)
            .await?;
            
            return Ok(blob);
        }

        // Store original size before potential move
        let original_size = data.len();
        
        // Compress large files (>100KB)
        let (compressed_data, is_compressed) = if data.len() > 100 * 1024 {
            match compress_data(&data) {
                Ok(compressed) if compressed.len() < data.len() => (compressed, true),
                _ => (data, false),
            }
        } else {
            (data, false)
        };

        // Store in database
        let blob = sqlx::query_as::<_, FileBlob>(
            r#"
            INSERT INTO file_blobs (
                original_name, mime_type, file_size, file_hash, 
                data, is_compressed, original_size, compression_algorithm, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, original_name, mime_type, file_size, data, is_compressed
            "#
        )
        .bind(original_name)
        .bind(mime_type)
        .bind(compressed_data.len() as i64)
        .bind(&file_hash)
        .bind(&compressed_data)
        .bind(is_compressed)
        .bind(original_size as i64)
        .bind(if is_compressed { Some("gzip") } else { None })
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(blob)
    }

    /// Get a file blob by ID
    pub async fn get_file(&self, blob_id: Uuid) -> Result<FileBlob> {
        let blob = sqlx::query_as::<_, FileBlob>(
            r#"
            SELECT id, original_name, mime_type, file_size, data, is_compressed
            FROM file_blobs
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(blob_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

        // Update access stats
        sqlx::query(
            "UPDATE file_blobs SET access_count = access_count + 1, last_accessed_at = NOW() WHERE id = $1"
        )
        .bind(blob_id)
        .execute(&self.db)
        .await?;

        Ok(blob)
    }

    /// Get file metadata without data
    pub async fn get_file_metadata(&self, blob_id: Uuid) -> Result<FileMetadata> {
        let meta = sqlx::query_as::<_, FileMetadata>(
            r#"
            SELECT id, original_name, mime_type, file_size, is_compressed, created_at
            FROM file_blobs
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(blob_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

        Ok(meta)
    }

    /// Delete a file (soft delete)
    pub async fn delete_file(&self, blob_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE file_blobs SET deleted_at = NOW() WHERE id = $1")
            .bind(blob_id)
            .execute(&self.db)
            .await?;
        
        Ok(())
    }

    /// Hard delete (use with caution)
    pub async fn hard_delete_file(&self, blob_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM file_blobs WHERE id = $1")
            .bind(blob_id)
            .execute(&self.db)
            .await?;
        
        Ok(())
    }

    /// Get file data (decompress if needed)
    pub async fn get_file_data(&self, blob_id: Uuid) -> Result<Vec<u8>> {
        let row = sqlx::query(
            "SELECT data, is_compressed, original_size FROM file_blobs WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(blob_id)
        .fetch_one(&self.db)
        .await?;

        let data: Vec<u8> = row.get("data");
        let is_compressed: bool = row.get("is_compressed");

        if is_compressed {
            decompress_data(&data)
        } else {
            Ok(data)
        }
    }

    /// Clean up expired files
    pub async fn cleanup_expired(&self) -> Result<i64> {
        let result = sqlx::query(
            "UPDATE file_blobs SET deleted_at = NOW() WHERE expires_at < NOW() AND deleted_at IS NULL"
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Get storage stats
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let stats = sqlx::query_as::<_, StorageStats>(
            r#"
            SELECT 
                COUNT(*) as total_files,
                COALESCE(SUM(file_size), 0) as total_size,
                COALESCE(SUM(CASE WHEN is_compressed THEN original_size ELSE file_size END), 0) as original_size
            FROM file_blobs
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_one(&self.db)
        .await?;

        Ok(stats)
    }
}

/// File metadata (without data)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileMetadata {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub is_compressed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Storage statistics
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StorageStats {
    pub total_files: i64,
    pub total_size: i64,
    pub original_size: i64,
}

// SQLx mappers
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for FileBlob {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(FileBlob {
            id: row.try_get("id")?,
            original_name: row.try_get("original_name")?,
            mime_type: row.try_get("mime_type")?,
            file_size: row.try_get("file_size")?,
            data: row.try_get("data")?,
            is_compressed: row.try_get("is_compressed")?,
        })
    }
}

/// Compress data using gzip
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;
    
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data).map_err(|e| AppError::Internal(format!("Compression failed: {}", e)))?;
    encoder.finish().map_err(|e| AppError::Internal(format!("Compression failed: {}", e)))
}

/// Decompress gzipped data
fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Read;
    
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result).map_err(|e| AppError::Internal(format!("Decompression failed: {}", e)))?;
    Ok(result)
}

/// File upload request
#[derive(Debug, Clone)]
pub struct FileUploadRequest {
    pub data: Vec<u8>,
    pub original_name: String,
    pub mime_type: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Validate file type
pub fn validate_file_type(mime_type: &str, allowed_types: &[&str]) -> Result<()> {
    if allowed_types.contains(&mime_type) || allowed_types.contains(&"*/*") {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "File type '{}' not allowed. Allowed types: {:?}",
            mime_type, allowed_types
        )))
    }
}

/// Common MIME type groups
pub const IMAGE_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp", "image/svg+xml"];
pub const DOCUMENT_TYPES: &[&str] = &["application/pdf", "text/plain", "application/msword", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"];
pub const SPREADSHEET_TYPES: &[&str] = &["application/vnd.ms-excel", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "text/csv"];
pub const PRESENTATION_TYPES: &[&str] = &["application/vnd.ms-powerpoint", "application/vnd.openxmlformats-officedocument.presentationml.presentation"];
pub const ALL_DOCUMENTS: &[&str] = &["application/pdf", "text/plain", "application/msword", "application/vnd.openxmlformats-officedocument.wordprocessingml.document", 
    "application/vnd.ms-excel", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "text/csv",
    "application/vnd.ms-powerpoint", "application/vnd.openxmlformats-officedocument.presentationml.presentation"];
