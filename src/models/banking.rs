// Phase 2: Banking & Payments Models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ============================================
// BANK ACCOUNT MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct BankAccount {
    pub id: Uuid,
    pub business_id: Uuid,
    pub bank_name: String,
    pub bank_code: Option<String>,
    pub account_type: String, // checking, savings, merchant
    pub account_number: String,
    pub account_name: String,
    pub currency: String,
    pub country_code: String,
    pub branch_code: Option<String>,
    pub swift_code: Option<String>,
    pub iban: Option<String>,
    pub balance: Option<i64>, // stored in cents/smallest unit
    pub status: String, // pending, active, frozen, closed
    pub is_verified: bool,
    pub opened_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct BankIntegration {
    pub id: Uuid,
    pub business_id: Uuid,
    pub provider: String, // stripe, flutterwave, paystack, etc.
    pub provider_type: String, // payment_gateway, bank_api, wallet
    pub status: String, // pending, connected, disconnected, error
    pub api_key_encrypted: Option<String>,
    pub webhook_secret_encrypted: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub settings: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// PAYMENT MODELS
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub business_id: Uuid,
    pub bank_account_id: Option<Uuid>,
    pub integration_id: Option<Uuid>,
    pub transaction_type: String, // incoming, outgoing, transfer
    pub amount: i64, // stored in cents/smallest unit
    pub currency: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub external_reference: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
    pub status: String, // pending, processing, completed, failed, reversed
    pub failure_reason: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct BankInvoice {
    pub id: Uuid,
    pub business_id: Uuid,
    pub invoice_number: String,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub issue_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub amount_subtotal: i64,
    pub amount_tax: i64,
    pub amount_total: i64,
    pub currency: String,
    pub status: String, // draft, sent, viewed, paid, overdue, cancelled
    pub payment_method: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub line_items: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[sqlx(rename_all = "snake_case")]
pub struct BankPaymentMethod {
    pub id: Uuid,
    pub business_id: Uuid,
    pub method_type: String, // card, bank_transfer, mobile_money, wallet
    pub provider: String,
    pub display_name: String,
    pub last_four: Option<String>,
    pub expiry_month: Option<i32>,
    pub expiry_year: Option<i32>,
    pub is_default: bool,
    pub is_verified: bool,
    pub status: String, // active, expired, revoked
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// REQUEST/RESPONSE MODELS
// ============================================

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateBankAccountRequest {
    #[validate(length(min = 1))]
    pub bank_name: String,
    pub bank_code: Option<String>,
    #[validate(length(min = 1))]
    pub account_type: String,
    #[validate(length(min = 1))]
    pub account_number: String,
    #[validate(length(min = 1))]
    pub account_name: String,
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
    #[validate(length(min = 2, max = 2))]
    pub country_code: String,
    pub branch_code: Option<String>,
    pub swift_code: Option<String>,
    pub iban: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectBankIntegrationRequest {
    pub provider: String,
    pub api_key: String,
    pub webhook_secret: Option<String>,
    pub settings: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateInvoiceRequest {
    #[validate(length(min = 1))]
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_address: Option<String>,
    pub due_date: DateTime<Utc>,
    pub line_items: Vec<BankInvoiceLineItem>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BankInvoiceLineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: i64, // in cents
    pub tax_rate: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordPaymentRequest {
    pub invoice_id: Uuid,
    pub amount: i64,
    pub payment_method: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BankAccountResponse {
    pub id: Uuid,
    pub bank_name: String,
    pub account_type: String,
    pub account_number_masked: String,
    pub account_name: String,
    pub currency: String,
    pub balance: Option<f64>,
    pub status: String,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BankInvoiceResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub amount_total: f64,
    pub currency: String,
    pub status: String,
    pub issue_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub transaction_type: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub counterparty_name: Option<String>,
    pub status: String,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BankingDashboard {
    pub total_balance: f64,
    pub currency: String,
    pub accounts: Vec<BankAccountResponse>,
    pub recent_transactions: Vec<TransactionResponse>,
    pub pending_invoices: i64,
    pub overdue_invoices: i64,
    pub monthly_revenue: f64,
    pub monthly_expenses: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SupportedBank {
    pub code: String,
    pub name: String,
    pub country: String,
    pub supported_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SupportedPaymentProvider {
    pub code: String,
    pub name: String,
    pub countries: Vec<String>,
    pub methods: Vec<String>,
    pub features: Vec<String>,
}
