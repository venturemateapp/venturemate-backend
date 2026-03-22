//! Media Marketplace Service
//!
//! Manages freelancer marketplace, orders, reviews, and AI content generation.

use crate::utils::{AppError, Result};
use crate::models::marketplace::{
    AiContent, AiContentCalendarItem, AiContentCalendarResponse, AiContentResponse, AiContentStatus, CategoryInfo,
    CreateOrderRequest, CreateOrderResponse, GenerateAiContentRequest,
    GenerateAiContentResponse, ListServiceListingsRequest, ListServiceListingsResponse,
    MarketplaceMessage, MarketplaceMessageResponse, MarketplaceOrder, MarketplaceReview, OrderResponse,
    OrderServiceInfo, OrderStatus, ReviewResponse, ScheduleAiContentRequest, ServicePricingInfo,
    SendProviderMessageRequest, ServiceCategory, ServiceListing, ServiceListingResponse, UpdateContentRequest,
    build_social_content_prompt,
};
use crate::services::ai_service::AIService;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub struct MarketplaceService {
    db: PgPool,
    ai_service: Arc<AIService>,
}

impl MarketplaceService {
    pub fn new(db: PgPool, ai_service: Arc<AIService>) -> Self {
        Self { db, ai_service }
    }

    // =================================================================================
    // Service Listings
    // =================================================================================

    /// List service listings
    pub async fn list_listings(&self, req: ListServiceListingsRequest) -> Result<ListServiceListingsResponse> {
        let listings: Vec<ServiceListing> = if let Some(category) = req.category {
            if req.featured_only {
                sqlx::query_as(
                    "SELECT * FROM service_listings WHERE service_category = $1 AND status = 'active' AND featured = true ORDER BY rating DESC"
                )
                .bind(&category)
                .fetch_all(&self.db)
                .await
                .map_err(AppError::Database)?
            } else {
                sqlx::query_as(
                    "SELECT * FROM service_listings WHERE service_category = $1 AND status = 'active' ORDER BY rating DESC"
                )
                .bind(&category)
                .fetch_all(&self.db)
                .await
                .map_err(AppError::Database)?
            }
        } else {
            sqlx::query_as(
                "SELECT * FROM service_listings WHERE status = 'active' ORDER BY featured DESC, rating DESC LIMIT 20"
            )
            .fetch_all(&self.db)
            .await
            .map_err(AppError::Database)?
        };

        // Get provider info for each listing
        let mut responses = Vec::new();
        for listing in listings {
            let provider: (String, Option<String>, Option<String>) = sqlx::query_as(
                "SELECT u.email, up.first_name, up.last_name FROM users u LEFT JOIN user_profiles up ON u.id = up.user_id WHERE u.id = $1"
            )
            .bind(listing.provider_id)
            .fetch_one(&self.db)
            .await
            .map_err(AppError::Database)?;

            responses.push(ServiceListingResponse {
                id: listing.id,
                provider_id: listing.provider_id,
                provider_name: format!("{} {}", provider.1.unwrap_or_default(), provider.2.unwrap_or_default()).trim().to_string(),
                provider_avatar: None,
                service_category: listing.service_category,
                service_name: listing.service_name,
                description: listing.description.unwrap_or_default(),
                pricing: serde_json::from_value(listing.pricing.unwrap_or_default()).unwrap_or(ServicePricingInfo {
                    base_price: rust_decimal::Decimal::new(0, 0),
                    currency: "USD".to_string(),
                    price_tiers: vec![],
                }),
                delivery_time_days: listing.delivery_time_days.unwrap_or(7),
                portfolio_urls: listing.portfolio_urls.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
                rating: listing.rating.map(|r| r.to_string().parse().unwrap_or(0.0)).unwrap_or(0.0),
                review_count: listing.review_count,
                is_verified: listing.is_verified,
                featured: listing.featured,
            });
        }

        // Get category counts
        let categories: Vec<(String, i64)> = sqlx::query_as(
            "SELECT service_category, COUNT(*) FROM service_listings WHERE status = 'active' GROUP BY service_category"
        )
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        let category_info: Vec<CategoryInfo> = categories.into_iter()
            .filter_map(|(code, count)| {
                code.parse::<ServiceCategory>().ok().map(|cat| CategoryInfo {
                    code: cat.as_str().to_string(),
                    name: cat.display_name().to_string(),
                    price_range: cat.price_range().to_string(),
                    count,
                })
            })
            .collect();

        Ok(ListServiceListingsResponse {
            listings: responses,
            categories: category_info,
        })
    }

