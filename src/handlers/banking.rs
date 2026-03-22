// Phase 2: Banking & Payments Handler
use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::utils::get_user_id;


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/banking")
            .route("/dashboard", web::get().to(get_dashboard))
            .route("/accounts", web::get().to(list_accounts))
            .route("/accounts", web::post().to(create_account))
            .route("/accounts/{id}", web::get().to(get_account))
            .route("/accounts/{id}", web::delete().to(delete_account))
            .route("/transactions", web::get().to(list_transactions))
            .route("/integrations", web::get().to(list_integrations))
            .route("/integrations", web::post().to(connect_integration))
            .route("/integrations/{id}", web::delete().to(disconnect_integration))
            .route("/invoices", web::get().to(list_invoices))
            .route("/invoices", web::post().to(create_invoice))
            .route("/invoices/{id}", web::get().to(get_invoice))
            .route("/invoices/{id}/send", web::post().to(send_invoice))
            .route("/invoices/{id}/record-payment", web::post().to(record_payment))
            .route("/supported-banks", web::get().to(get_supported_banks))
            .route("/supported-providers", web::get().to(get_supported_providers))
    );
}

// Dashboard
async fn get_dashboard(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Get accounts
    let accounts = sqlx::query_as::<_, BankAccount>(
        "SELECT * FROM bank_accounts WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    // Calculate total balance
    let total_balance: i64 = accounts.iter()
        .filter_map(|a| a.balance)
        .sum();
    
    let account_responses: Vec<BankAccountResponse> = accounts.into_iter()
        .map(|a| BankAccountResponse {
            id: a.id,
            bank_name: a.bank_name,
            account_type: a.account_type,
            account_number_masked: mask_account_number(&a.account_number),
            account_name: a.account_name,
            currency: a.currency.clone(),
            balance: a.balance.map(|b| b as f64 / 100.0),
            status: a.status,
            is_verified: a.is_verified,
        })
        .collect();
    
    // Get recent transactions
    let transactions = sqlx::query_as::<_, PaymentTransaction>(
        "SELECT * FROM payment_transactions WHERE business_id = $1 ORDER BY created_at DESC LIMIT 10"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();
    
    let transaction_responses: Vec<TransactionResponse> = transactions.into_iter()
        .map(|t| TransactionResponse {
            id: t.id,
            transaction_type: t.transaction_type,
            amount: t.amount as f64 / 100.0,
            currency: t.currency,
            description: t.description,
            reference: t.reference,
            counterparty_name: t.counterparty_name,
            status: t.status,
            processed_at: t.processed_at,
            created_at: t.created_at,
        })
        .collect();
    
    // Get invoice stats
    let pending_invoices = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM invoices WHERE business_id = $1 AND status IN ('draft', 'sent', 'viewed')"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let overdue_invoices = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM invoices WHERE business_id = $1 AND status = 'overdue'"
    )
    .bind(business_id)
    .fetch_one(pool.get_ref())
    .await
    .unwrap_or(0);
    
    let dashboard = BankingDashboard {
        total_balance: total_balance as f64 / 100.0,
        currency: "USD".to_string(),
        accounts: account_responses,
        recent_transactions: transaction_responses,
        pending_invoices,
        overdue_invoices,
        monthly_revenue: 0.0,
        monthly_expenses: 0.0,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(dashboard)))
}

