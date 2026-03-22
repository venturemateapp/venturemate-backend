use sqlx::{PgPool, Row};
use uuid::Uuid;

use serde_json::{json, Value};

use crate::models::{
    Website, WebsitePage, WebsiteTemplate, CreateWebsiteRequest, UpdateWebsiteRequest,
    UpdatePageRequest, PublishWebsiteRequest, WebsiteResponse, PageResponse,
};
use crate::services::file_storage_service::FileStorageService;
use crate::utils::{AppError, Result};

/// Website Builder Service
pub struct WebsiteService {
    db: PgPool,
    _storage: FileStorageService,
}

impl WebsiteService {
    pub fn new(db: PgPool) -> Self {
        let storage = FileStorageService::new(db.clone());
        Self { db, _storage: storage }
    }

    // ============================================================================
    // WEBSITES
    // ============================================================================

    /// Create a website for a business
    pub async fn create_website(
        &self,
        business_id: Uuid,
        _user_id: Uuid,
        req: CreateWebsiteRequest,
    ) -> Result<WebsiteResponse> {
        // Check if business already has a website
        let existing = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM websites WHERE business_id = $1)"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        if existing {
            return Err(AppError::Conflict("Business already has a website".to_string()));
        }

        // Get business info for subdomain
        let business = sqlx::query(
            "SELECT slug, name, tagline, description, brand_colors FROM businesses WHERE id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        let slug: String = business.get("slug");
        let name: String = business.get("name");
        let tagline: Option<String> = business.get("tagline");
        let description: Option<String> = business.get("description");
        let brand_colors: Option<Value> = business.get("brand_colors");

        // Generate subdomain from slug
        let subdomain = generate_subdomain(&slug);

        // Get template
        let template_id = req.template_id.unwrap_or_else(|| {
            // Default to first active template
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::new_v4())
        });

        // Create website
        let website = sqlx::query_as::<_, Website>(
            r#"
            INSERT INTO websites (
                business_id, subdomain, template_id, template_config,
                global_styles, seo_title, seo_description
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(business_id)
        .bind(&subdomain)
        .bind(template_id)
        .bind(json!({}))
        .bind(brand_colors.unwrap_or_else(|| json!({})))
        .bind(req.seo_title.unwrap_or_else(|| name.clone()))
        .bind(req.seo_description.or(description).unwrap_or_default())
        .fetch_one(&self.db)
        .await?;

        // Create default pages based on template
        self.create_default_pages(website.id, template_id, &name, tagline.as_deref()).await?;

        Ok(website.into())
    }

    /// Get website by business ID
    pub async fn get_website(&self, business_id: Uuid, user_id: Uuid) -> Result<WebsiteResponse> {
        // Verify access
        self.verify_business_access(business_id, user_id).await?;

        let website = sqlx::query_as::<_, Website>(
            r#"
            SELECT w.* FROM websites w
            WHERE w.business_id = $1
            "#
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Website not found".to_string()))?;

        let website_id = website.id;
        let mut response: WebsiteResponse = website.into();
        
        // Get pages
        let pages = self.get_pages(website_id).await?;
        response.pages = pages;

        Ok(response)
    }

    /// Update website
    pub async fn update_website(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        req: UpdateWebsiteRequest,
    ) -> Result<WebsiteResponse> {
        self.verify_business_access(business_id, user_id).await?;

        let website = sqlx::query_as::<_, Website>(
            r#"
            UPDATE websites
            SET 
                seo_title = COALESCE($1, seo_title),
                seo_description = COALESCE($2, seo_description),
                seo_keywords = COALESCE($3, seo_keywords),
                analytics_config = COALESCE($4, analytics_config),
                template_config = COALESCE($5, template_config),
                global_styles = COALESCE($6, global_styles),
                last_modified_at = NOW()
            WHERE business_id = $7
            RETURNING *
            "#
        )
        .bind(req.seo_title)
        .bind(req.seo_description)
        .bind(req.seo_keywords)
        .bind(req.analytics_config)
        .bind(req.template_config)
        .bind(req.global_styles)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(website.into())
    }

    /// Delete website
    pub async fn delete_website(&self, business_id: Uuid, user_id: Uuid) -> Result<()> {
        self.verify_business_access(business_id, user_id).await?;

        sqlx::query("DELETE FROM websites WHERE business_id = $1")
            .bind(business_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ============================================================================
    // PAGES
    // ============================================================================

    /// Get all pages for a website
    async fn get_pages(&self, website_id: Uuid) -> Result<Vec<PageResponse>> {
        let pages = sqlx::query_as::<_, WebsitePage>(
            r#"
            SELECT * FROM website_pages
            WHERE website_id = $1
            ORDER BY order_index, created_at
            "#
        )
        .bind(website_id)
        .fetch_all(&self.db)
        .await?;

        Ok(pages.into_iter().map(|p| p.into()).collect())
    }

    /// Get single page
    pub async fn get_page(
        &self,
        business_id: Uuid,
        page_id: String,
        user_id: Uuid,
    ) -> Result<PageResponse> {
        self.verify_business_access(business_id, user_id).await?;

        let page = sqlx::query_as::<_, WebsitePage>(
            r#"
            SELECT p.* FROM website_pages p
            JOIN websites w ON p.website_id = w.id
            WHERE w.business_id = $1 AND p.page_key = $2
            "#
        )
        .bind(business_id)
        .bind(&page_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;

        Ok(page.into())
    }

    /// Update page
    pub async fn update_page(
        &self,
        business_id: Uuid,
        page_id: String,
        user_id: Uuid,
        req: UpdatePageRequest,
    ) -> Result<PageResponse> {
        self.verify_business_access(business_id, user_id).await?;

        let page = sqlx::query_as::<_, WebsitePage>(
            r#"
            UPDATE website_pages p
            SET 
                sections = COALESCE($1, sections),
                is_enabled = COALESCE($2, is_enabled),
                seo_title = COALESCE($3, seo_title),
                seo_description = COALESCE($4, seo_description),
                updated_at = NOW()
            FROM websites w
            WHERE p.website_id = w.id 
              AND w.business_id = $5 
              AND p.page_key = $6
            RETURNING p.*
            "#
        )
        .bind(req.sections)
        .bind(req.is_enabled)
        .bind(req.seo_title)
        .bind(req.seo_description)
        .bind(business_id)
        .bind(&page_id)
        .fetch_one(&self.db)
        .await?;

        // Update website last_modified
        sqlx::query("UPDATE websites SET last_modified_at = NOW() WHERE business_id = $1")
            .bind(business_id)
            .execute(&self.db)
            .await?;

        Ok(page.into())
    }

    // ============================================================================
    // PUBLISHING
    // ============================================================================

    /// Publish website
    pub async fn publish_website(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        _req: PublishWebsiteRequest,
    ) -> Result<WebsiteResponse> {
        self.verify_business_access(business_id, user_id).await?;

        let website = sqlx::query_as::<_, Website>(
            r#"
            UPDATE websites
            SET status = 'published', published_at = NOW(), last_modified_at = NOW()
            WHERE business_id = $1
            RETURNING *
            "#
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(website.into())
    }

    /// Unpublish website
    pub async fn unpublish_website(&self, business_id: Uuid, user_id: Uuid) -> Result<WebsiteResponse> {
        self.verify_business_access(business_id, user_id).await?;

        let website = sqlx::query_as::<_, Website>(
            r#"
            UPDATE websites
            SET status = 'unpublished', last_modified_at = NOW()
            WHERE business_id = $1
            RETURNING *
            "#
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(website.into())
    }

    // ============================================================================
    // DOMAIN MANAGEMENT
    // ============================================================================

    /// Connect custom domain
    pub async fn connect_domain(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        domain: String,
    ) -> Result<WebsiteResponse> {
        self.verify_business_access(business_id, user_id).await?;

        // Validate domain format
        if !is_valid_domain(&domain) {
            return Err(AppError::Validation("Invalid domain format".to_string()));
        }

        let website = sqlx::query_as::<_, Website>(
            r#"
            UPDATE websites
            SET custom_domain = $1, domain_status = 'pending_dns', last_modified_at = NOW()
            WHERE business_id = $2
            RETURNING *
            "#
        )
        .bind(&domain)
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        Ok(website.into())
    }

    /// Check domain status
    pub async fn check_domain_status(&self, business_id: Uuid, user_id: Uuid) -> Result<DomainStatus> {
        self.verify_business_access(business_id, user_id).await?;

        let website = sqlx::query_as::<_, Website>(
            "SELECT * FROM websites WHERE business_id = $1"
        )
        .bind(business_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Website not found".to_string()))?;

        let dns_records = vec![
            json!({
                "type": "CNAME",
                "name": "@",
                "value": format!("{}.venture.site", website.subdomain),
                "required": true
            }),
            json!({
                "type": "CNAME",
                "name": "www",
                "value": format!("{}.venture.site", website.subdomain),
                "required": false
            }),
        ];

        let domain_status = website.domain_status.clone();
        Ok(DomainStatus {
            domain: website.custom_domain,
            status: domain_status.clone(),
            dns_records,
            ssl_status: if domain_status == "active" { "active" } else { "pending" }.to_string(),
            message: "Add the DNS records above to your domain registrar".to_string(),
        })
    }

    // ============================================================================
    // TEMPLATES
    // ============================================================================

    /// List available templates
    pub async fn list_templates(&self) -> Result<Vec<TemplateResponse>> {
        let templates = sqlx::query_as::<_, WebsiteTemplate>(
            r#"
            SELECT 
                id, code, name, description, category,
                industries, features, default_sections, default_styles,
                is_active, is_premium, created_at
            FROM website_templates
            WHERE is_active = true
            ORDER BY name
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(templates.into_iter().map(|t| t.into()).collect())
    }

    /// Get template by code
    pub async fn get_template(&self, code: &str) -> Result<TemplateResponse> {
        let template = sqlx::query_as::<_, WebsiteTemplate>(
            "SELECT * FROM website_templates WHERE code = $1 AND is_active = true"
        )
        .bind(code)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Template not found".to_string()))?;

        Ok(template.into())
    }

    // ============================================================================
    // GENERATION
    // ============================================================================

    /// Generate website using AI
    pub async fn generate_website(
        &self,
        business_id: Uuid,
        user_id: Uuid,
        _template_code: Option<String>,
    ) -> Result<WebsiteResponse> {
        self.verify_business_access(business_id, user_id).await?;

        // Get business data
        let business = sqlx::query(
            "SELECT name, tagline, description, industry FROM businesses WHERE id = $1"
        )
        .bind(business_id)
        .fetch_one(&self.db)
        .await?;

        let name: String = business.get("name");
        let tagline: Option<String> = business.get("tagline");
        let description: Option<String> = business.get("description");

        // Create or update website
        let website = match self.get_website(business_id, user_id).await {
            Ok(w) => w,
            Err(_) => {
                self.create_website(
                    business_id,
                    user_id,
                    CreateWebsiteRequest {
                        template_id: None,
                        seo_title: Some(name.clone()),
                        seo_description: description.clone(),
                    },
                )
                .await?
            }
        };

        // Generate page content using AI (simplified)
        let hero_section = json!({
            "id": "hero",
            "type": "hero",
            "content": {
                "headline": tagline.unwrap_or_else(|| format!("Welcome to {}", name)),
                "subheadline": description.unwrap_or_default(),
                "cta_text": "Learn More",
                "cta_link": "#about"
            }
        });

        // Update home page with generated content
        self.update_page(
            business_id,
            "home".to_string(),
            user_id,
            UpdatePageRequest {
                sections: Some(json!([hero_section])),
                is_enabled: None,
                seo_title: None,
                seo_description: None,
            },
        )
        .await?;

        Ok(website)
    }

    // ============================================================================
    // HELPERS
    // ============================================================================

    async fn verify_business_access(&self, business_id: Uuid, user_id: Uuid) -> Result<()> {
        let has_access = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM businesses b
                LEFT JOIN business_members bm ON b.id = bm.business_id AND bm.user_id = $2
                WHERE b.id = $1 AND (b.owner_id = $2 OR bm.user_id = $2)
            )
            "#
        )
        .bind(business_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if !has_access {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }

        Ok(())
    }

    async fn create_default_pages(
        &self,
        website_id: Uuid,
        template_id: Uuid,
        _business_name: &str,
        _tagline: Option<&str>,
    ) -> Result<()> {
        // Get template sections
        let template = sqlx::query(
            "SELECT default_sections FROM website_templates WHERE id = $1"
        )
        .bind(template_id)
        .fetch_optional(&self.db)
        .await?;

        let default_sections: Value = template
            .and_then(|t| t.get("default_sections"))
            .unwrap_or_else(|| {
                json!([
                    {"id": "hero", "type": "hero"},
                    {"id": "about", "type": "about"},
                    {"id": "contact", "type": "contact"}
                ])
            });

        // Create pages
        let pages = vec![
            ("home", "Home", "/", true, json!(default_sections)),
            ("about", "About", "/about", true, json!([{"id": "about", "type": "about"}])),
            ("contact", "Contact", "/contact", true, json!([{"id": "contact", "type": "contact"}])),
        ];

        for (i, (key, name, slug, enabled, sections)) in pages.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO website_pages (
                    website_id, page_key, name, slug, sections,
                    is_enabled, is_homepage, order_index
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (website_id, page_key) DO NOTHING
                "#
            )
            .bind(website_id)
            .bind(*key)
            .bind(*name)
            .bind(*slug)
            .bind(sections)
            .bind(*enabled)
            .bind(key == &"home")
            .bind(i as i32)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }
}

/// Generate subdomain from slug
fn generate_subdomain(slug: &str) -> String {
    slug.to_lowercase()
        .replace(" ", "-")
        .replace("_", "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
}

/// Validate domain format
fn is_valid_domain(domain: &str) -> bool {
    // Basic validation - should contain at least one dot and no spaces
    domain.contains('.') && !domain.contains(' ') && domain.len() > 3
}

/// Domain status response
#[derive(Debug, Clone, serde::Serialize)]
pub struct DomainStatus {
    pub domain: Option<String>,
    pub status: String,
    pub dns_records: Vec<Value>,
    pub ssl_status: String,
    pub message: String,
}

/// Template response
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub features: Value,
    pub is_premium: bool,
}

impl From<WebsiteTemplate> for TemplateResponse {
    fn from(t: WebsiteTemplate) -> Self {
        Self {
            id: t.id,
            code: t.code,
            name: t.name,
            description: t.description,
            category: t.category,
            features: t.features,
            is_premium: t.is_premium,
        }
    }
}
