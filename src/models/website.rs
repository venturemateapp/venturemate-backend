use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Website {
    pub id: Uuid,
    pub business_id: Uuid,
    pub subdomain: String,
    pub custom_domain: Option<String>,
    pub domain_status: String,
    pub template_id: Option<Uuid>,
    pub template_config: Value,
    pub global_styles: Value,
    pub status: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Value,
    pub og_image_blob_id: Option<Uuid>,
    pub analytics_config: Value,
    pub published_at: Option<DateTime<Utc>>,
    pub last_modified_at: DateTime<Utc>,
    pub ai_job_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebsitePage {
    pub id: Uuid,
    pub website_id: Uuid,
    pub page_key: String,
    pub name: String,
    pub slug: String,
    pub sections: Value,
    pub is_enabled: bool,
    pub is_homepage: bool,
    pub order_index: i32,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub ai_job_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebsiteTemplate {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_blob_id: Option<Uuid>,
    pub category: Option<String>,
    pub industries: Value,
    pub features: Value,
    pub default_sections: Value,
    pub default_styles: Value,
    pub is_active: bool,
    pub is_premium: bool,
    pub created_at: DateTime<Utc>,
}

// Request/Response structs

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWebsiteRequest {
    pub template_id: Option<Uuid>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebsiteRequest {
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<Value>,
    pub analytics_config: Option<Value>,
    pub template_config: Option<Value>,
    pub global_styles: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePageRequest {
    pub sections: Option<Value>,
    pub is_enabled: Option<bool>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PublishWebsiteRequest {
    pub publish: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectDomainRequest {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebsiteResponse {
    pub id: Uuid,
    pub subdomain: String,
    pub custom_domain: Option<String>,
    pub domain_status: String,
    pub template_id: Option<Uuid>,
    pub status: String,
    pub url: String,
    pub public_url: String,
    pub seo: SeoInfo,
    pub analytics_config: Value,
    pub pages: Vec<PageResponse>,
    pub published_at: Option<DateTime<Utc>>,
    pub last_modified_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageResponse {
    pub id: Uuid,
    pub page_key: String,
    pub name: String,
    pub slug: String,
    pub sections: Value,
    pub is_enabled: bool,
    pub is_homepage: bool,
    pub seo: SeoInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeoInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub features: Value,
    pub is_premium: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainStatusResponse {
    pub domain: Option<String>,
    pub status: String,
    pub dns_records: Vec<Value>,
    pub ssl_status: String,
    pub message: String,
}

// Legacy types for backwards compatibility

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWebsiteRequestLegacy {
    pub template: String,
    pub subdomain: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebsiteRequestLegacy {
    pub custom_domain: Option<String>,
    pub template_config: Option<Value>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWebsitePageRequest {
    pub name: Option<String>,
    pub sections: Option<Value>,
    pub is_enabled: Option<bool>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebsiteResponseLegacy {
    pub id: Uuid,
    pub subdomain: String,
    pub custom_domain: Option<String>,
    pub domain_status: String,
    pub template: String,
    pub status: String,
    pub url: String,
    pub seo: SeoInfoLegacy,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeoInfoLegacy {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub og_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebsitePageResponseLegacy {
    pub id: Uuid,
    pub page_key: String,
    pub name: String,
    pub slug: String,
    pub sections: Value,
    pub is_enabled: bool,
    pub is_homepage: bool,
    pub seo: SeoInfoLegacy,
}

#[derive(Debug, Clone, Serialize)]
pub struct WebsiteTemplateResponseLegacy {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub category: Option<String>,
    pub features: Vec<String>,
    pub is_premium: bool,
}

// Implementations

impl From<Website> for WebsiteResponse {
    fn from(site: Website) -> Self {
        let public_url = format!("https://{}.venture.site", site.subdomain);
        Self {
            id: site.id,
            subdomain: site.subdomain.clone(),
            custom_domain: site.custom_domain.clone(),
            domain_status: site.domain_status.clone(),
            template_id: site.template_id,
            status: site.status.clone(),
            url: public_url.clone(),
            public_url,
            seo: SeoInfo {
                title: site.seo_title,
                description: site.seo_description,
                keywords: site.seo_keywords,
            },
            analytics_config: site.analytics_config,
            pages: vec![],
            published_at: site.published_at,
            last_modified_at: site.last_modified_at,
            created_at: site.created_at,
        }
    }
}

impl From<WebsitePage> for PageResponse {
    fn from(page: WebsitePage) -> Self {
        Self {
            id: page.id,
            page_key: page.page_key,
            name: page.name,
            slug: page.slug,
            sections: page.sections,
            is_enabled: page.is_enabled,
            is_homepage: page.is_homepage,
            seo: SeoInfo {
                title: page.seo_title,
                description: page.seo_description,
                keywords: json!([]),
            },
        }
    }
}

impl From<WebsiteTemplate> for TemplateResponse {
    fn from(template: WebsiteTemplate) -> Self {
        Self {
            id: template.id,
            code: template.code,
            name: template.name,
            description: template.description,
            category: template.category,
            features: template.features,
            is_premium: template.is_premium,
        }
    }
}

// Legacy implementations

impl From<Website> for WebsiteResponseLegacy {
    fn from(site: Website) -> Self {
        let url = format!("https://{}.venture.site", site.subdomain);
        Self {
            id: site.id,
            subdomain: site.subdomain,
            custom_domain: site.custom_domain,
            domain_status: site.domain_status,
            template: site.template_id.map(|id| id.to_string()).unwrap_or_default(),
            status: site.status,
            url,
            seo: SeoInfoLegacy {
                title: site.seo_title,
                description: site.seo_description,
                keywords: serde_json::from_value(site.seo_keywords).unwrap_or_default(),
                og_image_url: None,
            },
            published_at: site.published_at,
            created_at: site.created_at,
        }
    }
}

impl From<WebsitePage> for WebsitePageResponseLegacy {
    fn from(page: WebsitePage) -> Self {
        Self {
            id: page.id,
            page_key: page.page_key,
            name: page.name,
            slug: page.slug,
            sections: page.sections,
            is_enabled: page.is_enabled,
            is_homepage: page.is_homepage,
            seo: SeoInfoLegacy {
                title: page.seo_title,
                description: page.seo_description,
                keywords: vec![],
                og_image_url: None,
            },
        }
    }
}

impl From<WebsiteTemplate> for WebsiteTemplateResponseLegacy {
    fn from(template: WebsiteTemplate) -> Self {
        Self {
            id: template.id,
            code: template.code,
            name: template.name,
            description: template.description,
            thumbnail_url: None,
            preview_url: None,
            category: template.category,
            features: serde_json::from_value(template.features).unwrap_or_default(),
            is_premium: template.is_premium,
        }
    }
}
