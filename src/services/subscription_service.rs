use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateSubscriptionRequest, Invoice, InvoiceResponse, PaymentMethod, PaymentMethodResponse,
    Subscription, SubscriptionPlan, SubscriptionPlanResponse, SubscriptionResponse,
};
use crate::utils::{AppError, Result};

pub struct SubscriptionService {
    db: PgPool,
    _stripe_secret_key: String,
}

impl SubscriptionService {
    pub fn new(db: PgPool, stripe_secret_key: impl Into<String>) -> Self {
        Self {
            db,
            _stripe_secret_key: stripe_secret_key.into(),
        }
    }

    /// Get all subscription plans
    pub async fn get_plans(&self) -> Result<Vec<SubscriptionPlanResponse>> {
        let plans = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            SELECT 
                id, code, name, description, 
                (price_monthly * 100)::bigint as price_monthly,
                (price_yearly * 100)::bigint as price_yearly,
                currency, features, limits, is_active, is_popular, display_order, created_at, updated_at
            FROM subscription_plans 
            WHERE is_active = true 
            ORDER BY display_order
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(plans.into_iter().map(Into::into).collect())
    }

    /// Get plan by code
    pub async fn get_plan_by_code(&self, code: &str) -> Result<SubscriptionPlan> {
        let plan = sqlx::query_as::<_, SubscriptionPlan>(
            r#"
            SELECT 
                id, code, name, description, 
                (price_monthly * 100)::bigint as price_monthly,
                (price_yearly * 100)::bigint as price_yearly,
                currency, features, limits, is_active, is_popular, display_order, created_at, updated_at
            FROM subscription_plans 
            WHERE code = $1 AND is_active = true
            "#
        )
        .bind(code)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Subscription plan not found".to_string()))?;

        Ok(plan)
    }

