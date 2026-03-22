# VentureMate - Database Schema & Migrations

> Complete database documentation for VentureMate Platform

## 📋 Table of Contents

1. [Overview](#1-overview)
2. [Core Tables](#2-core-tables)
3. [Business Domain](#3-business-domain)
4. [AI Generation Domain](#4-ai-generation-domain)
5. [Branding & Media](#5-branding--media)
6. [Website Builder](#6-website-builder)
7. [Marketplace Domain](#7-marketplace-domain)
8. [Billing & Subscriptions](#8-billing--subscriptions)
9. [Indexes & Performance](#9-indexes--performance)
10. [Migrations Guide](#10-migrations-guide)

---

## 1. Overview

### Database Choice: PostgreSQL 15+

VentureMate uses PostgreSQL as the primary database due to:
- **ACID compliance** for financial transactions
- **JSONB support** for flexible schema evolution
- **Full-text search** capabilities
- **Extensions ecosystem** (pgvector, PostGIS, etc.)
- **Proven scalability** for SaaS applications

### Entity Relationship Diagram (High Level)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           VENTUREMATE DATABASE SCHEMA                            │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────┐       ┌───────────────┐       ┌───────────────┐
│   users     │───────│  businesses   │───────│  websites     │
└──────┬──────┘       └───────┬───────┘       └───────────────┘
       │                      │
       │              ┌───────┴───────┐
       │              │               │
       │       ┌──────▼──────┐ ┌──────▼──────┐
       │       │   branding  │ │  documents  │
       │       └─────────────┘ └─────────────┘
       │
       │       ┌───────────────┐       ┌───────────────┐
       └──────►│ subscriptions │       │   ai_jobs     │
               └───────────────┘       └───────────────┘
```

---

## 2. Core Tables

### 2.1 Users Table

```sql
-- users table: Core user accounts
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
    timezone VARCHAR(50) DEFAULT 'Africa/Johannesburg',
    
    -- OAuth providers
    google_id VARCHAR(255) UNIQUE,
    
    -- Subscription reference
    current_subscription_id UUID,
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'deleted')),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Onboarding
    onboarding_completed BOOLEAN DEFAULT FALSE,
    onboarding_step VARCHAR(50),
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_google_id ON users(google_id) WHERE google_id IS NOT NULL;
CREATE INDEX idx_users_status ON users(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_country ON users(country_code);
CREATE INDEX idx_users_created_at ON users(created_at);
```

### 2.2 Sessions Table

```sql
-- sessions table: User session management
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    refresh_token_hash VARCHAR(255) NOT NULL,
    
    -- Device info
    device_fingerprint VARCHAR(255),
    user_agent TEXT,
    ip_address INET,
    
    -- Expiration
    expires_at TIMESTAMPTZ NOT NULL,
    refresh_expires_at TIMESTAMPTZ NOT NULL,
    
    -- Status
    revoked_at TIMESTAMPTZ,
    revoked_reason VARCHAR(100),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(token_hash);
CREATE INDEX idx_sessions_refresh ON sessions(refresh_token_hash);
CREATE INDEX idx_sessions_expires ON sessions(expires_at) WHERE revoked_at IS NULL;
```

### 2.3 Password Resets Table

```sql
-- password_resets table: Password reset tokens
CREATE TABLE password_resets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_password_resets_token ON password_resets(token_hash);
CREATE INDEX idx_password_resets_user ON password_resets(user_id);
```

---

## 3. Business Domain

### 3.1 Businesses Table

```sql
-- businesses table: Core business entities
CREATE TABLE businesses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Basic Info
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    tagline VARCHAR(500),
    description TEXT,
    
    -- Classification
    industry VARCHAR(100) NOT NULL,
    sub_industry VARCHAR(100),
    
    -- Location
    country_code CHAR(2) NOT NULL,
    city VARCHAR(100),
    
    -- Status & Stage
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'archived', 'deleted')),
    stage VARCHAR(30) DEFAULT 'idea' CHECK (stage IN ('idea', 'validation', 'mvp', 'early_traction', 'growth', 'scaling')),
    
    -- Legal
    legal_structure VARCHAR(50),
    registration_number VARCHAR(100),
    founded_date DATE,
    tax_id VARCHAR(100),
    
    -- Branding (denormalized for quick access)
    logo_url TEXT,
    brand_colors JSONB DEFAULT '{}',
    
    -- Web presence
    website_url TEXT,
    custom_domain VARCHAR(255),
    
    -- Health score (cached)
    health_score INTEGER CHECK (health_score >= 0 AND health_score <= 100),
    health_score_updated_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- Settings
    settings JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes
CREATE INDEX idx_businesses_owner ON businesses(owner_id);
CREATE INDEX idx_businesses_slug ON businesses(slug);
CREATE INDEX idx_businesses_country ON businesses(country_code);
CREATE INDEX idx_businesses_status ON businesses(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_businesses_stage ON businesses(stage);
CREATE INDEX idx_businesses_industry ON businesses(industry);
CREATE INDEX idx_businesses_health ON businesses(health_score) WHERE status = 'active';
CREATE INDEX idx_businesses_created ON businesses(created_at);

-- GIN index for JSONB queries
CREATE INDEX idx_businesses_metadata ON businesses USING GIN(metadata);
CREATE INDEX idx_businesses_brand_colors ON businesses USING GIN(brand_colors);
```

### 3.2 Business Team Members

```sql
-- business_members table: Team management
CREATE TABLE business_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    role VARCHAR(30) NOT NULL DEFAULT 'member' 
        CHECK (role IN ('founder', 'co_founder', 'admin', 'member', 'advisor')),
    
    permissions JSONB DEFAULT '{}',
    
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    invited_by UUID REFERENCES users(id),
    invitation_accepted_at TIMESTAMPTZ,
    
    UNIQUE(business_id, user_id)
);

CREATE INDEX idx_business_members_business ON business_members(business_id);
CREATE INDEX idx_business_members_user ON business_members(user_id);
```

### 3.3 Business Checklist

```sql
-- checklist_categories table
CREATE TABLE checklist_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    country_code CHAR(2), -- NULL for global
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- checklist_items table
CREATE TABLE checklist_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES checklist_categories(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    priority VARCHAR(20) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
    country_code CHAR(2), -- NULL for global
    order_index INTEGER DEFAULT 0,
    estimated_duration_minutes INTEGER,
    required_for_stage VARCHAR(30),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- business_checklist_progress table
CREATE TABLE business_checklist_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    checklist_item_id UUID NOT NULL REFERENCES checklist_items(id),
    
    completed BOOLEAN DEFAULT FALSE,
    completed_at TIMESTAMPTZ,
    completed_by UUID REFERENCES users(id),
    notes TEXT,
    attachments JSONB DEFAULT '[]',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id, checklist_item_id)
);

CREATE INDEX idx_checklist_progress_business ON business_checklist_progress(business_id);
CREATE INDEX idx_checklist_progress_item ON business_checklist_progress(checklist_item_id);
```

### 3.4 Industries Reference Table

```sql
-- industries reference table
CREATE TABLE industries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_id UUID REFERENCES industries(id),
    
    -- AI generation hints
    ai_prompt_context TEXT,
    common_business_models JSONB DEFAULT '[]',
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed data
INSERT INTO industries (code, name, description, ai_prompt_context) VALUES
('agritech', 'AgriTech', 'Agricultural technology', 'Focus on supply chain, farmer empowerment, food security'),
('fintech', 'FinTech', 'Financial technology', 'Focus on financial inclusion, mobile payments, lending'),
('healthtech', 'HealthTech', 'Healthcare technology', 'Focus on telemedicine, health records, accessibility'),
('edtech', 'EdTech', 'Education technology', 'Focus on e-learning, skill development, accessibility'),
('ecommerce', 'E-Commerce', 'Online commerce', 'Focus on marketplace, logistics, payments'),
('logistics', 'Logistics', 'Transportation & logistics', 'Focus on last-mile delivery, supply chain optimization'),
('saas', 'SaaS', 'Software as a Service', 'Focus on B2B solutions, recurring revenue, scalability'),
('media', 'Media & Entertainment', 'Content and media', 'Focus on content creation, distribution, monetization'),
('energy', 'Energy', 'Clean energy solutions', 'Focus on renewable energy, access, efficiency'),
('realestate', 'PropTech', 'Real estate technology', 'Focus on property management, marketplaces, fintech');
```

---

## 4. AI Generation Domain

### 4.1 AI Generation Jobs

```sql
-- ai_generation_jobs table: Track all AI generation requests
CREATE TABLE ai_generation_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Job details
    job_type VARCHAR(50) NOT NULL 
        CHECK (job_type IN ('business_plan', 'pitch_deck', 'one_pager', 'logo', 'branding_kit', 
                           'website', 'content', 'financial_model', 'legal_document')),
    
    -- Status tracking
    status VARCHAR(20) DEFAULT 'queued' 
        CHECK (status IN ('queued', 'processing', 'completed', 'failed', 'cancelled')),
    progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    
    -- Input
    input_params JSONB NOT NULL DEFAULT '{}',
    prompt_version VARCHAR(20),
    
    -- Output
    result JSONB,
    output_urls JSONB DEFAULT '[]',
    
    -- AI model info
    ai_model VARCHAR(100),
    token_usage INTEGER,
    cost DECIMAL(10, 4),
    
    -- Error tracking
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    
    -- Timings
    queued_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    -- Webhook
    webhook_url TEXT,
    webhook_delivered_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_ai_jobs_business ON ai_generation_jobs(business_id);
CREATE INDEX idx_ai_jobs_user ON ai_generation_jobs(user_id);
CREATE INDEX idx_ai_jobs_status ON ai_generation_jobs(status);
CREATE INDEX idx_ai_jobs_type ON ai_generation_jobs(job_type);
CREATE INDEX idx_ai_jobs_created ON ai_generation_jobs(created_at);
CREATE INDEX idx_ai_jobs_queued_at ON ai_generation_jobs(queued_at) WHERE status = 'queued';
```

### 4.2 Generated Documents

```sql
-- generated_documents table: Store generated content
CREATE TABLE generated_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    job_id UUID REFERENCES ai_generation_jobs(id),
    
    document_type VARCHAR(50) NOT NULL,
    version INTEGER DEFAULT 1,
    
    -- Content (stored as JSON for structured data)
    content JSONB,
    
    -- File references
    file_url TEXT,
    file_size BIGINT,
    file_format VARCHAR(20),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    -- AI info
    ai_model VARCHAR(100),
    token_usage INTEGER,
    
    -- Status
    is_archived BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id, document_type, version)
);

CREATE INDEX idx_gen_docs_business ON generated_documents(business_id);
CREATE INDEX idx_gen_docs_type ON generated_documents(document_type);
CREATE INDEX idx_gen_docs_job ON generated_documents(job_id);
```

### 4.3 AI Prompts Registry

```sql
-- ai_prompts table: Versioned prompt templates
CREATE TABLE ai_prompts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    prompt_id VARCHAR(100) NOT NULL,
    version VARCHAR(20) NOT NULL,
    
    -- Content
    name VARCHAR(200) NOT NULL,
    description TEXT,
    system_prompt TEXT NOT NULL,
    user_template TEXT NOT NULL,
    
    -- Configuration
    variables JSONB DEFAULT '[]',
    output_schema JSONB,
    model_config JSONB DEFAULT '{}',
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Usage
    use_count INTEGER DEFAULT 0,
    avg_tokens INTEGER,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    
    UNIQUE(prompt_id, version)
);

CREATE INDEX idx_ai_prompts_id ON ai_prompts(prompt_id);
CREATE INDEX idx_ai_prompts_active ON ai_prompts(prompt_id, version) WHERE is_active = TRUE;
```

---

## 5. Branding & Media

### 5.1 Brand Assets

```sql
-- brand_assets table: Logos and other brand media
CREATE TABLE brand_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    asset_type VARCHAR(50) NOT NULL 
        CHECK (asset_type IN ('logo', 'favicon', 'social_banner', 'business_card', 'flyer', 'icon')),
    
    -- Storage
    file_url TEXT NOT NULL,
    thumbnail_url TEXT,
    file_size BIGINT,
    format VARCHAR(10),
    dimensions JSONB, -- {width, height}
    
    -- For logos: variants
    variant VARCHAR(50), -- primary, dark, light, icon, etc.
    
    -- For AI generated
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    generation_params JSONB,
    
    -- Selection
    is_selected BOOLEAN DEFAULT FALSE,
    selected_at TIMESTAMPTZ,
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_brand_assets_business ON brand_assets(business_id);
CREATE INDEX idx_brand_assets_type ON brand_assets(asset_type);
CREATE INDEX idx_brand_assets_selected ON brand_assets(business_id, asset_type) WHERE is_selected = TRUE;
```

### 5.2 Brand Colors

```sql
-- brand_colors table: Color palette storage
CREATE TABLE brand_colors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Color palette
    palette JSONB NOT NULL, -- {primary, secondary, accent, neutral, background, text, success, warning, error}
    
    -- AI generation info
    ai_generated BOOLEAN DEFAULT FALSE,
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    
    -- Selection
    is_active BOOLEAN DEFAULT TRUE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_brand_colors_business ON brand_colors(business_id);
```

### 5.3 User Uploads / Document Vault

```sql
-- uploads table: General file storage
CREATE TABLE uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- File info
    original_name VARCHAR(255) NOT NULL,
    storage_path TEXT NOT NULL,
    public_url TEXT,
    
    file_size BIGINT,
    mime_type VARCHAR(100),
    checksum VARCHAR(64),
    
    -- Categorization
    folder_id UUID,
    tags JSONB DEFAULT '[]',
    
    -- Access
    visibility VARCHAR(20) DEFAULT 'private' CHECK (visibility IN ('private', 'shared', 'public')),
    share_token VARCHAR(255),
    share_expires_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_uploads_business ON uploads(business_id);
CREATE INDEX idx_uploads_user ON uploads(user_id);
CREATE INDEX idx_uploads_folder ON uploads(folder_id);
CREATE INDEX idx_uploads_share ON uploads(share_token) WHERE share_token IS NOT NULL;
```

### 5.4 Upload Folders

```sql
-- upload_folders table: Organize uploads
CREATE TABLE upload_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES upload_folders(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_upload_folders_business ON upload_folders(business_id);
```

---

## 6. Website Builder

### 6.1 Websites

```sql
-- websites table: Website configuration
CREATE TABLE websites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Domain
    subdomain VARCHAR(100) UNIQUE NOT NULL,
    custom_domain VARCHAR(255),
    domain_status VARCHAR(20) DEFAULT 'not_connected' 
        CHECK (domain_status IN ('not_connected', 'pending_dns', 'active', 'ssl_pending', 'error')),
    
    -- Template
    template VARCHAR(50) NOT NULL DEFAULT 'startup-modern',
    template_config JSONB DEFAULT '{}',
    
    -- Status
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'unpublished')),
    
    -- SEO
    seo_title VARCHAR(100),
    seo_description VARCHAR(300),
    seo_keywords JSONB DEFAULT '[]',
    og_image_url TEXT,
    
    -- Analytics
    analytics_config JSONB DEFAULT '{}',
    
    -- Publishing
    published_at TIMESTAMPTZ,
    last_modified_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- AI generation
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id)
);

