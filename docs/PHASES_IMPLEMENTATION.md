# VentureMate - Step-by-Step Implementation Guide

> Complete beginner-friendly guide to building VentureMate from MVP to Scale

## 📋 Table of Contents

1. [Development Environment Setup](#1-development-environment-setup)
2. [Phase 1: MVP (Months 1-3)](#2-phase-1-mvp-months-1-3)
3. [Phase 2: Growth (Months 4-6)](#3-phase-2-growth-months-4-6)
4. [Phase 3: Scale (Months 7-12)](#4-phase-3-scale-months-7-12)
5. [Testing Strategy](#5-testing-strategy)
6. [Common Issues & Solutions](#6-common-issues--solutions)

---

## 1. Development Environment Setup

### 1.1 Prerequisites Installation

```bash
# 1. Install Rust (follow prompts)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Verify installation
rustc --version  # Should show 1.75 or higher
cargo --version

# 3. Install required tools
cargo install sqlx-cli --features native-tls
cargo install cargo-watch

# 4. Install PostgreSQL (Ubuntu/Debian)
sudo apt update
sudo apt install postgresql postgresql-contrib libpq-dev

# Or macOS with Homebrew
brew install postgresql
brew services start postgresql

# 5. Install Redis
sudo apt install redis-server  # Ubuntu
brew install redis             # macOS

# 6. Install Node.js (for frontend)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# 7. Verify all installations
psql --version
redis-cli ping  # Should return PONG
node --version  # Should be v20+
```

### 1.2 Project Setup

```bash
# 1. Clone the repository
git clone <your-repo-url>
cd bcknd

# 2. Set up environment variables
cp .env.example .env

# Edit .env with your settings:
# DATABASE_URL=postgres://username:password@localhost/venturemate
# REDIS_URL=redis://127.0.0.1:6379
# JWT_SECRET=your-super-secret-jwt-key-at-least-32-chars
# OPENAI_API_KEY=your-openai-api-key

# 3. Create database
createdb venturemate

# 4. Run migrations
cargo sqlx migrate run

# 5. Build the project
cargo build

# 6. Run the server
cargo run

# Server should start on http://localhost:8080
# Test: curl http://localhost:8080/api/v1/health
```

### 1.3 Project Structure Explanation

```
bcknd/
├── Cargo.toml              # Rust dependencies
├── .env                    # Environment variables (gitignored)
├── migrations/             # Database migrations
│   ├── 001_create_users.sql
│   └── 002_create_businesses.sql
├── src/
│   ├── main.rs            # Application entry point
│   ├── lib.rs             # Library exports
│   ├── config/            # Configuration management
│   ├── db/                # Database connection
│   ├── models/            # Data structures
│   ├── services/          # Business logic
│   ├── handlers/          # API endpoints
│   ├── middleware/        # Auth, logging, etc.
│   └── utils/             # Helper functions
└── docs/                  # Documentation
```

---

## 2. Phase 1: MVP (Months 1-3)

### 🎯 Goal
Build the core founder journey: Signup → Onboarding → Business Creation → AI Generation → Dashboard

### Week 1-2: Authentication System

#### Step 1: Create User Model

**File: `src/models/user.rs`**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub password_hash: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub country_code: String,
    pub status: String,
    pub onboarding_completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}
```

#### Step 2: Create Database Migration

**File: `migrations/001_create_users.sql`**

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified_at TIMESTAMPTZ,
    password_hash VARCHAR(255),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    avatar_url TEXT,
    phone VARCHAR(20),
    country_code CHAR(2) NOT NULL DEFAULT 'ZA',
    status VARCHAR(20) DEFAULT 'active',
    onboarding_completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
```

Run migration:
```bash
cargo sqlx migrate run
```

#### Step 3: Implement Auth Service

**File: `src/services/auth.rs`**

```rust
use crate::models::user::{AuthResponse, CreateUserRequest, LoginRequest, User};
use crate::utils::error::AppError;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
}

pub struct AuthService {
    db: PgPool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(db: PgPool, jwt_secret: String) -> Self {
        Self { db, jwt_secret }
    }

    // Register new user
    pub async fn register(&self, req: CreateUserRequest) -> Result<AuthResponse, AppError> {
        // Check if email exists
        let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", req.email)
            .fetch_optional(&self.db)
            .await?;
        
        if existing.is_some() {
            return Err(AppError::BadRequest("Email already registered".to_string()));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        // Create user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, first_name, last_name, country_code)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.country_code)
        .fetch_one(&self.db)
        .await?;

        // Generate tokens
        let tokens = self.generate_tokens(&user.id.to_string())?;

        Ok(AuthResponse {
            user,
            access_token: tokens.0,
            refresh_token: tokens.1,
            expires_in: 900, // 15 minutes
        })
    }

    // Login user
    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&req.email)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash.as_ref().ok_or(
            AppError::Unauthorized("Invalid credentials".to_string()
        ))?)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        
        Argon2::default()
            .verify_password(req.password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Unauthorized("Invalid credentials".to_string()))?;

        // Generate tokens
        let tokens = self.generate_tokens(&user.id.to_string())?;

        Ok(AuthResponse {
            user,
            access_token: tokens.0,
            refresh_token: tokens.1,
            expires_in: 900,
        })
    }

    fn generate_tokens(&self, user_id: &str) -> Result<(String, String), AppError> {
        let now = Utc::now();
        
        // Access token (15 min)
        let access_claims = Claims {
            sub: user_id.to_string(),
            exp: (now + Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
        };
        
        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        ).map_err(|e| AppError::Internal(e.to_string()))?;

        // Refresh token (7 days)
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            exp: (now + Duration::days(7)).timestamp(),
            iat: now.timestamp(),
        };
        
        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        ).map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((access_token, refresh_token))
    }
}
```

#### Step 4: Create Auth Handler

**File: `src/handlers/auth.rs`**

```rust
use actix_web::{post, web, HttpResponse};
use crate::models::user::{CreateUserRequest, LoginRequest};
use crate::services::auth::AuthService;
use crate::utils::response::ApiResponse;

#[post("/auth/register")]
async fn register(
    req: web::Json<CreateUserRequest>,
    auth_service: web::Data<AuthService>,
) -> HttpResponse {
    match auth_service.register(req.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}

#[post("/auth/login")]
async fn login(
    req: web::Json<LoginRequest>,
    auth_service: web::Data<AuthService>,
) -> HttpResponse {
    match auth_service.login(req.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::success(response)),
        Err(e) => e.into_response(),
    }
}
```

#### Step 5: Update main.rs

**File: `src/main.rs`**

```rust
mod config;
mod db;
mod handlers;
mod models;
mod services;
mod utils;
mod middleware;

use actix_web::{web, App, HttpServer};
use crate::config::Config;
use crate::db::init_db_pool;
use crate::services::auth::AuthService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load config
    let config = Config::from_env();
    
    // Initialize database
    let db_pool = init_db_pool(&config.database_url).await
        .expect("Failed to connect to database");
    
    // Initialize services
    let auth_service = web::Data::new(AuthService::new(
        db_pool.clone(),
        config.jwt_secret.clone(),
    ));
    
    println!("🚀 Server starting on http://0.0.0.0:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(auth_service.clone())
            .configure(handlers::init_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### Week 3-4: Onboarding Flow

#### Step 1: Create Onboarding Models

**File: `src/models/onboarding.rs`**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OnboardingSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub current_step: String,
    pub progress_percentage: i32,
    pub status: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct IdeaIntakeRequest {
    pub session_id: Uuid,
    pub business_idea: String,
    pub problem_statement: String,
    pub target_customers: String,
    pub country_code: String,
    pub city: String,
    pub founder_type: String,
    pub team_size: i32,
}

#[derive(Debug, Serialize)]
pub struct IdeaIntakeResponse {
    pub session_id: Uuid,
    pub ai_analysis: AiAnalysis,
    pub next_step: String,
    pub progress_percentage: i32,
}

#[derive(Debug, Serialize)]
pub struct AiAnalysis {
    pub industry: String,
    pub sub_industry: String,
    pub market_size: String,
    pub complexity: String,
    pub estimated_launch_time: String,
    pub suggested_business_models: Vec<String>,
}
```

#### Step 2: Create Business Model

**File: `src/models/business.rs`**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Business {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub slug: String,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub industry: String,
    pub sub_industry: Option<String>,
    pub country_code: String,
    pub city: Option<String>,
    pub status: String,
    pub stage: String,
    pub health_score: Option<i32>,
    pub logo_url: Option<String>,
    pub brand_colors: Option<serde_json::Value>,
    pub website_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBusinessRequest {
    pub name: String,
    pub industry: String,
    pub country_code: String,
    pub description: Option<String>,
}
```

#### Step 3: Implement Onboarding Service

**File: `src/services/onboarding.rs`**

```rust
use crate::models::onboarding::{AiAnalysis, IdeaIntakeRequest, IdeaIntakeResponse, OnboardingSession};
use crate::utils::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct OnboardingService {
    db: PgPool,
}

impl OnboardingService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn start_session(&self, user_id: Uuid) -> Result<OnboardingSession, AppError> {
        let session = sqlx::query_as::<_, OnboardingSession>(
            r#"
            INSERT INTO onboarding_sessions (user_id, current_step, progress_percentage, status, data)
            VALUES ($1, 'idea_intake', 0, 'active', '{}')
            RETURNING *
            "#
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(session)
    }

    pub async fn submit_idea_intake(
        &self,
        req: IdeaIntakeRequest,
    ) -> Result<IdeaIntakeResponse, AppError> {
        // Save the idea data
        let data = serde_json::json!({
            "business_idea": req.business_idea,
            "problem_statement": req.problem_statement,
            "target_customers": req.target_customers,
            "country_code": req.country_code,
            "city": req.city,
            "founder_type": req.founder_type,
            "team_size": req.team_size,
        });

        sqlx::query(
            r#"
            UPDATE onboarding_sessions 
            SET current_step = 'founder_profile', 
                progress_percentage = 25,
                data = $1,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(&data)
        .bind(req.session_id)
        .execute(&self.db)
        .await?;

        // Call AI to analyze the idea (simplified - just mock for now)
        let analysis = AiAnalysis {
            industry: "Technology".to_string(),
            sub_industry: "SaaS".to_string(),
            market_size: "$10B Global".to_string(),
            complexity: "Medium".to_string(),
            estimated_launch_time: "6-8 weeks".to_string(),
            suggested_business_models: vec![
                "Subscription".to_string(),
                "Freemium".to_string(),
            ],
        };

        Ok(IdeaIntakeResponse {
            session_id: req.session_id,
            ai_analysis: analysis,
            next_step: "founder_profile".to_string(),
            progress_percentage: 25,
        })
    }

    pub async fn complete_onboarding(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Uuid, AppError> {
        // Get session data
        let session = sqlx::query_as::<_, OnboardingSession>(
            "SELECT * FROM onboarding_sessions WHERE id = $1"
        )
        .bind(session_id)
        .fetch_one(&self.db)
        .await?;

        // Extract data from session
        let data = session.data;
        let business_name = data
            .get("business_idea")
            .and_then(|v| v.as_str())
            .unwrap_or("My Business")
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");

        // Create business
        let business = sqlx::query_as::<_, crate::models::business::Business>(
            r#"
            INSERT INTO businesses (owner_id, name, slug, industry, country_code, status, stage)
            VALUES ($1, $2, $3, $4, $5, 'active', 'idea')
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(&business_name)
        .bind(slugify(&business_name))
        .bind("Technology")
        .bind(data.get("country_code").and_then(|v| v.as_str()).unwrap_or("ZA"))
        .fetch_one(&self.db)
        .await?;

        // Update user onboarding status
        sqlx::query("UPDATE users SET onboarding_completed = true WHERE id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await?;

        Ok(business.id)
    }
}

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
}
```

### Week 5-6: AI Integration (OpenAI/Claude)

#### Step 1: Add AI Service

**File: `src/services/ai.rs`**

```rust
use crate::utils::error::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct AIService {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
}

impl AIService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
        }
    }

    pub async fn generate_business_plan(
        &self,
        business_idea: &str,
        industry: &str,
        country: &str,
    ) -> Result<String, AppError> {
        let prompt = format!(
            r#"You are an expert business consultant. Create a comprehensive business plan for the following startup idea:

BUSINESS IDEA: {}
INDUSTRY: {}
TARGET COUNTRY: {}

Generate a detailed business plan with the following sections:
1. Executive Summary
2. Problem Statement
3. Solution
4. Market Analysis (include TAM, SAM, SOM for {})
5. Business Model & Revenue Streams
6. Competitive Analysis
7. Go-to-Market Strategy
8. Financial Projections (3-year)
9. Team Requirements
10. Risk Analysis & Mitigation

Format the response in Markdown."#,
            business_idea, industry, country, country
        );

        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4000,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("AI request failed: {}", e)))?;

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse AI response: {}", e)))?;

        Ok(claude_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default())
    }

    pub async fn analyze_business_idea(
        &self,
        idea: &str,
        problem: &str,
        target: &str,
    ) -> Result<serde_json::Value, AppError> {
        let prompt = format!(
            r#"Analyze this business idea and return ONLY a JSON object:

IDEA: {}
PROBLEM: {}
TARGET CUSTOMERS: {}

Respond with ONLY this JSON structure (no markdown, no explanation):
{{
    "industry": "Primary industry category",
    "sub_industry": "More specific category",
    "market_size": "Brief market size estimate",
    "complexity": "low|medium|high",
    "estimated_launch_time": "X weeks/months",
    "suggested_business_models": ["model1", "model2", "model3"],
    "viability_score": 1-10
}}"#,
            idea, problem, target
        );

        // Similar implementation to above...
        // Parse the JSON response and return
        
        Ok(json!({
            "industry": "Technology",
            "sub_industry": "SaaS",
            "market_size": "$10B",
            "complexity": "medium",
            "estimated_launch_time": "8-12 weeks",
            "suggested_business_models": ["Subscription", "Freemium"],
            "viability_score": 8
        }))
    }
}
```

### Week 7-8: Business Dashboard & Core UI

#### Step 1: Create Business Handlers

**File: `src/handlers/business.rs`**

```rust
use actix_web::{get, post, patch, delete, web, HttpRequest, HttpResponse};
use crate::middleware::auth::AuthMiddleware;
use crate::models::business::{Business, CreateBusinessRequest};
use crate::services::business::BusinessService;
use crate::utils::response::ApiResponse;
use uuid::Uuid;

#[get("/businesses")]
async fn list_businesses(
    req: HttpRequest,
    business_service: web::Data<BusinessService>,
    _: AuthMiddleware,
) -> HttpResponse {
    let user_id = req.extensions().get::<Uuid>().unwrap();
    
    match business_service.list_by_user(*user_id).await {
        Ok(businesses) => HttpResponse::Ok().json(ApiResponse::success(businesses)),
        Err(e) => e.into_response(),
    }
}

#[post("/businesses")]
async fn create_business(
    req: HttpRequest,
    body: web::Json<CreateBusinessRequest>,
    business_service: web::Data<BusinessService>,
    _: AuthMiddleware,
) -> HttpResponse {
    let user_id = req.extensions().get::<Uuid>().unwrap();
    
    match business_service.create(*user_id, body.into_inner()).await {
        Ok(business) => HttpResponse::Created().json(ApiResponse::success(business)),
        Err(e) => e.into_response(),
    }
}

#[get("/businesses/{id}")]
async fn get_business(
    path: web::Path<Uuid>,
    business_service: web::Data<BusinessService>,
    _: AuthMiddleware,
) -> HttpResponse {
    match business_service.get_by_id(path.into_inner()).await {
        Ok(business) => HttpResponse::Ok().json(ApiResponse::success(business)),
        Err(e) => e.into_response(),
    }
}

#[patch("/businesses/{id}")]
async fn update_business(
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
    business_service: web::Data<BusinessService>,
    _: AuthMiddleware,
) -> HttpResponse {
    match business_service.update(path.into_inner(), body.into_inner()).await {
        Ok(business) => HttpResponse::Ok().json(ApiResponse::success(business)),
        Err(e) => e.into_response(),
    }
}

#[delete("/businesses/{id}")]
async fn delete_business(
    path: web::Path<Uuid>,
    business_service: web::Data<BusinessService>,
    _: AuthMiddleware,
) -> HttpResponse {
    match business_service.delete(path.into_inner()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => e.into_response(),
    }
}
```

### Week 9-10: Website Builder (Basic)

#### Step 1: Website Templates

Create simple HTML templates stored in database:

```rust
// Template structure
const STARTUP_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{business_name}}</title>
    <style>
        :root {
            --primary: {{primary_color}};
            --secondary: {{secondary_color}};
        }
        /* Template styles */
    </style>
</head>
<body>
    <header>
        <h1>{{headline}}</h1>
        <p>{{subheadline}}</p>
    </header>
    <!-- More sections -->
</body>
</html>
"#;
```

### Week 11-12: Testing & MVP Launch

#### Run Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With coverage
cargo tarpaulin

# Load testing (install k6)
k6 run load-tests/basic.js
```

---

## 3. Phase 2: Growth (Months 4-6)

### 🎯 Goal
Add operations, finance, and marketing capabilities

### Month 4: Payment Integration (Stripe)

#### Step 1: Stripe Setup

```rust
// src/services/payment.rs
use stripe::{Client, CreateCustomer, CreateSubscription, Customer, Subscription};

pub struct PaymentService {
    client: Client,
}

impl PaymentService {
    pub fn new(secret_key: String) -> Self {
        Self {
            client: Client::new(secret_key),
        }
    }

    pub async fn create_customer(
        &self,
        email: &str,
        name: &str,
    ) -> Result<Customer, AppError> {
        let create_customer = CreateCustomer {
            email: Some(email),
            name: Some(name),
            ..Default::default()
        };

        Customer::create(&self.client, create_customer)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
    ) -> Result<Subscription, AppError> {
        let mut params = CreateSubscription::new(customer_id);
        params.items = Some(vec![CreateSubscriptionItems {
            price: Some(price_id.to_string()),
            ..Default::default()
        }]);

        Subscription::create(&self.client, params)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}
```

### Month 5: CRM & Operations

#### Simple CRM Data Model

```sql
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id),
    name VARCHAR(200) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    company VARCHAR(200),
    status VARCHAR(50) DEFAULT 'lead', -- lead, prospect, customer, churned
    source VARCHAR(100),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE deals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id),
    contact_id UUID REFERENCES contacts(id),
    title VARCHAR(255) NOT NULL,
    value DECIMAL(12, 2),
    currency CHAR(3) DEFAULT 'USD',
    stage VARCHAR(50) DEFAULT 'prospecting', -- prospecting, negotiation, closed_won, closed_lost
    probability INTEGER CHECK (probability >= 0 AND probability <= 100),
    expected_close_date DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Month 6: Social Media & Content AI

#### AI Content Generation

```rust
pub async fn generate_social_posts(
    &self,
    business_name: &str,
    industry: &str,
    platform: &str, // twitter, linkedin, instagram
    count: i32,
) -> Result<Vec<String>, AppError> {
    let prompt = format!(
        "Generate {} {} posts for a {} business called '{}'. \
         Make them engaging and relevant to the African market.",
        count, platform, industry, business_name
    );
    
    // Call AI and parse response...
}
```

---

## 4. Phase 3: Scale (Months 7-12)

### 🎯 Goal
Ecosystem features, advanced analytics, and marketplace

### Month 7-8: Health Score Algorithm

```rust
pub fn calculate_health_score(business: &Business) -> HealthScore {
    let mut score = 0;
    let mut breakdown = HashMap::new();

    // Compliance (20 points)
    let compliance = if business.registration_number.is_some() { 20 } else { 0 };
    score += compliance;
    breakdown.insert("compliance", compliance);

    // Digital Presence (20 points)
    let digital = if business.website_url.is_some() { 15 } else { 0 };
    let digital = digital + if business.logo_url.is_some() { 5 } else { 0 };
    score += digital;
    breakdown.insert("digital_presence", digital);

    // Documentation (20 points)
    let docs = 20; // Simplified
    score += docs;
    breakdown.insert("documentation", docs);

    // Financial (20 points)
    let finance = 15; // Simplified
    score += finance;
    breakdown.insert("financial", finance);

    // Market (20 points)
    let market = 15; // Simplified
    score += market;
    breakdown.insert("market", market);

    HealthScore {
        overall: score,
        breakdown,
        recommendations: generate_recommendations(&breakdown),
    }
}
```

### Month 9-10: Marketplace

#### Service Provider Model

```rust
#[derive(Debug, FromRow)]
pub struct ServiceProvider {
    pub id: Uuid,
    pub company_name: String,
    pub description: String,
    pub country_code: String,
    pub services: Vec<Service>,
    pub rating: f32,
    pub verified: bool,
}

#[derive(Debug, FromRow)]
pub struct Service {
    pub id: Uuid,
    pub title: String,
    pub category: String,
    pub price_from: Decimal,
    pub delivery_time_days: i32,
}
```

### Month 11-12: Mobile App & Advanced Features

---

## 5. Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_registration() {
        // Setup test database
        let pool = setup_test_db().await;
        let service = AuthService::new(pool, "test_secret".to_string());

        let req = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "Password123!".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            country_code: "ZA".to_string(),
        };

        let result = service.register(req).await;
        assert!(result.is_ok());
        
        let user = result.unwrap().user;
        assert_eq!(user.email, "test@example.com");
    }
}
```

### Integration Tests

```rust
// tests/integration/auth_tests.rs
use actix_web::{test, web, App};

#[actix_web::test]
async fn test_auth_flow() {
    let app = test::init_service(
        App::new().configure(handlers::init_routes)
    ).await;

    // Register
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&json!({
            "email": "test@example.com",
            "password": "Password123!",
            "first_name": "Test",
            "last_name": "User",
            "country_code": "ZA"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

---

## 6. Common Issues & Solutions

### Issue 1: Database Connection Pool Exhaustion

**Solution:**
```rust
// Increase pool size in config
let pool = PgPoolOptions::new()
    .max_connections(20)  // Increase from default 5
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

### Issue 2: AI API Rate Limits

**Solution:**
```rust
// Implement exponential backoff
use tokio::time::{sleep, Duration};

async fn call_with_retry<F, Fut, T>(f: F, max_retries: u32) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                sleep(Duration::from_secs(2u64.pow(retries))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Issue 3: JWT Token Expiration

**Solution:**
```rust
// Implement refresh token rotation
pub async fn refresh_token(&self, refresh_token: &str) -> Result<Tokens, AppError> {
    // Verify refresh token
    let claims = self.verify_token(refresh_token)?;
    
    // Check if token is in whitelist (stored in Redis)
    let is_valid: bool = redis::get(format!("refresh:{}", claims.sub)).await?;
    if !is_valid {
        return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
    }
    
    // Generate new tokens
    let tokens = self.generate_tokens(&claims.sub)?;
    
    // Invalidate old refresh token
    redis::del(format!("refresh:{}", claims.sub)).await?;
    
    // Store new refresh token
    redis::set_ex(format!("refresh:{}", claims.sub), &tokens.refresh, 7 * 24 * 3600).await?;
    
    Ok(tokens)
}
```

---

## Quick Reference: Daily Development Workflow

```bash
# 1. Start development environment
docker-compose up -d postgres redis

# 2. Run migrations
cargo sqlx migrate run

# 3. Start server with auto-reload
cargo watch -x run

# 4. In another terminal, test endpoints
curl http://localhost:8080/api/v1/health

# 5. Run tests before committing
cargo test

# 6. Check code formatting
cargo fmt -- --check

# 7. Run linter
cargo clippy -- -D warnings

# 8. Build release
cargo build --release
```

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-03-20  
**Remember**: Start small, test often, deploy incrementally!