    /// Get user's current subscription
    pub async fn get_user_subscription(&self, user_id: Uuid) -> Result<Option<SubscriptionResponse>> {
        // Get subscription first
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT * FROM subscriptions
            WHERE user_id = $1
            AND status IN ('active', 'trialing')
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;
        
        if let Some(sub) = subscription {
            // Get plan separately
            let plan = sqlx::query_as::<_, SubscriptionPlan>(
                "SELECT * FROM subscription_plans WHERE id = $1"
            )
            .bind(sub.plan_id)
            .fetch_one(&self.db)
            .await?;

            // Get usage stats
            let businesses_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM businesses WHERE owner_id = $1 AND deleted_at IS NULL"
            )
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

            let ai_generations_count = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM ai_generation_jobs WHERE user_id = $1 AND created_at > $2"
            )
            .bind(user_id)
            .bind(sub.current_period_start.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30)))
            .fetch_one(&self.db)
            .await?;

            let limits: serde_json::Value = plan.limits.clone();
            let businesses_limit = limits.get("businesses").and_then(|v| v.as_i64()).unwrap_or(1);
            let ai_limit = limits.get("ai_generations_per_month").and_then(|v| v.as_i64()).unwrap_or(10);

            Ok(Some(SubscriptionResponse {
                id: sub.id,
                plan: crate::models::PlanInfo {
                    code: plan.code,
                    name: plan.name,
                    price: if sub.billing_interval == Some("year".to_string()) {
                        plan.price_yearly
                    } else {
                        plan.price_monthly
                    },
                    currency: plan.currency.clone(),
                    features: serde_json::from_value(plan.features).unwrap_or_default(),
                    limits: plan.limits,
                },
                status: sub.status,
                billing_interval: sub.billing_interval.unwrap_or_else(|| "month".to_string()),
                current_period_start: sub.current_period_start,
                current_period_end: sub.current_period_end,
                cancel_at_period_end: sub.cancel_at_period_end,
                usage: crate::models::UsageInfo {
                    businesses_used: businesses_count,
                    businesses_limit: if businesses_limit < 0 { 999999 } else { businesses_limit },
                    ai_generations_used: ai_generations_count,
                    ai_generations_limit: if ai_limit < 0 { 999999 } else { ai_limit },
                    storage_used_gb: 0.0, // TODO: Calculate from uploads
                    storage_limit_gb: limits.get("storage_gb").and_then(|v| v.as_f64()).unwrap_or(1.0),
                },
            }))
        } else {
            Ok(None)
        }
    }

    /// Create a new subscription (mock implementation - integrate with Stripe for production)
    pub async fn create_subscription(
        &self,
        user_id: Uuid,
        req: CreateSubscriptionRequest,
    ) -> Result<SubscriptionResponse> {
        let plan = self.get_plan_by_code(&req.plan_code).await?;

        // Check if user already has an active subscription
        let existing = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM subscriptions WHERE user_id = $1 AND status IN ('active', 'trialing')"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(
                "User already has an active subscription".to_string(),
            ));
        }

        // In production: Create Stripe customer and subscription
        // For now, we'll create a mock subscription
        let now = Utc::now();
        let period_end = if req.billing_interval == "year" {
            now + chrono::Duration::days(365)
        } else {
            now + chrono::Duration::days(30)
        };

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (user_id, plan_id, status, billing_interval, current_period_start, current_period_end)
            VALUES ($1, $2, 'active', $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(plan.id)
        .bind(&req.billing_interval)
        .bind(now)
        .bind(period_end)
        .fetch_one(&self.db)
        .await?;

        // Update user's current subscription
        sqlx::query("UPDATE users SET current_subscription_id = $1 WHERE id = $2")
            .bind(subscription.id)
            .bind(user_id)
            .execute(&self.db)
            .await?;

        self.get_user_subscription(user_id)
            .await
            .map(|s| s.unwrap())
    }

    /// Cancel subscription
    pub async fn cancel_subscription(&self, user_id: Uuid, cancel_at_period_end: bool) -> Result<()> {
        let subscription = sqlx::query_as::<_, Subscription>(
            "SELECT * FROM subscriptions WHERE user_id = $1 AND status = 'active'"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(sub) = subscription {
            if cancel_at_period_end {
                sqlx::query(
                    "UPDATE subscriptions SET cancel_at_period_end = true WHERE id = $1"
                )
                .bind(sub.id)
                .execute(&self.db)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE subscriptions SET status = 'cancelled', cancelled_at = NOW() WHERE id = $1"
                )
                .bind(sub.id)
                .execute(&self.db)
                .await?;
            }
        }

        Ok(())
    }

    /// Get user's invoices
    pub async fn get_invoices(&self, user_id: Uuid) -> Result<Vec<InvoiceResponse>> {
        let invoices = sqlx::query_as::<_, Invoice>(
            "SELECT * FROM invoices WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(invoices.into_iter().map(Into::into).collect())
    }

    /// Get user's payment methods
    pub async fn get_payment_methods(&self, user_id: Uuid) -> Result<Vec<PaymentMethodResponse>> {
        let methods = sqlx::query_as::<_, PaymentMethod>(
            "SELECT * FROM payment_methods WHERE user_id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(methods.into_iter().map(Into::into).collect())
    }

    /// Check if user has access to feature
    pub async fn check_feature_access(&self, user_id: Uuid, feature: &str) -> Result<bool> {
        let subscription = self.get_user_subscription(user_id).await?;
        
        if let Some(sub) = subscription {
            // Check based on plan code
            match feature {
                "ai_generation" => Ok(true), // All plans have AI, just different limits
                "custom_domain" => Ok(sub.plan.code != "free"),
                "analytics" => Ok(sub.plan.code == "growth"),
                "crm_tools" => Ok(sub.plan.code == "growth"),
                _ => Ok(false),
            }
        } else {
            // Free tier
            match feature {
                "ai_generation" => Ok(true), // Limited
                "custom_domain" => Ok(false),
                "analytics" => Ok(false),
                _ => Ok(false),
            }
        }
    }

    /// Check if user is within usage limits
    pub async fn check_usage_limits(&self, user_id: Uuid, usage_type: &str) -> Result<bool> {
        let subscription = self.get_user_subscription(user_id).await?;
        
        if let Some(sub) = subscription {
            match usage_type {
                "businesses" => Ok(sub.usage.businesses_used < sub.usage.businesses_limit),
                "ai_generations" => Ok(sub.usage.ai_generations_used < sub.usage.ai_generations_limit),
                "storage" => Ok(sub.usage.storage_used_gb < sub.usage.storage_limit_gb),
                _ => Ok(true),
            }
        } else {
            // Free tier limits
            let count = match usage_type {
                "businesses" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT COUNT(*) FROM businesses WHERE owner_id = $1 AND deleted_at IS NULL"
                    )
                    .bind(user_id)
                    .fetch_one(&self.db)
                    .await?
                }
                "ai_generations" => {
                    sqlx::query_scalar::<_, i64>(
                        "SELECT COUNT(*) FROM ai_generation_jobs WHERE user_id = $1 AND created_at > NOW() - INTERVAL '30 days'"
                    )
                    .bind(user_id)
                    .fetch_one(&self.db)
                    .await?
                }
                _ => 0,
            };
            
            match usage_type {
                "businesses" => Ok(count < 1),
                "ai_generations" => Ok(count < 10),
                _ => Ok(true),
            }
        }
    }
}