CREATE INDEX idx_websites_business ON websites(business_id);
CREATE INDEX idx_websites_subdomain ON websites(subdomain);
CREATE INDEX idx_websites_custom_domain ON websites(custom_domain) WHERE custom_domain IS NOT NULL;
CREATE INDEX idx_websites_status ON websites(status);
```

### 6.2 Website Pages

```sql
-- website_pages table: Individual pages
CREATE TABLE website_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    website_id UUID NOT NULL REFERENCES websites(id) ON DELETE CASCADE,
    
    page_key VARCHAR(50) NOT NULL, -- home, about, contact, etc.
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    
    -- Content
    sections JSONB NOT NULL DEFAULT '[]',
    
    -- Settings
    is_enabled BOOLEAN DEFAULT TRUE,
    is_homepage BOOLEAN DEFAULT FALSE,
    order_index INTEGER DEFAULT 0,
    
    -- SEO
    seo_title VARCHAR(100),
    seo_description VARCHAR(300),
    
    -- AI generation
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(website_id, page_key)
);

CREATE INDEX idx_website_pages_website ON website_pages(website_id);
CREATE INDEX idx_website_pages_enabled ON website_pages(website_id) WHERE is_enabled = TRUE;
```

### 6.3 Website Templates

```sql
-- website_templates reference table
CREATE TABLE website_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Preview
    thumbnail_url TEXT,
    preview_url TEXT,
    
    -- Configuration
    category VARCHAR(50),
    industries JSONB DEFAULT '[]',
    
    -- Features
    features JSONB DEFAULT '[]',
    default_sections JSONB DEFAULT '[]',
    
    -- Availability
    is_active BOOLEAN DEFAULT TRUE,
    is_premium BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## 7. Marketplace Domain

