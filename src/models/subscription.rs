use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

// Type alias for monetary values (using i64 cents to avoid floating point issues)
pub type Money = i64;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub price_monthly: Money,
    pub price_yearly: Money,
    pub currency: String,
    pub features: Value,
    pub limits: Value,
    pub is_active: bool,
    pub is_popular: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub status: String,
    pub billing_interval: Option<String>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub usage_this_period: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub user_id: Uuid,
    pub invoice_number: String,
    pub description: Option<String>,
    pub subtotal: i64,
    pub tax_amount: i64,
    pub total: i64,
    pub currency: String,
    pub status: String,
    pub stripe_invoice_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub paid_at: Option<DateTime<Utc>>,
    pub pdf_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_payment_method_id: String,
    pub payment_type: String,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub billing_name: Option<String>,
    pub billing_email: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// Request/Response structs

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub plan_code: String,
    #[serde(alias = "interval")]
    pub billing_interval: String,
    pub payment_method_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub plan_code: Option<String>,
    pub cancel_at_period_end: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub plan: PlanInfo,
    pub status: String,
    pub billing_interval: String,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub usage: UsageInfo,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanInfo {
    pub code: String,
    pub name: String,
    pub price: Money,
    pub currency: String,
    pub features: Vec<String>,
    pub limits: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageInfo {
    pub businesses_used: i64,
    pub businesses_limit: i64,
    pub ai_generations_used: i64,
    pub ai_generations_limit: i64,
    pub storage_used_gb: f64,
    pub storage_limit_gb: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionPlanResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub price_monthly: i64,
    pub price_yearly: i64,
    pub currency: String,
    pub features: Vec<String>,
    pub limits: Value,
    pub is_popular: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub description: Option<String>,
    pub total: i64,
    pub currency: String,
    pub status: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub paid_at: Option<DateTime<Utc>>,
    pub pdf_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentMethodResponse {
    pub id: Uuid,
    pub payment_type: String,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub billing_name: Option<String>,
    pub is_default: bool,
}

impl From<SubscriptionPlan> for SubscriptionPlanResponse {
    fn from(plan: SubscriptionPlan) -> Self {
        Self {
            id: plan.id,
            code: plan.code,
            name: plan.name,
            description: plan.description,
            price_monthly: plan.price_monthly,
            price_yearly: plan.price_yearly,
            currency: plan.currency,
            features: serde_json::from_value(plan.features).unwrap_or_default(),
            limits: plan.limits,
            is_popular: plan.is_popular,
        }
    }
}

impl From<Invoice> for InvoiceResponse {
    fn from(invoice: Invoice) -> Self {
        Self {
            id: invoice.id,
            invoice_number: invoice.invoice_number,
            description: invoice.description,
            total: invoice.total,
            currency: invoice.currency,
            status: invoice.status,
            invoice_date: invoice.invoice_date,
            due_date: invoice.due_date,
            paid_at: invoice.paid_at,
            pdf_url: invoice.pdf_url,
        }
    }
}

impl From<PaymentMethod> for PaymentMethodResponse {
    fn from(pm: PaymentMethod) -> Self {
        Self {
            id: pm.id,
            payment_type: pm.payment_type.clone(),
            card_brand: pm.card_brand.clone(),
            card_last4: pm.card_last4.clone(),
            card_exp_month: pm.card_exp_month,
            card_exp_year: pm.card_exp_year,
            billing_name: pm.billing_name.clone(),
            is_default: pm.is_default,
        }
    }
}
