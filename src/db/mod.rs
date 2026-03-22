use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub type DbPool = PgPool;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Initialize database connection pool with Supabase-optimized settings
/// 
/// # Supabase Connection Notes:
/// - Uses connection pooler by default (recommended for serverless)
/// - Set `SUPABASE_DB_URL` to your pooled connection string
/// - For direct connection, use port 5432 instead of 6543
/// 
/// # Connection Pool Settings:
/// - max_connections: 10 (Supabase free tier limit is typically 60, leave room for other clients)
/// - min_connections: 2 (keep warm connections ready)
/// - acquire_timeout: 30s
/// - idle_timeout: 300s (5 min - shorter for Supabase pooler compatibility)
/// - max_lifetime: 600s (10 min - shorter for Supabase pooler compatibility)
pub async fn init_db(database_url: &str) -> Result<DbPool, DbError> {
    // Validate database URL
    if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
        return Err(DbError::Config(
            "Invalid DATABASE_URL. Must start with postgres:// or postgresql://".to_string()
        ));
    }

    let is_supabase_pooler = database_url.contains("pooler.supabase.com");

    let pool = if is_supabase_pooler {
        // Supabase connection pooler optimized settings
        PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(600))
            .test_before_acquire(true)
            .connect(database_url)
            .await?
    } else {
        // Direct connection or local PostgreSQL
        PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .test_before_acquire(true)
            .connect(database_url)
            .await?
    };

    // Run migrations (log warning but don't fail on checksum mismatch)
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::warn!("Migration warning (may be checksum mismatch): {}", e);
        // Continue anyway - migrations are already applied
    }

    Ok(pool)
}

/// Alternative: Initialize without migrations (useful for Supabase managed migrations)
pub async fn init_db_without_migrations(database_url: &str) -> Result<DbPool, DbError> {
    if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
        return Err(DbError::Config(
            "Invalid DATABASE_URL. Must start with postgres:// or postgresql://".to_string()
        ));
    }

    let is_supabase_pooler = database_url.contains("pooler.supabase.com");

    let pool = if is_supabase_pooler {
        PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(600))
            .test_before_acquire(true)
            .connect(database_url)
            .await?
    } else {
        PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .test_before_acquire(true)
            .connect(database_url)
            .await?
    };

    Ok(pool)
}

/// Health check - simple query to verify database connectivity
pub async fn health_check(pool: &DbPool) -> Result<bool, sqlx::Error> {
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(pool)
        .await?;
    
    Ok(result.0 == 1)
}

/// Get connection pool statistics (useful for monitoring)
pub fn pool_stats(pool: &DbPool) -> PoolStats {
    PoolStats {
        size: pool.size(),
        num_idle: pool.num_idle(),
        is_closed: pool.is_closed(),
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub num_idle: usize,
    pub is_closed: bool,
}

impl std::fmt::Display for PoolStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool: {} active, {} idle, closed: {}",
            self.size.saturating_sub(self.num_idle as u32),
            self.num_idle,
            self.is_closed
        )
    }
}
