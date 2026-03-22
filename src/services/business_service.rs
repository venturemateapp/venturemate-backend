use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    Business, BusinessChecklistResponse, BusinessResponse, ChecklistCategoryResponse, ChecklistItemResponse,
    CreateBusinessRequest, Industry, UpdateBusinessRequest, UpdateChecklistItemRequest,
};
use crate::utils::{AppError, Result};

pub struct BusinessService {
    db: PgPool,
}

impl BusinessService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Create a new business
    pub async fn create(
        &self,
        owner_id: Uuid,
        req: CreateBusinessRequest,
    ) -> Result<BusinessResponse> {
        let slug = slug::slugify(&req.name);

        // Check if slug is unique
        let existing = sqlx::query_scalar::<_, Uuid>("SELECT id FROM businesses WHERE slug = $1")
            .bind(&slug)
            .fetch_optional(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(
                "A business with this name already exists".to_string(),
            ));
        }

        let business = sqlx::query_as::<_, Business>(
            r#"
            INSERT INTO businesses (owner_id, name, slug, tagline, description, industry, sub_industry, country_code, city, legal_structure)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(owner_id)
        .bind(&req.name)
        .bind(&slug)
        .bind(req.tagline)
        .bind(req.description)
        .bind(&req.industry)
        .bind(req.sub_industry)
        .bind(&req.country_code)
        .bind(req.city)
        .bind(req.legal_structure)
        .fetch_one(&self.db)
        .await?;

        // Add owner as founder
        sqlx::query(
            r#"
            INSERT INTO business_members (business_id, user_id, role, permissions)
            VALUES ($1, $2, 'founder', '{}')
            "#,
        )
        .bind(business.id)
        .bind(owner_id)
        .execute(&self.db)
        .await?;

        Ok(business.into())
    }

    /// Get business by ID
    pub async fn get_by_id(&self, business_id: Uuid, user_id: Uuid) -> Result<BusinessResponse> {
        // Check if user has access
        let has_access = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM businesses b
                LEFT JOIN business_members bm ON b.id = bm.business_id
                WHERE b.id = $1 AND (b.owner_id = $2 OR bm.user_id = $2)
                AND b.deleted_at IS NULL
            )
            "#,
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !has_access {
            return Err(AppError::Forbidden(
                "You don't have access to this business".to_string(),
            ));
        }

        let business = sqlx::query_as::<_, Business>(
            "SELECT * FROM businesses WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        let mut response: BusinessResponse = business.into();
        response.team = self.get_business_members(business_id).await?;

        Ok(response)
    }

    /// List businesses for user
    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        page: i64,
        per_page: i64,
    ) -> Result<(Vec<BusinessResponse>, i64)> {
        let offset = (page - 1) * per_page;

        let businesses = sqlx::query_as::<_, Business>(
            r#"
            SELECT DISTINCT b.* FROM businesses b
            LEFT JOIN business_members bm ON b.id = bm.business_id
            WHERE (b.owner_id = $1 OR bm.user_id = $1)
            AND b.deleted_at IS NULL
            ORDER BY b.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT b.id) FROM businesses b
            LEFT JOIN business_members bm ON b.id = bm.business_id
            WHERE (b.owner_id = $1 OR bm.user_id = $1)
            AND b.deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok((businesses.into_iter().map(Into::into).collect(), total))
    }

    /// Update business
    pub async fn update(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        req: UpdateBusinessRequest,
    ) -> Result<BusinessResponse> {
        // Check ownership
        let is_owner = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM businesses WHERE id = $1 AND owner_id = $2)"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !is_owner {
            return Err(AppError::Forbidden(
                "Only the owner can update this business".to_string(),
            ));
        }

        let business = sqlx::query_as::<_, Business>(
            r#"
            UPDATE businesses 
            SET 
                name = COALESCE($1, name),
                tagline = COALESCE($2, tagline),
                description = COALESCE($3, description),
                industry = COALESCE($4, industry),
                sub_industry = COALESCE($5, sub_industry),
                country_code = COALESCE($6, country_code),
                city = COALESCE($7, city),
                stage = COALESCE($8, stage),
                legal_structure = COALESCE($9, legal_structure),
                registration_number = COALESCE($10, registration_number),
                tax_id = COALESCE($11, tax_id),
                website_url = COALESCE($12, website_url),
                custom_domain = COALESCE($13, custom_domain),
                updated_at = NOW()
            WHERE id = $14
            RETURNING *
            "#,
        )
        .bind(req.name)
        .bind(req.tagline)
        .bind(req.description)
        .bind(req.industry)
        .bind(req.sub_industry)
        .bind(req.country_code)
        .bind(req.city)
        .bind(req.stage)
        .bind(req.legal_structure)
        .bind(req.registration_number)
        .bind(req.tax_id)
        .bind(req.website_url)
        .bind(req.custom_domain)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(business.into())
    }

    /// Delete business (soft delete)
    pub async fn delete(&self, business_id: Uuid, user_id: Uuid) -> Result<()> {
        let is_owner = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM businesses WHERE id = $1 AND owner_id = $2)"
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !is_owner {
            return Err(AppError::Forbidden(
                "Only the owner can delete this business".to_string(),
            ));
        }

        sqlx::query(
            "UPDATE businesses SET status = 'deleted', deleted_at = NOW() WHERE id = $1"
        )
        .bind(business_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get business checklist
    pub async fn get_checklist(&self, business_id: Uuid, user_id: Uuid) -> Result<BusinessChecklistResponse> {
        // Verify access
        let has_access = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM businesses b
                LEFT JOIN business_members bm ON b.id = bm.business_id
                WHERE b.id = $1 AND (b.owner_id = $2 OR bm.user_id = $2)
                AND b.deleted_at IS NULL
            )
            "#,
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !has_access {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        // Get all checklist items with progress using a simpler query
        let rows = sqlx::query(
            r#"
            SELECT 
                cc.id as cat_id,
                cc.name as cat_name,
                cc.description as cat_description,
                ci.id as item_id,
                ci.title,
                ci.description as item_description,
                ci.priority,
                bcp.completed,
                bcp.notes
            FROM checklist_categories cc
            JOIN checklist_items ci ON ci.category_id = cc.id
            LEFT JOIN business_checklist_progress bcp ON bcp.checklist_item_id = ci.id AND bcp.business_id = $1
            WHERE cc.country_code IS NULL OR cc.country_code = (SELECT country_code FROM businesses WHERE id = $1)
            ORDER BY cc.order_index, ci.order_index
            "#
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        // Group by category
        use std::collections::HashMap;
        let mut category_map: HashMap<Uuid, (String, Vec<ChecklistItemResponse>)> = HashMap::new();
        
        for row in rows {
            let cat_id: Uuid = row.get("cat_id");
            let cat_name: String = row.get("cat_name");
            let entry = category_map.entry(cat_id)
                .or_insert_with(|| (cat_name, vec![]));
            
            entry.1.push(ChecklistItemResponse {
                id: row.get("item_id"),
                title: row.get("title"),
                description: row.get("item_description"),
                priority: row.get("priority"),
                completed: row.get::<Option<bool>, _>("completed").unwrap_or(false),
                notes: row.get("notes"),
            });
        }

        let mut categories: Vec<ChecklistCategoryResponse> = category_map
            .into_iter()
            .map(|(_, (name, items))| ChecklistCategoryResponse {
                name,
                progress: 0,
                items,
            })
            .collect();

        // Calculate progress for each category
        for category in &mut categories {
            let completed = category.items.iter().filter(|i| i.completed).count();
            let total = category.items.len();
            category.progress = if total > 0 {
                ((completed as f32 / total as f32) * 100.0) as i32
            } else {
                0
            };
        }

        let overall_progress = if categories.is_empty() {
            0
        } else {
            categories.iter().map(|c| c.progress).sum::<i32>() / categories.len() as i32
        };

        Ok(BusinessChecklistResponse {
            overall_progress,
            categories,
        })
    }

    /// Update checklist item
    pub async fn update_checklist_item(
        &self,
        business_id: Uuid,
        item_id: Uuid,
        user_id: Uuid,
        req: UpdateChecklistItemRequest,
    ) -> Result<()> {
        // Verify access
        let has_access = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM businesses b
                LEFT JOIN business_members bm ON b.id = bm.business_id
                WHERE b.id = $1 AND (b.owner_id = $2 OR bm.user_id = $2)
                AND b.deleted_at IS NULL
            )
            "#,
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !has_access {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        sqlx::query(
            r#"
            INSERT INTO business_checklist_progress (business_id, checklist_item_id, completed, completed_at, completed_by, notes)
            VALUES ($1, $2, $3, CASE WHEN $3 THEN NOW() ELSE NULL END, CASE WHEN $3 THEN $4 ELSE NULL END, $5)
            ON CONFLICT (business_id, checklist_item_id) 
            DO UPDATE SET 
                completed = $3,
                completed_at = CASE WHEN $3 THEN NOW() ELSE NULL END,
                completed_by = CASE WHEN $3 THEN $4 ELSE NULL END,
                notes = $5,
                updated_at = NOW()
            "#,
        )
        .bind(business_id)
        .bind(item_id)
        .bind(req.completed)
        .bind(user_id)
        .bind(req.notes)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get all industries
    pub async fn list_industries(&self) -> Result<Vec<Industry>> {
        let industries = sqlx::query_as::<_, Industry>(
            "SELECT * FROM industries WHERE parent_id IS NULL ORDER BY name"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(industries)
    }

    async fn get_business_members(
        &self,
        business_id: Uuid,
    ) -> Result<Vec<crate::models::BusinessMemberResponse>> {
        let members = sqlx::query(
            r#"
            SELECT 
                bm.user_id,
                bm.role,
                bm.joined_at,
                u.first_name,
                u.last_name,
                u.email
            FROM business_members bm
            JOIN users u ON bm.user_id = u.id
            WHERE bm.business_id = $1
            "#
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await?;

        Ok(members
            .into_iter()
            .map(|row| crate::models::BusinessMemberResponse {
                user_id: row.get("user_id"),
                role: row.get("role"),
                name: format!("{} {}", row.get::<String, _>("first_name"), row.get::<String, _>("last_name")),
                email: row.get("email"),
                joined_at: row.get("joined_at"),
            })
            .collect())
    }
}


