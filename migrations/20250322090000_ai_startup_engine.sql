-- ============================================================================
-- AI STARTUP ENGINE MIGRATION
-- Implements AI Startup Engine database schema per specification
-- ============================================================================

-- ============================================================================
-- 1. GENERATION LOGS TABLE - Tracks every AI generation attempt
-- ============================================================================

CREATE TABLE IF NOT EXISTS generation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE SET NULL,
    
    -- Input data
    input_data JSONB NOT NULL DEFAULT '{}',
    onboarding_session_id UUID,
    
    -- AI model info
    ai_model VARCHAR(100) NOT NULL DEFAULT 'claude-3-opus',
    prompt_sent TEXT,
    raw_ai_response TEXT,
    parsed_output JSONB,
    
    -- Processing metrics
    processing_time_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    estimated_cost DECIMAL(10, 4),
    
    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'processing' 
        CHECK (status IN ('processing', 'completed', 'failed', 'partial')),
    error_message TEXT,
    
    -- Blueprint output (cached result)
    blueprint JSONB,
    
    -- Confidence scores
    confidence_overall DECIMAL(3, 2),
    confidence_industry DECIMAL(3, 2),
    confidence_revenue DECIMAL(3, 2),
    confidence_name DECIMAL(3, 2),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    
    -- Index for user lookups and billing
    CONSTRAINT chk_confidence_range CHECK (
        confidence_overall IS NULL OR (confidence_overall >= 0 AND confidence_overall <= 1)
    )
);

CREATE INDEX idx_generation_logs_user ON generation_logs(user_id);
CREATE INDEX idx_generation_logs_business ON generation_logs(business_id);
CREATE INDEX idx_generation_logs_status ON generation_logs(status);
CREATE INDEX idx_generation_logs_created ON generation_logs(created_at);
CREATE INDEX idx_generation_logs_model ON generation_logs(ai_model);

-- ============================================================================
-- 2. AI VALIDATION LOGS TABLE - Tracks validation issues and corrections
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_validation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    generation_log_id UUID NOT NULL REFERENCES generation_logs(id) ON DELETE CASCADE,
    
    -- Validation details
    field_name VARCHAR(100) NOT NULL,
    original_value TEXT,
    corrected_value TEXT,
    validation_rule VARCHAR(200),
    action_taken VARCHAR(50) CHECK (action_taken IN ('auto_fixed', 'flagged_for_review', 'accepted')),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ai_validation_logs_generation ON ai_validation_logs(generation_log_id);
CREATE INDEX idx_ai_validation_logs_field ON ai_validation_logs(field_name);

-- ============================================================================
-- 3. INDUSTRY CLASSIFICATION CACHE TABLE - Caches classifications
-- ============================================================================

CREATE TABLE IF NOT EXISTS industry_classification_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Cache key (hashed keywords)
    keywords_hash VARCHAR(64) UNIQUE NOT NULL,
    keywords_text TEXT, -- Original keywords for debugging
    
    -- Classification result
    industry VARCHAR(100) NOT NULL,
    sub_industry VARCHAR(100),
    confidence_score DECIMAL(3, 2) NOT NULL,
    
    -- Usage tracking
    usage_count INTEGER DEFAULT 1,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ DEFAULT NOW(),
    
    CONSTRAINT chk_cache_confidence CHECK (confidence_score >= 0 AND confidence_score <= 1)
);

CREATE INDEX idx_industry_cache_hash ON industry_classification_cache(keywords_hash);
CREATE INDEX idx_industry_cache_industry ON industry_classification_cache(industry);
CREATE INDEX idx_industry_cache_usage ON industry_classification_cache(usage_count);

-- ============================================================================
-- 4. REGULATORY REQUIREMENTS TABLE - Country-specific compliance rules
-- ============================================================================

