use crate::db::DbPool;
use crate::utils::{AppError, Result};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub version: String,
}

pub struct HealthService {
    db: DbPool,
}

impl HealthService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub async fn check(&self) -> Result<HealthStatus> {
        let db_healthy = sqlx::query_as::<_, (i32,)>("SELECT 1")
            .fetch_one(&self.db)
            .await
            .map(|r| r.0 == 1)
            .map_err(AppError::Database)?;
        
        Ok(HealthStatus {
            status: "healthy".to_string(),
            database: if db_healthy { "connected".to_string() } else { "disconnected".to_string() },
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }
}
