-- ============================================
-- ONBOARDING WIZARD - Full Implementation
-- Database schema per specification
-- ============================================

-- ============================================
-- 1. ONBOARDING ANSWERS TABLE
-- Stores all wizard responses per step
-- ============================================
CREATE TABLE IF NOT EXISTS onboarding_answers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    startup_id UUID REFERENCES businesses(id) ON DELETE SET NULL,
    session_id UUID NOT NULL REFERENCES onboarding_sessions(id) ON DELETE CASCADE,
    
    step_number INTEGER NOT NULL CHECK (step_number BETWEEN 1 AND 5),
    question_key VARCHAR(100) NOT NULL,
    answer_value TEXT,
    answer_json JSONB NOT NULL DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_onboarding_answers_user ON onboarding_answers(user_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_answers_startup ON onboarding_answers(startup_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_answers_session ON onboarding_answers(session_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_answers_step ON onboarding_answers(step_number);
CREATE INDEX IF NOT EXISTS idx_onboarding_answers_question ON onboarding_answers(question_key);

-- ============================================
-- 2. FOUNDERS TABLE (for Team Founder Type)
-- Co-founders with invitation system
-- ============================================
CREATE TABLE IF NOT EXISTS founders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    invited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    email VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    role VARCHAR(100),
    equity_percentage DECIMAL(5,2) CHECK (equity_percentage >= 0 AND equity_percentage <= 100),
    
    status VARCHAR(20) NOT NULL DEFAULT 'invited' 
        CHECK (status IN ('invited', 'accepted', 'declined', 'active', 'removed')),
    invitation_token VARCHAR(255) UNIQUE,
    invitation_sent_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ,
    declined_at TIMESTAMPTZ,
    declined_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_founders_startup ON founders(startup_id);
CREATE INDEX IF NOT EXISTS idx_founders_user ON founders(user_id);
CREATE INDEX IF NOT EXISTS idx_founders_invited_by ON founders(invited_by);
CREATE INDEX IF NOT EXISTS idx_founders_email ON founders(email);
CREATE INDEX IF NOT EXISTS idx_founders_status ON founders(status);
CREATE INDEX IF NOT EXISTS idx_founders_token ON founders(invitation_token);

-- ============================================
-- 3. BUSINESS IDEAS TABLE
-- Versioned business ideas with AI processing
-- ============================================
CREATE TABLE IF NOT EXISTS business_ideas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID REFERENCES businesses(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID REFERENCES onboarding_sessions(id) ON DELETE SET NULL,
    
    raw_idea_text TEXT NOT NULL,
    processed_idea_text TEXT,
    ai_enhanced_version TEXT,
    
    -- Extracted metadata
    industry VARCHAR(100),
    sub_industry VARCHAR(100),
    target_customers JSONB,
    revenue_model JSONB,
    current_stage VARCHAR(50),
    funding_status VARCHAR(50),
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_version_id UUID REFERENCES business_ideas(id),
    is_active BOOLEAN NOT NULL DEFAULT true,
    
    -- AI processing metadata
    language_detected VARCHAR(10),
    keywords JSONB,
    complexity_score INTEGER CHECK (complexity_score BETWEEN 1 AND 10),
    viability_score INTEGER CHECK (viability_score BETWEEN 1 AND 100),
    flagged_for_review BOOLEAN DEFAULT false,
    flag_reason TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_business_ideas_startup ON business_ideas(startup_id);
CREATE INDEX IF NOT EXISTS idx_business_ideas_user ON business_ideas(user_id);
CREATE INDEX IF NOT EXISTS idx_business_ideas_session ON business_ideas(session_id);
CREATE INDEX IF NOT EXISTS idx_business_ideas_active ON business_ideas(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_business_ideas_version ON business_ideas(startup_id, version);
CREATE INDEX IF NOT EXISTS idx_business_ideas_industry ON business_ideas(industry);
CREATE INDEX IF NOT EXISTS idx_business_ideas_flagged ON business_ideas(flagged_for_review) WHERE flagged_for_review = true;

-- ============================================
-- 4. SUPPORTED COUNTRIES TABLE
-- For onboarding country selection with metadata
-- ============================================
CREATE TABLE IF NOT EXISTS supported_countries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(2) NOT NULL UNIQUE,
    code_3 VARCHAR(3) NOT NULL,
    
    -- Business metadata
    currency VARCHAR(3) NOT NULL,
    currency_symbol VARCHAR(10),
    
    -- Feature flags
    is_active BOOLEAN NOT NULL DEFAULT true,
    supports_banking BOOLEAN NOT NULL DEFAULT false,
    supports_investor_matching BOOLEAN NOT NULL DEFAULT false,
    supports_marketplace BOOLEAN NOT NULL DEFAULT false,
    
    -- Compliance complexity (1-10)
    regulatory_complexity_score INTEGER CHECK (regulatory_complexity_score BETWEEN 1 AND 10),
    
    -- Available services (JSON array)
    available_services JSONB NOT NULL DEFAULT '[]',
    
    -- Regional grouping
    region VARCHAR(50),
    sub_region VARCHAR(50),
    
    -- Display
    flag_emoji VARCHAR(10),
    phone_code VARCHAR(10),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_supported_countries_code ON supported_countries(code);
CREATE INDEX IF NOT EXISTS idx_supported_countries_active ON supported_countries(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_supported_countries_region ON supported_countries(region);

-- Insert African countries per spec
INSERT INTO supported_countries (name, code, code_3, currency, currency_symbol, region, sub_region, flag_emoji, phone_code, regulatory_complexity_score, available_services, supports_banking, supports_investor_matching, supports_marketplace) VALUES
('Nigeria', 'NG', 'NGA', 'NGN', '₦', 'Africa', 'West Africa', '🇳🇬', '+234', 7, '["business_registration", "tax_filing", "banking", "investor_matching", "marketplace"]', true, true, true),
('Kenya', 'KE', 'KEN', 'KES', 'KSh', 'Africa', 'East Africa', '🇰🇪', '+254', 6, '["business_registration", "tax_filing", "banking", "investor_matching", "marketplace"]', true, true, true),
('Ghana', 'GH', 'GHA', 'GHS', 'GH₵', 'Africa', 'West Africa', '🇬🇭', '+233', 6, '["business_registration", "tax_filing", "banking", "investor_matching", "marketplace"]', true, true, true),
('South Africa', 'ZA', 'ZAF', 'ZAR', 'R', 'Africa', 'South Africa', '🇿🇦', '+27', 5, '["business_registration", "tax_filing", "banking", "investor_matching", "marketplace"]', true, true, true),
('Egypt', 'EG', 'EGY', 'EGP', 'E£', 'Africa', 'North Africa', '🇪🇬', '+20', 6, '["business_registration", "tax_filing", "banking"]', true, false, false),
('Ethiopia', 'ET', 'ETH', 'ETB', 'Br', 'Africa', 'East Africa', '🇪🇹', '+251', 8, '["business_registration"]', false, false, false),
('Uganda', 'UG', 'UGA', 'UGX', 'USh', 'Africa', 'East Africa', '🇺🇬', '+256', 6, '["business_registration", "tax_filing"]', false, false, false),
('Tanzania', 'TZ', 'TZA', 'TZS', 'TSh', 'Africa', 'East Africa', '🇹🇿', '+255', 6, '["business_registration", "tax_filing"]', false, false, false),
('Rwanda', 'RW', 'RWA', 'RWF', 'FRw', 'Africa', 'East Africa', '🇷🇼', '+250', 5, '["business_registration", "tax_filing", "banking"]', true, false, false),
('Morocco', 'MA', 'MAR', 'MAD', 'DH', 'Africa', 'North Africa', '🇲🇦', '+212', 6, '["business_registration", "tax_filing"]', false, false, false),
('Senegal', 'SN', 'SEN', 'XOF', 'CFA', 'Africa', 'West Africa', '🇸🇳', '+221', 6, '["business_registration", "tax_filing"]', false, false, false),
('Ivory Coast', 'CI', 'CIV', 'XOF', 'CFA', 'Africa', 'West Africa', '🇨🇮', '+225', 6, '["business_registration", "tax_filing"]', false, false, false),
('Zambia', 'ZM', 'ZMB', 'ZMW', 'K', 'Africa', 'Southern Africa', '🇿🇲', '+260', 6, '["business_registration", "tax_filing"]', false, false, false),
('Botswana', 'BW', 'BWA', 'BWP', 'P', 'Africa', 'Southern Africa', '🇧🇼', '+267', 5, '["business_registration", "tax_filing", "banking"]', true, false, false)
ON CONFLICT (code) DO NOTHING;

-- ============================================
-- 5. ONBOARDING ANALYTICS TABLE
-- Track wizard completion metrics
-- ============================================
CREATE TABLE IF NOT EXISTS onboarding_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES onboarding_sessions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    event_type VARCHAR(50) NOT NULL,
    step_number INTEGER,
    event_data JSONB NOT NULL DEFAULT '{}',
    
    -- Timing
    time_spent_seconds INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_onboarding_analytics_session ON onboarding_analytics(session_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_analytics_user ON onboarding_analytics(user_id);
CREATE INDEX IF NOT EXISTS idx_onboarding_analytics_event ON onboarding_analytics(event_type);
CREATE INDEX IF NOT EXISTS idx_onboarding_analytics_created ON onboarding_analytics(created_at);

-- ============================================
-- 6. UPDATE ONBOARDING SESSIONS TABLE
-- Add fields for wizard state tracking
-- ============================================
ALTER TABLE onboarding_sessions 
ADD COLUMN IF NOT EXISTS wizard_started_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS wizard_completed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS last_completed_step INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS total_time_seconds INTEGER,
ADD COLUMN IF NOT EXISTS abandonment_step INTEGER,
ADD COLUMN IF NOT EXISTS resumed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS resume_count INTEGER DEFAULT 0;

-- ============================================
-- 7. TRIGGER FOR UPDATED_AT
-- ============================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers
DROP TRIGGER IF EXISTS update_onboarding_answers_updated_at ON onboarding_answers;
CREATE TRIGGER update_onboarding_answers_updated_at BEFORE UPDATE ON onboarding_answers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_founders_updated_at ON founders;
CREATE TRIGGER update_founders_updated_at BEFORE UPDATE ON founders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_business_ideas_updated_at ON business_ideas;
CREATE TRIGGER update_business_ideas_updated_at BEFORE UPDATE ON business_ideas FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_supported_countries_updated_at ON supported_countries;
CREATE TRIGGER update_supported_countries_updated_at BEFORE UPDATE ON supported_countries FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- 8. VIEWS FOR CONVENIENCE
-- ============================================

-- View: User onboarding progress
CREATE OR REPLACE VIEW user_onboarding_progress AS
SELECT 
    u.id as user_id,
    u.email,
    os.id as session_id,
    os.status,
    os.current_step,
    os.progress_percentage,
    os.last_completed_step,
    os.wizard_started_at,
    os.wizard_completed_at,
    os.total_time_seconds,
    COUNT(DISTINCT oa.step_number) as steps_answered,
    COUNT(DISTINCT bi.id) as ideas_submitted,
    COUNT(DISTINCT f.id) as cofounders_invited
FROM users u
LEFT JOIN onboarding_sessions os ON u.id = os.user_id
LEFT JOIN onboarding_answers oa ON os.id = oa.session_id
LEFT JOIN business_ideas bi ON os.id = bi.session_id
LEFT JOIN founders f ON bi.startup_id = f.startup_id
GROUP BY u.id, u.email, os.id, os.status, os.current_step, os.progress_percentage, 
         os.last_completed_step, os.wizard_started_at, os.wizard_completed_at, os.total_time_seconds;

-- View: Pending founder invitations
CREATE OR REPLACE VIEW pending_founder_invitations AS
SELECT 
    f.id,
    f.email,
    f.full_name,
    f.role,
    f.equity_percentage,
    f.invitation_token,
    f.invitation_sent_at,
    f.status,
    b.name as startup_name,
    inviter.first_name as inviter_first_name,
    inviter.last_name as inviter_last_name,
    inviter.email as inviter_email
FROM founders f
JOIN businesses b ON f.startup_id = b.id
JOIN users inviter ON f.invited_by = inviter.id
WHERE f.status = 'invited';