### 7.1 Service Providers

```sql
-- service_providers table
CREATE TABLE service_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    
    -- Company info
    company_name VARCHAR(200) NOT NULL,
    description TEXT,
    logo_url TEXT,
    website_url TEXT,
    
    -- Contact
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(20),
    
    -- Location
    country_code CHAR(2) NOT NULL,
    city VARCHAR(100),
    
    -- Verification
    is_verified BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    
    -- Rating
    rating DECIMAL(2, 1) DEFAULT 5.0,
    review_count INTEGER DEFAULT 0,
    completed_projects INTEGER DEFAULT 0,
    
    -- Status
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'suspended')),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_service_providers_country ON service_providers(country_code);
CREATE INDEX idx_service_providers_status ON service_providers(status);
CREATE INDEX idx_service_providers_rating ON service_providers(rating);
```

### 7.2 Services

```sql
-- services table
CREATE TABLE services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES service_providers(id) ON DELETE CASCADE,
    
    title VARCHAR(200) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    
    -- Pricing
    pricing_model VARCHAR(20) CHECK (pricing_model IN ('fixed', 'hourly', 'package')),
    price_from DECIMAL(12, 2),
    price_to DECIMAL(12, 2),
    currency CHAR(3) DEFAULT 'USD',
    
    -- Delivery
    delivery_time_days INTEGER,
    
    -- Media
    images JSONB DEFAULT '[]',
    
    -- Availability
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Stats
    order_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_services_provider ON services(provider_id);
CREATE INDEX idx_services_category ON services(category);
CREATE INDEX idx_services_active ON services(category) WHERE is_active = TRUE;
```