// Bank Accounts
async fn list_accounts(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let accounts = sqlx::query_as::<_, BankAccount>(
        "SELECT * FROM bank_accounts WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let responses: Vec<BankAccountResponse> = accounts.into_iter()
        .map(|a| BankAccountResponse {
            id: a.id,
            bank_name: a.bank_name,
            account_type: a.account_type,
            account_number_masked: mask_account_number(&a.account_number),
            account_name: a.account_name,
            currency: a.currency,
            balance: a.balance.map(|b| b as f64 / 100.0),
            status: a.status,
            is_verified: a.is_verified,
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn create_account(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateBankAccountRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let account = sqlx::query_as::<_, BankAccount>(
        r#"INSERT INTO bank_accounts (business_id, bank_name, bank_code, account_type, account_number, account_name, currency, country_code, branch_code, swift_code, iban, status, metadata)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending', '{}')
           RETURNING *"#
    )
    .bind(business_id)
    .bind(&body.bank_name)
    .bind(&body.bank_code)
    .bind(&body.account_type)
    .bind(&body.account_number)
    .bind(&body.account_name)
    .bind(&body.currency)
    .bind(&body.country_code)
    .bind(&body.branch_code)
    .bind(&body.swift_code)
    .bind(&body.iban)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(account)))
}

async fn get_account(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let account_id = path.into_inner();
    
    let account = sqlx::query_as::<_, BankAccount>(
        "SELECT * FROM bank_accounts WHERE id = $1 AND business_id = $2"
    )
    .bind(account_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match account {
        Some(a) => Ok(HttpResponse::Ok().json(ApiResponse::success(a))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "Account not found"))),
    }
}

async fn delete_account(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let account_id = path.into_inner();
    
    sqlx::query("DELETE FROM bank_accounts WHERE id = $1 AND business_id = $2")
        .bind(account_id)
        .bind(business_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

// Transactions
async fn list_transactions(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let offset = query.offset();
    let limit = query.limit();
    
    let transactions = sqlx::query_as::<_, PaymentTransaction>(
        "SELECT * FROM payment_transactions WHERE business_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(business_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(transactions)))
}

// Integrations
async fn list_integrations(
    pool: web::Data<PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    let integrations = sqlx::query_as::<_, BankIntegration>(
        "SELECT * FROM bank_integrations WHERE business_id = $1 ORDER BY created_at DESC"
    )
    .bind(business_id)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(integrations)))
}

async fn connect_integration(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<ConnectBankIntegrationRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // In production, encrypt these values
    let api_key_encrypted = body.api_key.clone();
    let webhook_secret = body.webhook_secret.clone().unwrap_or_default();
    
    let integration = sqlx::query_as::<_, BankIntegration>(
        r#"INSERT INTO bank_integrations (business_id, provider, provider_type, status, api_key_encrypted, webhook_secret_encrypted, settings)
           VALUES ($1, $2, 'payment_gateway', 'connected', $3, $4, COALESCE($5, '{}'))
           RETURNING *"#
    )
    .bind(business_id)
    .bind(&body.provider)
    .bind(&api_key_encrypted)
    .bind(&webhook_secret)
    .bind(body.settings.clone())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(integration)))
}

async fn disconnect_integration(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let integration_id = path.into_inner();
    
    sqlx::query("UPDATE bank_integrations SET status = 'disconnected' WHERE id = $1 AND business_id = $2")
        .bind(integration_id)
        .bind(business_id)
        .execute(pool.get_ref())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::NoContent().finish())
}

// Invoices
async fn list_invoices(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let offset = query.offset();
    let limit = query.limit();
    
    let invoices = sqlx::query_as::<_, BankInvoice>(
        "SELECT * FROM invoices WHERE business_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(business_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    let responses: Vec<BankInvoiceResponse> = invoices.into_iter()
        .map(|i| BankInvoiceResponse {
            id: i.id,
            invoice_number: i.invoice_number,
            customer_name: i.customer_name,
            customer_email: i.customer_email,
            amount_total: i.amount_total as f64 / 100.0,
            currency: i.currency,
            status: i.status,
            issue_date: i.issue_date,
            due_date: i.due_date,
            paid_at: i.paid_at,
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(responses)))
}

async fn create_invoice(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    body: web::Json<CreateInvoiceRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    
    // Calculate totals
    let mut subtotal = 0i64;
    for item in &body.line_items {
        let item_total = (item.quantity * item.unit_price as f64) as i64;
        subtotal += item_total;
    }
    let tax = (subtotal as f64 * 0.0) as i64; // Add tax calculation later
    let total = subtotal + tax;
    
    let invoice_number = generate_invoice_number();
    let line_items_json = serde_json::to_value(&body.line_items).unwrap_or_default();
    
    let invoice = sqlx::query_as::<_, BankInvoice>(
        r#"INSERT INTO invoices (business_id, invoice_number, customer_name, customer_email, customer_address, issue_date, due_date, amount_subtotal, amount_tax, amount_total, currency, status, notes, line_items)
           VALUES ($1, $2, $3, $4, $5, NOW(), $6, $7, $8, $9, 'USD', 'draft', $10, $11)
           RETURNING *"#
    )
    .bind(business_id)
    .bind(&invoice_number)
    .bind(&body.customer_name)
    .bind(&body.customer_email)
    .bind(&body.customer_address)
    .bind(body.due_date)
    .bind(subtotal)
    .bind(tax)
    .bind(total)
    .bind(&body.notes)
    .bind(line_items_json)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Created().json(ApiResponse::success(invoice)))
}

async fn get_invoice(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let invoice_id = path.into_inner();
    
    let invoice = sqlx::query_as::<_, BankInvoice>(
        "SELECT * FROM invoices WHERE id = $1 AND business_id = $2"
    )
    .bind(invoice_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match invoice {
        Some(i) => Ok(HttpResponse::Ok().json(ApiResponse::success(i))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "BankInvoice not found"))),
    }
}