    // =================================================================================
    // Orders
    // =================================================================================

    /// Create order
    pub async fn create_order(&self, buyer_id: Uuid, req: CreateOrderRequest) -> Result<CreateOrderResponse> {
        // Get service details
        let service: Option<ServiceListing> = sqlx::query_as(
            "SELECT * FROM service_listings WHERE id = $1 AND status = 'active'"
        )
        .bind(req.service_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some(service) = service else {
            return Err(AppError::NotFound("Service not found".to_string()));
        };

        // Get buyer's business
        let business_id: Uuid = sqlx::query_scalar(
            "SELECT id FROM businesses WHERE user_id = $1 LIMIT 1"
        )
        .bind(buyer_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        let pricing: ServicePricingInfo = serde_json::from_value(service.pricing.unwrap_or_default())
            .unwrap_or(ServicePricingInfo {
                base_price: rust_decimal::Decimal::new(0, 0),
                currency: "USD".to_string(),
                price_tiers: vec![],
            });

        let order_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO marketplace_orders (id, business_id, service_id, buyer_id, provider_id, requirements, attachments, total_amount, currency, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', NOW())"
        )
        .bind(order_id)
        .bind(business_id)
        .bind(req.service_id)
        .bind(buyer_id)
        .bind(service.provider_id)
        .bind(&req.requirements)
        .bind(serde_json::to_value(&req.attachments.unwrap_or_default()).unwrap_or_default())
        .bind(pricing.base_price)
        .bind(&pricing.currency)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(CreateOrderResponse {
            order_id,
            status: "pending".to_string(),
            total_amount: pricing.base_price,
            message: "Order created successfully".to_string(),
        })
    }

    /// List orders for a business
    pub async fn list_orders(&self, business_id: Uuid) -> Result<Vec<OrderResponse>> {
        let orders: Vec<(MarketplaceOrder, String, String)> = sqlx::query_as(
            "SELECT o.*, s.service_name, s.service_category FROM marketplace_orders o
             JOIN service_listings s ON o.service_id = s.id
             WHERE o.business_id = $1 ORDER BY o.created_at DESC"
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(orders.into_iter().map(|(o, service_name, category)| OrderResponse {
            id: o.id,
            business_id: o.business_id,
            service: OrderServiceInfo {
                id: o.service_id,
                name: service_name,
                category,
            },
            buyer_id: o.buyer_id,
            provider_id: o.provider_id,
            requirements: o.requirements.unwrap_or_default(),
            total_amount: o.total_amount,
            currency: o.currency,
            status: o.status,
            delivery_date: o.delivery_date,
            delivered_at: o.delivered_at,
            created_at: o.created_at,
        }).collect())
    }

    // =================================================================================
    // Reviews
    // =================================================================================

    /// Submit review
    pub async fn submit_review(&self, order_id: Uuid, rating: i32, review_text: Option<String>) -> Result<ReviewResponse> {
        let review_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO marketplace_reviews (id, order_id, business_id, provider_id, rating, review_text, created_at)
             SELECT $1, $2, business_id, provider_id, $3, $4, NOW() FROM marketplace_orders WHERE id = $2"
        )
        .bind(review_id)
        .bind(order_id)
        .bind(rating)
        .bind(review_text.clone())
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        // Update provider rating
        sqlx::query(
            "UPDATE service_listings SET 
                rating = (SELECT AVG(rating)::decimal FROM marketplace_reviews WHERE provider_id = service_listings.provider_id),
                review_count = (SELECT COUNT(*) FROM marketplace_reviews WHERE provider_id = service_listings.provider_id)
             WHERE provider_id = (SELECT provider_id FROM marketplace_orders WHERE id = $1)"
        )
        .bind(order_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(ReviewResponse {
            id: review_id,
            order_id,
            rating,
            review_text,
            is_public: true,
            response_text: None,
            created_at: chrono::Utc::now(),
        })
    }

    // =================================================================================
    // Messages
    // =================================================================================

    /// Send message
    pub async fn send_message(&self, order_id: Uuid, sender_id: Uuid, req: SendProviderMessageRequest) -> Result<MarketplaceMessageResponse> {
        let message_id = Uuid::new_v4();

        // Get sender name
        let sender_name: String = sqlx::query_scalar(
            "SELECT COALESCE(up.first_name || ' ' || up.last_name, u.email) 
             FROM users u LEFT JOIN user_profiles up ON u.id = up.user_id WHERE u.id = $1"
        )
        .bind(sender_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        sqlx::query(
            "INSERT INTO marketplace_messages (id, order_id, sender_id, message, attachment_url, created_at)
             VALUES ($1, $2, $3, $4, $5, NOW())"
        )
        .bind(message_id)
        .bind(order_id)
        .bind(sender_id)
        .bind(&req.message)
        .bind(req.attachment_url.clone())
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(MarketplaceMessageResponse {
            id: message_id,
            sender_id,
            sender_name,
            message: req.message,
            attachment_url: req.attachment_url,
            is_read: false,
            created_at: chrono::Utc::now(),
        })
    }

    /// Get messages for an order
    pub async fn get_messages(&self, order_id: Uuid) -> Result<Vec<MarketplaceMessageResponse>> {
        let messages: Vec<(MarketplaceMessage, String)> = sqlx::query_as(
            "SELECT m.*, COALESCE(up.first_name || ' ' || up.last_name, u.email) as sender_name
             FROM marketplace_messages m
             JOIN users u ON m.sender_id = u.id
             LEFT JOIN user_profiles up ON u.id = up.user_id
             WHERE m.order_id = $1 ORDER BY m.created_at ASC"
        )
        .bind(order_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(messages.into_iter().map(|(m, sender_name)| MarketplaceMessageResponse {
            id: m.id,
            sender_id: m.sender_id,
            sender_name,
            message: m.message,
            attachment_url: m.attachment_url,
            is_read: m.is_read,
            created_at: m.created_at,
        }).collect())
    }

    // =================================================================================
    // AI Content Generation
    // =================================================================================

    /// Generate AI content
    pub async fn generate_content(&self, req: GenerateAiContentRequest) -> Result<GenerateAiContentResponse> {
        let generation_id = Uuid::new_v4();

        // Get business details
        let business: Option<(String, String, Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT name, industry, tagline, target_audience FROM businesses WHERE id = $1"
        )
        .bind(req.business_id)
        .fetch_optional(&self.db)
        .await
        .map_err(AppError::Database)?;

        let Some((name, industry, tagline, target_audience)) = business else {
            return Err(AppError::NotFound("Business not found".to_string()));
        };

        // Queue the generation
        tokio::spawn({
            let db = self.db.clone();
            let ai_service = self.ai_service.clone();
            let business_id = req.business_id;
            let name = name.clone();
            let industry = industry.clone();
            let tagline = tagline.clone().unwrap_or_default();
            let target_audience = target_audience.clone().unwrap_or_default();
            let content_type = req.content_type.clone();
            let days = req.days;

            async move {
                if let Err(e) = Self::generate_content_background(
                    db, ai_service, business_id, name, industry, tagline, target_audience, content_type, days
                ).await {
                    error!("AI content generation failed: {}", e);
                }
            }
        });

        Ok(GenerateAiContentResponse {
            generation_id,
            status: "processing".to_string(),
            estimated_seconds: 30,
        })
    }

    /// Background content generation
    async fn generate_content_background(
        db: PgPool,
        ai_service: Arc<AIService>,
        business_id: Uuid,
        name: String,
        industry: String,
        tagline: String,
        target_audience: String,
        content_type: String,
        days: i32,
    ) -> Result<()> {
        let prompt = build_social_content_prompt(
            &name, &industry, &tagline, &target_audience, "products/services", days
        );

        let response = ai_service.generate_text(
            "You are a social media expert. Create engaging, platform-appropriate content.",
            &prompt,
            3000
        ).await.map_err(|e| AppError::AiGeneration(e.to_string()))?;

        // Parse and save content
        if let Ok(content_items) = serde_json::from_str::<Vec<serde_json::Value>>(&response) {
            for item in content_items {
                let content = item.get("caption").and_then(|v| v.as_str()).unwrap_or("");
                let platform = item.get("platform").and_then(|v| v.as_str()).unwrap_or("instagram");
                let hashtags = item.get("hashtags").cloned().unwrap_or_default();

                let _ = sqlx::query(
                    "INSERT INTO ai_content (business_id, content_type, platform, generated_content, hashtags, status, created_at)
                     VALUES ($1, $2, $3, $4, $5, 'draft', NOW())"
                )
                .bind(business_id)
                .bind(&content_type)
                .bind(platform)
                .bind(content)
                .bind(hashtags)
                .execute(&db)
                .await;
            }
        }

        Ok(())
    }

    /// Get content calendar
    pub async fn get_content_calendar(&self, business_id: Uuid) -> Result<AiContentCalendarResponse> {
        let content: Vec<AiContent> = sqlx::query_as(
            "SELECT * FROM ai_content WHERE business_id = $1 ORDER BY created_at DESC LIMIT 100"
        )
        .bind(business_id)
        .fetch_all(&self.db)
        .await
        .map_err(AppError::Database)?;

        let items = content.into_iter().map(|c| AiContentCalendarItem {
            id: c.id,
            date: c.created_at.format("%Y-%m-%d").to_string(),
            platform: c.platform.unwrap_or_default(),
            content_type: c.content_type,
            content: c.generated_content,
            hashtags: c.hashtags.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            image_url: c.image_url,
            status: c.status,
            scheduled_date: c.scheduled_date,
        }).collect();

        Ok(AiContentCalendarResponse {
            content_calendar: items,
        })
    }

    /// Update content
    pub async fn update_content(&self, content_id: Uuid, _business_id: Uuid, req: UpdateContentRequest) -> Result<AiContentResponse> {
        sqlx::query(
            "UPDATE ai_content SET generated_content = $1, hashtags = $2, updated_at = NOW() WHERE id = $3"
        )
        .bind(&req.content)
        .bind(serde_json::to_value(&req.hashtags.unwrap_or_default()).unwrap_or_default())
        .bind(content_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        let content: AiContent = sqlx::query_as(
            "SELECT * FROM ai_content WHERE id = $1"
        )
        .bind(content_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(AiContentResponse {
            id: content.id,
            content_type: content.content_type,
            platform: content.platform,
            generated_content: content.generated_content,
            image_url: content.image_url,
            hashtags: content.hashtags.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            scheduled_date: content.scheduled_date,
            status: content.status,
            created_at: content.created_at,
        })
    }

    /// Schedule content
    pub async fn schedule_content(&self, content_id: Uuid, _business_id: Uuid, req: ScheduleAiContentRequest) -> Result<AiContentResponse> {
        sqlx::query(
            "UPDATE ai_content SET scheduled_date = $1, status = 'scheduled', updated_at = NOW() WHERE id = $2"
        )
        .bind(req.scheduled_date)
        .bind(content_id)
        .execute(&self.db)
        .await
        .map_err(AppError::Database)?;

        let content: AiContent = sqlx::query_as(
            "SELECT * FROM ai_content WHERE id = $1"
        )
        .bind(content_id)
        .fetch_one(&self.db)
        .await
        .map_err(AppError::Database)?;

        Ok(AiContentResponse {
            id: content.id,
            content_type: content.content_type,
            platform: content.platform,
            generated_content: content.generated_content,
            image_url: content.image_url,
            hashtags: content.hashtags.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default(),
            scheduled_date: content.scheduled_date,
            status: content.status,
            created_at: content.created_at,
        })
    }
}