### 7.3 Service Requests

```sql
-- service_requests table
CREATE TABLE service_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id),
    service_id UUID NOT NULL REFERENCES services(id),
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Request details
    requirements TEXT NOT NULL,
    attachments JSONB DEFAULT '[]',
    
    -- Pricing
    agreed_price DECIMAL(12, 2),
    currency CHAR(3),
    
    -- Status
    status VARCHAR(20) DEFAULT 'pending' 
        CHECK (status IN ('pending', 'accepted', 'in_progress', 'completed', 'cancelled', 'disputed')),
    
    -- Timeline
    requested_at TIMESTAMPTZ DEFAULT NOW(),
    accepted_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    deadline TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_service_requests_business ON service_requests(business_id);
CREATE INDEX idx_service_requests_user ON service_requests(user_id);
CREATE INDEX idx_service_requests_status ON service_requests(status);
```

---

## 8. Billing & Subscriptions

### 8.1 Subscription Plans

```sql
-- subscription_plans table
CREATE TABLE subscription_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Pricing
    price_monthly DECIMAL(10, 2) NOT NULL,
    price_yearly DECIMAL(10, 2) NOT NULL,
    currency CHAR(3) DEFAULT 'USD',
    
    -- Features
    features JSONB DEFAULT '[]',
    
    -- Limits
    limits JSONB DEFAULT '{}', -- {businesses, ai_generations, storage_gb, website_pages}
    
    -- Display
    is_active BOOLEAN DEFAULT TRUE,
    is_popular BOOLEAN DEFAULT FALSE,
    display_order INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed plans
INSERT INTO subscription_plans (code, name, price_monthly, price_yearly, limits, features) VALUES
('free', 'Free', 0, 0, 
 '{"businesses": 1, "ai_generations_per_month": 10, "storage_gb": 1, "website_pages": 3}',
 '["1 business", "Basic AI generation", "Standard templates", "Community support"]'),

('starter', 'Starter', 29, 290,
 '{"businesses": 3, "ai_generations_per_month": 50, "storage_gb": 10, "website_pages": 10}',
 '["3 businesses", "Advanced AI generation", "Premium templates", "Custom domain", "Priority support"]'),

('growth', 'Growth', 79, 790,
 '{"businesses": -1, "ai_generations_per_month": -1, "storage_gb": 50, "website_pages": -1}',
 '["Unlimited businesses", "Unlimited AI generation", "All templates", "Custom domain + SSL", "Analytics", "CRM tools", "24/7 support"]');
```