async fn send_invoice(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let invoice_id = path.into_inner();
    
    let invoice = sqlx::query_as::<_, BankInvoice>(
        "UPDATE invoices SET status = 'sent' WHERE id = $1 AND business_id = $2 RETURNING *"
    )
    .bind(invoice_id)
    .bind(business_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match invoice {
        Some(i) => Ok(HttpResponse::Ok().json(ApiResponse::success(i))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "BankInvoice not found"))),
    }
}

async fn record_payment(
    pool: web::Data<PgPool>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<RecordPaymentRequest>,
) -> Result<HttpResponse> {
    let user_id = get_user_id(&req).ok_or_else(|| actix_web::error::ErrorUnauthorized("Unauthorized"))?;
    let business_id = get_default_business_id(&pool, user_id).await?;
    let invoice_id = path.into_inner();
    
    let invoice = sqlx::query_as::<_, BankInvoice>(
        r#"UPDATE invoices 
           SET status = 'paid', paid_at = NOW(), payment_method = $3
           WHERE id = $1 AND business_id = $2
           RETURNING *"#
    )
    .bind(invoice_id)
    .bind(business_id)
    .bind(&body.payment_method)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match invoice {
        Some(i) => Ok(HttpResponse::Ok().json(ApiResponse::success(i))),
        None => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error("NOT_FOUND", "BankInvoice not found"))),
    }
}

// Supported options
async fn get_supported_banks() -> Result<HttpResponse> {
    let banks = vec![
        SupportedBank {
            code: "chase".to_string(),
            name: "JPMorgan Chase".to_string(),
            country: "US".to_string(),
            supported_features: vec!["wire".to_string(), "ach".to_string(), "checking".to_string()],
        },
        SupportedBank {
            code: "bofa".to_string(),
            name: "Bank of America".to_string(),
            country: "US".to_string(),
            supported_features: vec!["wire".to_string(), "ach".to_string(), "savings".to_string()],
        },
        SupportedBank {
            code: "gtb".to_string(),
            name: "Guaranty Trust Bank".to_string(),
            country: "NG".to_string(),
            supported_features: vec!["transfer".to_string(), "mobile".to_string()],
        },
        SupportedBank {
            code: "absa".to_string(),
            name: "Absa Bank".to_string(),
            country: "ZA".to_string(),
            supported_features: vec!["transfer".to_string(), "business".to_string()],
        },
    ];
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(banks)))
}

async fn get_supported_providers() -> Result<HttpResponse> {
    let providers = vec![
        SupportedPaymentProvider {
            code: "stripe".to_string(),
            name: "Stripe".to_string(),
            countries: vec!["US".to_string(), "CA".to_string(), "GB".to_string(), "EU".to_string()],
            methods: vec!["card".to_string(), "bank_transfer".to_string()],
            features: vec!["subscriptions".to_string(), "invoicing".to_string()],
        },
        SupportedPaymentProvider {
            code: "flutterwave".to_string(),
            name: "Flutterwave".to_string(),
            countries: vec!["NG".to_string(), "GH".to_string(), "KE".to_string(), "ZA".to_string()],
            methods: vec!["card".to_string(), "mobile_money".to_string(), "bank_transfer".to_string()],
            features: vec!["multicurrency".to_string(), "payouts".to_string()],
        },
        SupportedPaymentProvider {
            code: "paystack".to_string(),
            name: "Paystack".to_string(),
            countries: vec!["NG".to_string(), "GH".to_string(), "ZA".to_string()],
            methods: vec!["card".to_string(), "bank_transfer".to_string(), "ussd".to_string()],
            features: vec!["subscriptions".to_string(), "transfers".to_string()],
        },
    ];
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(providers)))
}

// Helper functions
fn mask_account_number(account_number: &str) -> String {
    if account_number.len() <= 4 {
        return "****".to_string();
    }
    format!("****{}", &account_number[account_number.len()-4..])
}

fn generate_invoice_number() -> String {
    use chrono::Local;
    let timestamp = Local::now().timestamp();
    format!("INV-{}", timestamp)
}

async fn get_default_business_id(pool: &PgPool, user_id: Uuid) -> Result<Uuid, actix_web::Error> {
    let business_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM businesses WHERE owner_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
    .ok_or_else(|| actix_web::error::ErrorBadRequest("No business found"))?;
    
    Ok(business_id)
}