CREATE TABLE IF NOT EXISTS regulatory_requirements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Location
    country_code CHAR(2) NOT NULL,
    country_name VARCHAR(100) NOT NULL,
    
    -- Requirement details
    requirement_type VARCHAR(50) NOT NULL 
        CHECK (requirement_type IN ('registration', 'license', 'permit', 'tax', 'compliance', 'certification')),
    requirement_name VARCHAR(200) NOT NULL,
    description TEXT,
    
    -- Applicability
    applicable_industries JSONB DEFAULT '["all"]'::jsonb,
    applicable_business_types JSONB DEFAULT '["all"]'::jsonb,
    
    -- Estimates
    estimated_time_days INTEGER,
    estimated_cost_min DECIMAL(12, 2),
    estimated_cost_max DECIMAL(12, 2),
    currency CHAR(3) DEFAULT 'USD',
    
    -- Documents
    required_documents JSONB DEFAULT '[]'::jsonb,
    
    -- Authority info
    authority_name VARCHAR(200),
    authority_website TEXT,
    authority_contact_email VARCHAR(255),
    authority_contact_phone VARCHAR(50),
    
    -- Priority and rules
    is_mandatory BOOLEAN DEFAULT TRUE,
    priority INTEGER DEFAULT 5 CHECK (priority >= 1 AND priority <= 10),
    condition_note TEXT, -- e.g., "If annual turnover > 25M NGN"
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_regulatory_country ON regulatory_requirements(country_code);
CREATE INDEX idx_regulatory_type ON regulatory_requirements(requirement_type);
CREATE INDEX idx_regulatory_industries ON regulatory_requirements USING GIN(applicable_industries);
CREATE INDEX idx_regulatory_priority ON regulatory_requirements(priority);
CREATE INDEX idx_regulatory_active ON regulatory_requirements(country_code, is_active) WHERE is_active = TRUE;

-- Trigger for updated_at
CREATE TRIGGER update_regulatory_requirements_updated_at 
    BEFORE UPDATE ON regulatory_requirements 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 5. INDUSTRY DEFINITIONS TABLE - Predefined industries with sub-categories
-- ============================================================================

CREATE TABLE IF NOT EXISTS industry_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    industry_code VARCHAR(50) UNIQUE NOT NULL,
    industry_name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Keywords for classification
    classification_keywords JSONB DEFAULT '[]'::jsonb,
    
    -- Revenue model suggestions
    primary_revenue_models JSONB DEFAULT '[]'::jsonb,
    secondary_revenue_models JSONB DEFAULT '[]'::jsonb,
    
    -- Market data
    typical_startup_costs JSONB, -- {min: X, max: Y, currency: Z}
    average_time_to_revenue_months INTEGER,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_industry_definitions_code ON industry_definitions(industry_code);
CREATE INDEX idx_industry_definitions_active ON industry_definitions(is_active);

-- Sub-industries table
CREATE TABLE IF NOT EXISTS sub_industry_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    industry_code VARCHAR(50) NOT NULL REFERENCES industry_definitions(industry_code) ON DELETE CASCADE,
    
    sub_industry_code VARCHAR(50) UNIQUE NOT NULL,
    sub_industry_name VARCHAR(100) NOT NULL,
    description TEXT,
    
    classification_keywords JSONB DEFAULT '[]'::jsonb,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

CREATE INDEX idx_sub_industry_industry ON sub_industry_definitions(industry_code);

-- Trigger for updated_at
CREATE TRIGGER update_industry_definitions_updated_at 
    BEFORE UPDATE ON industry_definitions 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 6. SEED DATA - Industry Definitions
-- ============================================================================