### 8.2 Subscriptions

```sql
-- subscriptions table
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES subscription_plans(id),
    
    -- Stripe references
    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' 
        CHECK (status IN ('active', 'cancelled', 'past_due', 'unpaid', 'trialing')),
    
    -- Billing
    billing_interval VARCHAR(10) CHECK (billing_interval IN ('month', 'year')),
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN DEFAULT FALSE,
    
    -- Usage tracking
    usage_this_period JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    cancelled_at TIMESTAMPTZ
);

CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_period_end ON subscriptions(current_period_end);
```

### 8.3 Invoices

```sql
-- invoices table
CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID REFERENCES subscriptions(id),
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Invoice details
    invoice_number VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    
    -- Amounts
    subtotal DECIMAL(12, 2) NOT NULL,
    tax_amount DECIMAL(12, 2) DEFAULT 0,
    total DECIMAL(12, 2) NOT NULL,
    currency CHAR(3) DEFAULT 'USD',
    
    -- Status
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'failed', 'refunded')),
    
    -- Stripe
    stripe_invoice_id VARCHAR(255),
    stripe_payment_intent_id VARCHAR(255),
    
    -- Timestamps
    invoice_date DATE NOT NULL,
    due_date DATE,
    paid_at TIMESTAMPTZ,
    
    -- PDF
    pdf_url TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_invoices_user ON invoices(user_id);
CREATE INDEX idx_invoices_subscription ON invoices(subscription_id);
CREATE INDEX idx_invoices_status ON invoices(status);
```

