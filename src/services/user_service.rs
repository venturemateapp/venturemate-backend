use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{UpdateProfileRequest, User, UserResponse};
use crate::utils::Result;

pub struct UserService {
    db: PgPool,
}

impl UserService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get current user profile
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

        // Get businesses count
        let businesses_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM businesses WHERE owner_id = $1 AND deleted_at IS NULL"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let mut response: UserResponse = user.into();
        response.businesses_count = businesses_count;

        Ok(response)
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<UserResponse> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET 
                first_name = COALESCE($1, first_name),
                last_name = COALESCE($2, last_name),
                phone = COALESCE($3, phone),
                timezone = COALESCE($4, timezone),
                updated_at = NOW()
            WHERE id = $5
            RETURNING *
            "#,
        )
        .bind(req.first_name)
        .bind(req.last_name)
        .bind(req.phone)
        .bind(req.timezone)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(user.into())
    }

    /// Update avatar URL
    pub async fn update_avatar(&self, user_id: Uuid, avatar_url: String) -> Result<UserResponse> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
        )
        .bind(avatar_url)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(user.into())
    }

    /// Delete user account (soft delete)
    pub async fn delete_account(&self, user_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE users SET status = 'deleted', deleted_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// List all users (admin only)
    pub async fn list_users(&self, page: i64, per_page: i64) -> Result<(Vec<UserResponse>, i64)> {
        let offset = (page - 1) * per_page;

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users 
            WHERE status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE status != 'deleted'"
        )
        .fetch_one(&self.db)
        .await?;

        Ok((users.into_iter().map(Into::into).collect(), total))
    }
}