INSERT INTO industry_definitions (industry_code, industry_name, description, classification_keywords, primary_revenue_models, secondary_revenue_models, typical_startup_costs, average_time_to_revenue_months) VALUES
('fintech', 'Fintech', 'Financial technology solutions', '["payment", "bank", "lending", "finance", "money", "transaction", "wallet"]', '["Transaction Fee", "Subscription", "Interest"]', '["Interchange", "Commission", "API Fees"]', '{"min": 50000, "max": 500000, "currency": "USD"}', 6),
('agritech', 'Agritech', 'Agricultural technology', '["farm", "agriculture", "crop", "livestock", "farmer", "harvest", "seed"]', '["Commission", "Subscription", "SaaS"]', '["Financing Interest", "Marketplace Fee"]', '{"min": 25000, "max": 300000, "currency": "USD"}', 9),
('healthtech', 'Healthtech', 'Healthcare technology', '["health", "medical", "doctor", "patient", "hospital", "clinic", "wellness"]', '["Subscription", "Pay-per-use", "Insurance Billing"]', '["SaaS", "Commission"]', '{"min": 100000, "max": 1000000, "currency": "USD"}', 12),
('edtech', 'Edtech', 'Education technology', '["learn", "education", "student", "course", "school", "training", "skill"]', '["Subscription", "Course Fee", "Certification"]', '["B2B Licensing", "Marketplace Fee"]', '{"min": 20000, "max": 200000, "currency": "USD"}', 3),
('ecommerce', 'E-commerce', 'Online commerce platforms', '["shop", "store", "retail", "product", "buy", "sell", "marketplace"]', '["Product Margin", "Marketplace Fee"]', '["Subscription", "Advertising", "Featured Listings"]', '{"min": 10000, "max": 150000, "currency": "USD"}', 1),
('saas', 'SaaS', 'Software as a Service', '["software", "app", "platform", "tool", "solution", "automation"]', '["Subscription", "Tiered Pricing"]', '["Usage-based", "Enterprise", "API Fees"]', '{"min": 30000, "max": 400000, "currency": "USD"}', 6),
('logistics', 'Logistics', 'Transportation and logistics', '["delivery", "shipping", "transport", "fleet", "warehouse", "cargo"]', '["Per-delivery Fee", "Subscription"]', '["SaaS", "Commission"]', '{"min": 50000, "max": 600000, "currency": "USD"}', 4),
('marketplace', 'Marketplace', 'Service and product marketplaces', '["marketplace", "platform", "connect", "booking", "gig", "service"]', '["Commission", "Listing Fee"]', '["Subscription", "Featured Listings", "Advertising"]', '{"min": 75000, "max": 750000, "currency": "USD"}', 6),
('media', 'Media', 'Content and media platforms', '["content", "media", "news", "entertainment", "streaming", "publishing"]', '["Advertising", "Subscription"]', '["Sponsored Content", "Events", "Affiliate"]', '{"min": 25000, "max": 350000, "currency": "USD"}', 9),
('cleantech', 'CleanTech', 'Clean energy and sustainability', '["energy", "solar", "renewable", "recycling", "sustainability", "green", "environment"]', '["Product Sales", "Financing"]', '["Carbon Credits", "Subscription"]', '{"min": 100000, "max": 2000000, "currency": "USD"}', 12),
('proptech', 'PropTech', 'Real estate technology', '["real estate", "property", "housing", "construction", "building", "rent"]', '["Commission", "Subscription"]', '["Advertising", "Data Sales"]', '{"min": 40000, "max": 450000, "currency": "USD"}', 6),
('other', 'Other', 'Other industries', '["other"]', '["Various"]', '["Various"]', '{"min": 10000, "max": 100000, "currency": "USD"}', 6)
ON CONFLICT (industry_code) DO UPDATE SET 
    classification_keywords = EXCLUDED.classification_keywords,
    primary_revenue_models = EXCLUDED.primary_revenue_models,
    updated_at = NOW();

-- Sub-industries
INSERT INTO sub_industry_definitions (industry_code, sub_industry_code, sub_industry_name, description, classification_keywords) VALUES
('fintech', 'payments', 'Payments', 'Payment processing and wallets', '["payment", "wallet", "transfer", "remittance"]'),
('fintech', 'lending', 'Lending', 'Loan and credit services', '["loan", "credit", "lending", "borrow"]'),
('fintech', 'insurance', 'Insurance', 'Insurance technology', '["insurance", "cover", "policy", "risk"]'),
('fintech', 'investment', 'Investment', 'Investment and wealth tech', '["invest", "wealth", "stock", "trading"]'),
('agritech', 'supply_chain', 'Supply Chain', 'Agricultural supply chain', '["supply", "distribution", "logistics", "supply chain"]'),
('agritech', 'farm_tech', 'Farm Tech', 'Farm management technology', '["farm management", "cultivation", "irrigation"]'),
('agritech', 'marketplace_agri', 'Marketplace', 'Agricultural marketplaces', '["market", "buy", "sell", "produce"]'),
('healthtech', 'telemedicine', 'Telemedicine', 'Remote healthcare', '["telemedicine", "virtual care", "remote doctor"]'),
('healthtech', 'pharmacy', 'E-Pharmacy', 'Online pharmacy', '["pharmacy", "drug", "medicine", "prescription"]'),
('healthtech', 'diagnostics', 'Diagnostics', 'Health diagnostics', '["diagnosis", "test", "lab", "screening"]'),
('edtech', 'elearning', 'E-Learning', 'Online learning platforms', '["online course", "e-learning", "virtual learning"]'),
('edtech', 'training', 'Skills Training', 'Vocational training', '["training", "skill", "vocational", "career"]'),
('edtech', 'kids_edu', 'Kids Education', 'Educational content for children', '["kids", "children", "school", "primary"]')
ON CONFLICT (sub_industry_code) DO NOTHING;