### 8.4 Payment Methods

```sql
-- payment_methods table
CREATE TABLE payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Stripe
    stripe_payment_method_id VARCHAR(255) NOT NULL,
    
    -- Card info (last 4 only!)
    type VARCHAR(20) DEFAULT 'card',
    card_brand VARCHAR(20),
    card_last4 VARCHAR(4),
    card_exp_month INTEGER,
    card_exp_year INTEGER,
    
    -- Billing details
    billing_name VARCHAR(200),
    billing_email VARCHAR(255),
    
    -- Status
    is_default BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_payment_methods_user ON payment_methods(user_id);
```

---

## 9. Indexes & Performance

### 9.1 Critical Query Patterns

```sql
-- Dashboard: User's businesses with health scores
CREATE INDEX idx_businesses_dashboard ON businesses(owner_id, status, updated_at) 
WHERE deleted_at IS NULL;

-- AI Queue: Pending jobs ordered by creation
CREATE INDEX idx_ai_jobs_queue ON ai_generation_jobs(status, created_at) 
WHERE status IN ('queued', 'processing');

-- Analytics: Business activity over time
CREATE INDEX idx_businesses_activity ON businesses(created_at, country_code, industry) 
WHERE status = 'active';

-- Search: Business name search
CREATE INDEX idx_businesses_name_search ON businesses USING gin(to_tsvector('english', name));

-- Time-series: Website analytics
CREATE INDEX idx_website_analytics_time ON website_analytics(website_id, date);
```

### 9.2 Partitioning Strategy (Future)

```sql
-- Partition ai_generation_jobs by month for large scale
CREATE TABLE ai_generation_jobs (
    -- ... columns ...
    created_at TIMESTAMPTZ NOT NULL
) PARTITION BY RANGE (created_at);

-- Create monthly partitions
CREATE TABLE ai_generation_jobs_2025_03 PARTITION OF ai_generation_jobs
    FOR VALUES FROM ('2025-03-01') TO ('2025-04-01');
```

---

## 10. Migrations Guide

### 10.1 Migration Naming Convention

```
YYYYMMDDHHMMSS_description.sql
Example: 20250320100000_create_users_table.sql
```

### 10.2 Creating Migrations

```bash
# Using sqlx-cli
cargo sqlx migrate add create_businesses_table

# This creates: migrations/20250320100000_create_businesses_table.sql
```

### 10.3 Running Migrations

```bash
# Run pending migrations
cargo sqlx migrate run

# Revert last migration
cargo sqlx migrate revert

# Check status
cargo sqlx migrate info
```

### 10.4 Migration Template

```sql
-- migrations/20250320100000_example.sql
-- Up migration
BEGIN;

-- Create table
CREATE TABLE example (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_example_name ON example(name);

COMMIT;
```

```sql
-- Down migration (in separate file or use revert script)
BEGIN;

DROP INDEX IF EXISTS idx_example_name;
DROP TABLE IF EXISTS example;

COMMIT;
```

---

## 11. Data Retention & Cleanup

### 11.1 Soft Delete Policy

```sql
-- Archive old deleted records
CREATE OR REPLACE FUNCTION archive_old_deleted_records()
RETURNS void AS $$
BEGIN
    -- Move deleted businesses older than 30 days to archive
    INSERT INTO businesses_archive 
    SELECT * FROM businesses 
    WHERE deleted_at < NOW() - INTERVAL '30 days';
    
    DELETE FROM businesses 
    WHERE deleted_at < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;
```

### 11.2 Audit Logging

```sql
-- audit_logs table for compliance
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(100) NOT NULL,
    record_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL, -- INSERT, UPDATE, DELETE
    old_data JSONB,
    new_data JSONB,
    user_id UUID,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_table ON audit_logs(table_name, record_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at);
```

---

**Schema Version**: 1.0.0  
**Last Updated**: 2025-03-20  
**PostgreSQL Version**: 15+
