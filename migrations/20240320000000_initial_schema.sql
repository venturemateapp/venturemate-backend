-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- CORE TABLES
-- ============================================================================

-- Users table
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

-- Sessions table
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

-- Password resets table
CREATE TABLE password_resets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Email verification tokens
CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- BUSINESS DOMAIN
-- ============================================================================

-- Businesses table
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

-- Business Team Members
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

-- Checklist Categories
CREATE TABLE checklist_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    country_code CHAR(2),
    order_index INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Checklist Items
CREATE TABLE checklist_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES checklist_categories(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    priority VARCHAR(20) DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
    country_code CHAR(2),
    order_index INTEGER DEFAULT 0,
    estimated_duration_minutes INTEGER,
    required_for_stage VARCHAR(30),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Business Checklist Progress
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

-- Industries reference table
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

-- ============================================================================
-- AI GENERATION DOMAIN
-- ============================================================================

-- AI Generation Jobs
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

-- Generated Documents
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

-- AI Prompts Registry
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

-- ============================================================================
-- BRANDING & MEDIA
-- ============================================================================

-- Brand Assets
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
    dimensions JSONB,
    
    -- For logos: variants
    variant VARCHAR(50),
    
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

-- Brand Colors
CREATE TABLE brand_colors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Color palette
    palette JSONB NOT NULL,
    
    -- AI generation info
    ai_generated BOOLEAN DEFAULT FALSE,
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    
    -- Selection
    is_active BOOLEAN DEFAULT TRUE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Uploads / Document Vault
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

-- Upload Folders
CREATE TABLE upload_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    name VARCHAR(100) NOT NULL,
    parent_id UUID REFERENCES upload_folders(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================================
-- WEBSITE BUILDER
-- ============================================================================

-- Websites
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

-- Website Pages
CREATE TABLE website_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    website_id UUID NOT NULL REFERENCES websites(id) ON DELETE CASCADE,
    
    page_key VARCHAR(50) NOT NULL,
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

-- Website Templates
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

-- ============================================================================
-- MARKETPLACE DOMAIN
-- ============================================================================

-- Service Providers
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

-- Services
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

-- Service Requests
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

-- ============================================================================
-- BILLING & SUBSCRIPTIONS
-- ============================================================================

-- Subscription Plans
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
    limits JSONB DEFAULT '{}',
    
    -- Display
    is_active BOOLEAN DEFAULT TRUE,
    is_popular BOOLEAN DEFAULT FALSE,
    display_order INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Subscriptions
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

-- Invoices
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

-- Payment Methods
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

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Users indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_google_id ON users(google_id) WHERE google_id IS NOT NULL;
CREATE INDEX idx_users_status ON users(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_country ON users(country_code);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Sessions indexes
CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(token_hash);
CREATE INDEX idx_sessions_refresh ON sessions(refresh_token_hash);
CREATE INDEX idx_sessions_expires ON sessions(expires_at) WHERE revoked_at IS NULL;

-- Password resets indexes
CREATE INDEX idx_password_resets_token ON password_resets(token_hash);
CREATE INDEX idx_password_resets_user ON password_resets(user_id);

-- Businesses indexes
CREATE INDEX idx_businesses_owner ON businesses(owner_id);
CREATE INDEX idx_businesses_slug ON businesses(slug);
CREATE INDEX idx_businesses_country ON businesses(country_code);
CREATE INDEX idx_businesses_status ON businesses(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_businesses_stage ON businesses(stage);
CREATE INDEX idx_businesses_industry ON businesses(industry);
CREATE INDEX idx_businesses_health ON businesses(health_score) WHERE status = 'active';
CREATE INDEX idx_businesses_created ON businesses(created_at);
CREATE INDEX idx_businesses_metadata ON businesses USING GIN(metadata);
CREATE INDEX idx_businesses_brand_colors ON businesses USING GIN(brand_colors);

-- Business members indexes
CREATE INDEX idx_business_members_business ON business_members(business_id);
CREATE INDEX idx_business_members_user ON business_members(user_id);

-- AI generation indexes
CREATE INDEX idx_ai_jobs_business ON ai_generation_jobs(business_id);
CREATE INDEX idx_ai_jobs_user ON ai_generation_jobs(user_id);
CREATE INDEX idx_ai_jobs_status ON ai_generation_jobs(status);
CREATE INDEX idx_ai_jobs_type ON ai_generation_jobs(job_type);
CREATE INDEX idx_ai_jobs_created ON ai_generation_jobs(created_at);
CREATE INDEX idx_ai_jobs_queued_at ON ai_generation_jobs(queued_at) WHERE status = 'queued';

-- Documents indexes
CREATE INDEX idx_gen_docs_business ON generated_documents(business_id);
CREATE INDEX idx_gen_docs_type ON generated_documents(document_type);
CREATE INDEX idx_gen_docs_job ON generated_documents(job_id);

-- Brand assets indexes
CREATE INDEX idx_brand_assets_business ON brand_assets(business_id);
CREATE INDEX idx_brand_assets_type ON brand_assets(asset_type);
CREATE INDEX idx_brand_assets_selected ON brand_assets(business_id, asset_type) WHERE is_selected = TRUE;

-- Uploads indexes
CREATE INDEX idx_uploads_business ON uploads(business_id);
CREATE INDEX idx_uploads_user ON uploads(user_id);
CREATE INDEX idx_uploads_folder ON uploads(folder_id);
CREATE INDEX idx_uploads_share ON uploads(share_token) WHERE share_token IS NOT NULL;

-- Website indexes
CREATE INDEX idx_websites_business ON websites(business_id);
CREATE INDEX idx_websites_subdomain ON websites(subdomain);
CREATE INDEX idx_websites_custom_domain ON websites(custom_domain) WHERE custom_domain IS NOT NULL;
CREATE INDEX idx_websites_status ON websites(status);

-- Subscriptions indexes
CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_period_end ON subscriptions(current_period_end);

-- Invoices indexes
CREATE INDEX idx_invoices_user ON invoices(user_id);
CREATE INDEX idx_invoices_subscription ON invoices(subscription_id);
CREATE INDEX idx_invoices_status ON invoices(status);

-- Service providers indexes
CREATE INDEX idx_service_providers_country ON service_providers(country_code);
CREATE INDEX idx_service_providers_status ON service_providers(status);
CREATE INDEX idx_service_providers_rating ON service_providers(rating);

-- Services indexes
CREATE INDEX idx_services_provider ON services(provider_id);
CREATE INDEX idx_services_category ON services(category);
CREATE INDEX idx_services_active ON services(category) WHERE is_active = TRUE;

-- ============================================================================
-- TRIGGERS FOR AUTO-UPDATING updated_at
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_businesses_updated_at BEFORE UPDATE ON businesses FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_business_members_updated_at BEFORE UPDATE ON business_members FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_business_checklist_progress_updated_at BEFORE UPDATE ON business_checklist_progress FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_generation_jobs_updated_at BEFORE UPDATE ON ai_generation_jobs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_generated_documents_updated_at BEFORE UPDATE ON generated_documents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_brand_assets_updated_at BEFORE UPDATE ON brand_assets FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_uploads_updated_at BEFORE UPDATE ON uploads FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_websites_updated_at BEFORE UPDATE ON websites FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_website_pages_updated_at BEFORE UPDATE ON website_pages FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_service_providers_updated_at BEFORE UPDATE ON service_providers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_services_updated_at BEFORE UPDATE ON services FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_service_requests_updated_at BEFORE UPDATE ON service_requests FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_subscriptions_updated_at BEFORE UPDATE ON subscriptions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_subscription_plans_updated_at BEFORE UPDATE ON subscription_plans FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Seed industries
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

-- Seed subscription plans
INSERT INTO subscription_plans (code, name, price_monthly, price_yearly, limits, features, display_order) VALUES
('free', 'Free', 0, 0, 
 '{"businesses": 1, "ai_generations_per_month": 10, "storage_gb": 1, "website_pages": 3}',
 '["1 business", "Basic AI generation", "Standard templates", "Community support"]',
 1),

('starter', 'Starter', 29, 290,
 '{"businesses": 3, "ai_generations_per_month": 50, "storage_gb": 10, "website_pages": 10}',
 '["3 businesses", "Advanced AI generation", "Premium templates", "Custom domain", "Priority support"]',
 2),

('growth', 'Growth', 79, 790,
 '{"businesses": -1, "ai_generations_per_month": -1, "storage_gb": 50, "website_pages": -1}',
 '["Unlimited businesses", "Unlimited AI generation", "All templates", "Custom domain + SSL", "Analytics", "CRM tools", "24/7 support"]',
 3);

-- Seed checklist categories
INSERT INTO checklist_categories (name, description, order_index) VALUES
('Legal & Compliance', 'Business registration and legal requirements', 1),
('Digital Presence', 'Website, social media, and online presence', 2),
('Branding', 'Logo, colors, and brand identity', 3),
('Financial', 'Banking, accounting, and financial setup', 4),
('Operations', 'Day-to-day business operations', 5);

-- Seed checklist items
INSERT INTO checklist_items (category_id, title, description, priority, order_index) 
SELECT 
    c.id,
    item->>'title',
    item->>'description',
    item->>'priority',
    (item->>'order')::int
FROM checklist_categories c
CROSS JOIN jsonb_array_elements('[
    {"title": "Register business name", "description": "Officially register your business with local authorities", "priority": "high", "order": 1},
    {"title": "Obtain TIN", "description": "Get your Tax Identification Number", "priority": "high", "order": 2},
    {"title": "Register for VAT", "description": "Register for Value Added Tax if applicable", "priority": "medium", "order": 3}
]'::jsonb) as item
WHERE c.name = 'Legal & Compliance';