-- ============================================================================
-- 7. SEED DATA - Regulatory Requirements
-- ============================================================================

-- Nigeria
INSERT INTO regulatory_requirements (country_code, country_name, requirement_type, requirement_name, description, applicable_industries, estimated_time_days, estimated_cost_min, estimated_cost_max, currency, required_documents, authority_name, authority_website, is_mandatory, priority) VALUES
('NG', 'Nigeria', 'registration', 'CAC Business Registration', 'Register business with Corporate Affairs Commission', '["all"]', 5, 10000, 25000, 'NGN', '["Proposed business name", "Director details", "Address verification"]', 'Corporate Affairs Commission (CAC)', 'https://cac.gov.ng', true, 1),
('NG', 'Nigeria', 'tax', 'Tax Identification Number (TIN)', 'Obtain TIN from Federal Inland Revenue Service', '["all"]', 3, 0, 0, 'NGN', '["Valid ID", "Proof of address"]', 'Federal Inland Revenue Service (FIRS)', 'https://firs.gov.ng', true, 2),
('NG', 'Nigeria', 'tax', 'VAT Registration', 'Register for Value Added Tax if applicable', '["all"]', 7, 0, 0, 'NGN', '["TIN", "Business registration"]', 'Federal Inland Revenue Service (FIRS)', 'https://firs.gov.ng', false, 3),
('NG', 'Nigeria', 'license', 'Fintech License', 'Required for payment and lending services', '["fintech"]', 90, 2000000, 10000000, 'NGN', '["Business plan", "Financial projections", "KYC documents"]', 'Central Bank of Nigeria (CBN)', 'https://cbn.gov.ng', true, 1),
('NG', 'Nigeria', 'license', 'Health Facility License', 'Required for healthcare services', '["healthtech"]', 30, 50000, 200000, 'NGN', '["Facility inspection", "Staff credentials", "Equipment list"]', 'Ministry of Health', '', true, 1)
ON CONFLICT DO NOTHING;

-- Kenya
INSERT INTO regulatory_requirements (country_code, country_name, requirement_type, requirement_name, description, applicable_industries, estimated_time_days, estimated_cost_min, estimated_cost_max, currency, required_documents, authority_name, authority_website, is_mandatory, priority) VALUES
('KE', 'Kenya', 'registration', 'Business Registration', 'Register with eCitizen Business Registration Service', '["all"]', 3, 1000, 5000, 'KES', '["Proposed names", "Director ID", "Address"]', 'eCitizen / BRS', 'https://business.go.ke', true, 1),
('KE', 'Kenya', 'tax', 'KRA PIN', 'Obtain Kenya Revenue Authority PIN', '["all"]', 5, 0, 0, 'KES', '["ID document", "Passport photo"]', 'Kenya Revenue Authority (KRA)', 'https://kra.go.ke', true, 2),
('KE', 'Kenya', 'license', 'County Business Permit', 'Single Business Permit from county government', '["all"]', 7, 5000, 50000, 'KES', '["Business registration", "KRA PIN"]', 'County Government', '', true, 3),
('KE', 'Kenya', 'license', 'Digital Credit Provider License', 'Required for digital lending', '["fintech"]', 60, 100000, 500000, 'KES', '["Business plan", "Financials", "Compliance manual"]', 'Central Bank of Kenya (CBK)', 'https://cbk.go.ke', true, 1)
ON CONFLICT DO NOTHING;

-- South Africa
INSERT INTO regulatory_requirements (country_code, country_name, requirement_type, requirement_name, description, applicable_industries, estimated_time_days, estimated_cost_min, estimated_cost_max, currency, required_documents, authority_name, authority_website, is_mandatory, priority) VALUES
('ZA', 'South Africa', 'registration', 'Company Registration', 'Register with Companies and Intellectual Property Commission', '["all"]', 5, 175, 475, 'ZAR', '["Company name", "Director details", "MOI"]', 'CIPC', 'https://cipc.co.za', true, 1),
('ZA', 'South Africa', 'tax', 'SARS Tax Number', 'Register for tax with South African Revenue Service', '["all"]', 10, 0, 0, 'ZAR', '["Company docs", "Director ID"]', 'SARS', 'https://sars.gov.za', true, 2),
('ZA', 'South Africa', 'license', 'B-BBEE Certificate', 'Broad-Based Black Economic Empowerment certification', '["all"]', 14, 5000, 50000, 'ZAR', '["Company docs", "Ownership structure", "Management details"]', 'SANAS Accredited Agency', '', false, 4),
('ZA', 'South Africa', 'license', 'FSP License', 'Financial Services Provider license for fintech', '["fintech"]', 120, 150000, 500000, 'ZAR', '["Fit and proper tests", "Compliance framework", "Financials"]', 'Financial Sector Conduct Authority (FSCA)', 'https://fsca.co.za', true, 1)
ON CONFLICT DO NOTHING;

-- Ghana
INSERT INTO regulatory_requirements (country_code, country_name, requirement_type, requirement_name, description, applicable_industries, estimated_time_days, estimated_cost_min, estimated_cost_max, currency, required_documents, authority_name, authority_website, is_mandatory, priority) VALUES
('GH', 'Ghana', 'registration', 'Business Registration', 'Register with Registrar General Department', '["all"]', 7, 500, 2000, 'GHS', '["Business name", "Address", "Director ID"]', 'Registrar General Department (RGD)', 'https://rgd.gov.gh', true, 1),
('GH', 'Ghana', 'tax', 'TIN Registration', 'Obtain Tax Identification Number from GRA', '["all"]', 5, 0, 0, 'GHS', '["Business registration", "ID"]', 'Ghana Revenue Authority (GRA)', 'https://gra.gov.gh', true, 2),
('GH', 'Ghana', 'license', 'Bank of Ghana Approval', 'Required for fintech and payment services', '["fintech"]', 90, 100000, 500000, 'GHS', '["Business plan", "Compliance manual", "Audit report"]', 'Bank of Ghana', 'https://bog.gov.gh', true, 1)
ON CONFLICT DO NOTHING;

-- Rwanda
INSERT INTO regulatory_requirements (country_code, country_name, requirement_type, requirement_name, description, applicable_industries, estimated_time_days, estimated_cost_min, estimated_cost_max, currency, required_documents, authority_name, authority_website, is_mandatory, priority) VALUES
('RW', 'Rwanda', 'registration', 'Company Registration', 'Register with Rwanda Development Board', '["all"]', 1, 0, 21000, 'RWF', '["Business name", "Shareholder details", "Address"]', 'Rwanda Development Board (RDB)', 'https://rdb.rw', true, 1),
('RW', 'Rwanda', 'tax', 'RRA Tax Registration', 'Register with Rwanda Revenue Authority', '["all"]', 3, 0, 0, 'RWF', '["Company registration", "Shareholder IDs"]', 'Rwanda Revenue Authority (RRA)', 'https://rra.gov.rw', true, 2),
('RW', 'Rwanda', 'license', 'Fintech License', 'License for digital financial services', '["fintech"]', 60, 50000, 200000, 'RWF', '["Business plan", "Risk management", "Compliance framework"]', 'National Bank of Rwanda (BNR)', 'https://bnr.rw', true, 1)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 8. VIEWS FOR CONVENIENCE
-- ============================================================================

-- View for complete startup blueprint
CREATE OR REPLACE VIEW startup_blueprints AS
SELECT 
    gl.id as generation_id,
    gl.user_id,
    gl.business_id,
    gl.blueprint,
    gl.status,
    gl.confidence_overall,
    gl.created_at,
    gl.completed_at,
    b.name as business_name,
    u.email as user_email
FROM generation_logs gl
LEFT JOIN businesses b ON gl.business_id = b.id
LEFT JOIN users u ON gl.user_id = u.id;

-- ============================================================================
-- END OF MIGRATION
-- ============================================================================
