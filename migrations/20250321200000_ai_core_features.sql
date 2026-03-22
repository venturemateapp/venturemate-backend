-- ============================================
-- VENTUREMATE AI CORE FEATURES
-- Phase 1: AI-driven startup platform schema
-- ============================================

-- ============================================
-- 1. AI CONVERSATIONS & CHAT HISTORY
-- ============================================
CREATE TABLE IF NOT EXISTS ai_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE SET NULL,
    session_type VARCHAR(50) NOT NULL DEFAULT 'onboarding', -- onboarding, business_plan, branding, fundraising, etc.
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, completed, archived
    context JSONB NOT NULL DEFAULT '{}', -- AI context memory
    metadata JSONB NOT NULL DEFAULT '{}', -- Additional metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_conversations_user ON ai_conversations(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_conversations_business ON ai_conversations(business_id);
CREATE INDEX IF NOT EXISTS idx_ai_conversations_status ON ai_conversations(status);

-- Chat messages within conversations
CREATE TABLE IF NOT EXISTS ai_chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    ai_model VARCHAR(50), -- claude-3-opus, etc.
    tokens_used INTEGER,
    metadata JSONB NOT NULL DEFAULT '{}', -- intent, entities, actions, etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_conversation ON ai_chat_messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created ON ai_chat_messages(created_at);

-- ============================================
-- 2. AI GENERATED CONTENT REPOSITORY
-- ============================================
CREATE TABLE IF NOT EXISTS ai_generated_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    content_type VARCHAR(50) NOT NULL, -- business_plan, pitch_deck, one_pager, brand_strategy, financial_model, etc.
    status VARCHAR(20) NOT NULL DEFAULT 'generating', -- generating, completed, failed, archived
    
    -- Content storage (can be large)
    title VARCHAR(255),
    content JSONB NOT NULL DEFAULT '{}', -- Structured content
    raw_content TEXT, -- Raw text/markdown
    
    -- AI metadata
    ai_model VARCHAR(50),
    generation_params JSONB NOT NULL DEFAULT '{}',
    tokens_used INTEGER,
    generation_time_ms INTEGER,
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_version_id UUID REFERENCES ai_generated_content(id),
    
    -- User feedback
    user_rating INTEGER CHECK (user_rating BETWEEN 1 AND 5),
    user_feedback TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_content_user ON ai_generated_content(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_content_business ON ai_generated_content(business_id);
CREATE INDEX IF NOT EXISTS idx_ai_content_type ON ai_generated_content(content_type);
CREATE INDEX IF NOT EXISTS idx_ai_content_status ON ai_generated_content(status);

-- ============================================
-- 3. BRAND ASSETS (Enhanced from existing)
-- ============================================
ALTER TABLE brand_assets ADD COLUMN IF NOT EXISTS brand_strategy JSONB;
ALTER TABLE brand_assets ADD COLUMN IF NOT EXISTS voice_tone JSONB;
ALTER TABLE brand_assets ADD COLUMN IF NOT EXISTS tagline_options JSONB;
ALTER TABLE brand_assets ADD COLUMN IF NOT EXISTS brand_story TEXT;
ALTER TABLE brand_assets ADD COLUMN IF NOT EXISTS messaging_framework JSONB;

-- Brand color palettes (enhanced)
ALTER TABLE brand_colors ADD COLUMN IF NOT EXISTS color_meaning TEXT;
ALTER TABLE brand_colors ADD COLUMN IF NOT EXISTS usage_guidelines TEXT;
ALTER TABLE brand_colors ADD COLUMN IF NOT EXISTS is_ai_generated BOOLEAN DEFAULT false;

-- ============================================
-- 4. VIRTUAL CO-FOUNDER MATCHING
-- ============================================
CREATE TABLE IF NOT EXISTS cofounder_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Skills & expertise
    skills JSONB NOT NULL DEFAULT '[]', -- ["marketing", "sales", "tech", "finance"]
    expertise_areas JSONB NOT NULL DEFAULT '[]',
    experience_level VARCHAR(20), -- beginner, intermediate, expert
    
    -- Availability & commitment
    availability_hours INTEGER, -- hours per week
    commitment_type VARCHAR(20), -- full_time, part_time, advisory
    equity_expectation_min INTEGER, -- percentage
    equity_expectation_max INTEGER,
    
    -- Looking for
    looking_for_skills JSONB NOT NULL DEFAULT '[]',
    looking_for_commitment VARCHAR(20),
    preferred_industries JSONB NOT NULL DEFAULT '[]',
    
    -- Profile
    bio TEXT,
    linkedin_url TEXT,
    portfolio_url TEXT,
    location VARCHAR(100),
    remote_ok BOOLEAN DEFAULT true,
    
    -- Matching
    match_score INTEGER,
    is_active BOOLEAN DEFAULT true,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cofounder_profiles_user ON cofounder_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_cofounder_profiles_skills ON cofounder_profiles USING GIN(skills);
CREATE INDEX IF NOT EXISTS idx_cofounder_profiles_active ON cofounder_profiles(is_active) WHERE is_active = true;

-- Co-founder matches
CREATE TABLE IF NOT EXISTS cofounder_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id_1 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_id_2 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    match_score INTEGER NOT NULL,
    match_reasons JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, accepted, rejected, connected
    initiated_by UUID REFERENCES users(id),
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id_1, user_id_2)
);

CREATE INDEX IF NOT EXISTS idx_cofounder_matches_user1 ON cofounder_matches(user_id_1);
CREATE INDEX IF NOT EXISTS idx_cofounder_matches_user2 ON cofounder_matches(user_id_2);
CREATE INDEX IF NOT EXISTS idx_cofounder_matches_status ON cofounder_matches(status);

-- ============================================
-- 5. SOCIAL MEDIA & MARKETING
-- ============================================
CREATE TABLE IF NOT EXISTS social_media_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    platform VARCHAR(50) NOT NULL, -- instagram, twitter, linkedin, facebook, tiktok
    account_handle VARCHAR(100),
    account_url TEXT,
    status VARCHAR(20) DEFAULT 'pending', -- pending, connected, disconnected
    
    -- API tokens (encrypted)
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    token_expires_at TIMESTAMPTZ,
    
    -- Account metrics
    follower_count INTEGER,
    post_count INTEGER,
    engagement_rate DECIMAL(5,2),
    
    -- AI content settings
    ai_content_enabled BOOLEAN DEFAULT false,
    content_tone VARCHAR(50),
    posting_schedule JSONB, -- best times, frequency
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_social_accounts_business ON social_media_accounts(business_id);
CREATE INDEX IF NOT EXISTS idx_social_accounts_platform ON social_media_accounts(platform);

-- AI Content Calendar
CREATE TABLE IF NOT EXISTS content_calendar_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    social_account_id UUID REFERENCES social_media_accounts(id) ON DELETE SET NULL,
    
    content_type VARCHAR(50) NOT NULL, -- post, story, reel, thread
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- draft, scheduled, published, cancelled
    
    -- Content
    title VARCHAR(255),
    content TEXT,
    ai_generated_content JSONB,
    media_urls JSONB,
    
    -- Scheduling
    scheduled_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    timezone VARCHAR(50),
    
    -- Performance
    likes INTEGER DEFAULT 0,
    comments INTEGER DEFAULT 0,
    shares INTEGER DEFAULT 0,
    impressions INTEGER DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_content_calendar_business ON content_calendar_items(business_id);
CREATE INDEX IF NOT EXISTS idx_content_calendar_status ON content_calendar_items(status);
CREATE INDEX IF NOT EXISTS idx_content_calendar_scheduled ON content_calendar_items(scheduled_at);

-- ============================================
-- 6. STARTUP HEALTH SCORE
-- ============================================
CREATE TABLE IF NOT EXISTS startup_health_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Overall score (0-100)
    overall_score INTEGER NOT NULL CHECK (overall_score BETWEEN 0 AND 100),
    
    -- Component scores
    compliance_score INTEGER CHECK (compliance_score BETWEEN 0 AND 100),
    revenue_viability_score INTEGER CHECK (revenue_viability_score BETWEEN 0 AND 100),
    market_fit_score INTEGER CHECK (market_fit_score BETWEEN 0 AND 100),
    team_structure_score INTEGER CHECK (team_structure_score BETWEEN 0 AND 100),
    financial_sustainability_score INTEGER CHECK (financial_sustainability_score BETWEEN 0 AND 100),
    digital_presence_score INTEGER CHECK (digital_presence_score BETWEEN 0 AND 100),
    
    -- Score breakdown
    score_breakdown JSONB NOT NULL DEFAULT '{}',
    
    -- AI recommendations
    recommendations JSONB NOT NULL DEFAULT '[]',
    priority_actions JSONB NOT NULL DEFAULT '[]',
    
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_health_scores_business ON startup_health_scores(business_id);
CREATE INDEX IF NOT EXISTS idx_health_scores_overall ON startup_health_scores(overall_score);

-- ============================================
-- 7. SERVICE MARKETPLACE
-- ============================================
CREATE TABLE IF NOT EXISTS marketplace_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    service_type VARCHAR(50) NOT NULL, -- legal, accounting, marketing, design, dev
    title VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Pricing
    pricing_type VARCHAR(20) NOT NULL, -- fixed, hourly, package
    price_min DECIMAL(12,2),
    price_max DECIMAL(12,2),
    currency VARCHAR(3) DEFAULT 'USD',
    
    -- Details
    deliverables JSONB NOT NULL DEFAULT '[]',
    timeline_days INTEGER,
    
    -- Ratings
    rating DECIMAL(3,2),
    review_count INTEGER DEFAULT 0,
    completed_projects INTEGER DEFAULT 0,
    
    status VARCHAR(20) DEFAULT 'active', -- active, paused, suspended
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_marketplace_services_provider ON marketplace_services(provider_id);
CREATE INDEX IF NOT EXISTS idx_marketplace_services_type ON marketplace_services(service_type);
CREATE INDEX IF NOT EXISTS idx_marketplace_services_status ON marketplace_services(status);

-- Service requests/bookings
CREATE TABLE IF NOT EXISTS service_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES marketplace_services(id),
    requester_id UUID NOT NULL REFERENCES users(id),
    
    status VARCHAR(20) NOT NULL DEFAULT 'inquiry', -- inquiry, quoted, accepted, in_progress, completed, cancelled
    
    requirements TEXT,
    agreed_price DECIMAL(12,2),
    timeline_days INTEGER,
    
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_service_bookings_business ON service_bookings(business_id);
CREATE INDEX IF NOT EXISTS idx_service_bookings_status ON service_bookings(status);

-- ============================================
-- 8. SMART RECOMMENDATIONS ENGINE
-- ============================================
CREATE TABLE IF NOT EXISTS smart_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    business_id UUID REFERENCES businesses(id) ON DELETE CASCADE,
    
    recommendation_type VARCHAR(50) NOT NULL, -- funding_ready, pricing_issue, hire_needed, compliance_action, etc.
    priority VARCHAR(20) NOT NULL, -- low, medium, high, urgent
    
    title VARCHAR(255) NOT NULL,
    description TEXT,
    context JSONB NOT NULL DEFAULT '{}', -- Why this recommendation
    
    -- Action
    action_type VARCHAR(50), -- generate_document, book_service, complete_task, etc.
    action_params JSONB,
    action_url TEXT,
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'unread', -- unread, read, dismissed, actioned
    dismissed_reason TEXT,
    
    -- Timing
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_recommendations_user ON smart_recommendations(user_id);
CREATE INDEX IF NOT EXISTS idx_recommendations_business ON smart_recommendations(business_id);
CREATE INDEX IF NOT EXISTS idx_recommendations_status ON smart_recommendations(status);
CREATE INDEX IF NOT EXISTS idx_recommendations_priority ON smart_recommendations(priority);

-- ============================================
-- TRIGGERS FOR UPDATED_AT
-- ============================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers to new tables
DROP TRIGGER IF EXISTS update_ai_conversations_updated_at ON ai_conversations;
CREATE TRIGGER update_ai_conversations_updated_at BEFORE UPDATE ON ai_conversations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_ai_generated_content_updated_at ON ai_generated_content;
CREATE TRIGGER update_ai_generated_content_updated_at BEFORE UPDATE ON ai_generated_content FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_cofounder_profiles_updated_at ON cofounder_profiles;
CREATE TRIGGER update_cofounder_profiles_updated_at BEFORE UPDATE ON cofounder_profiles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_cofounder_matches_updated_at ON cofounder_matches;
CREATE TRIGGER update_cofounder_matches_updated_at BEFORE UPDATE ON cofounder_matches FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_social_accounts_updated_at ON social_media_accounts;
CREATE TRIGGER update_social_accounts_updated_at BEFORE UPDATE ON social_media_accounts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_content_calendar_updated_at ON content_calendar_items;
CREATE TRIGGER update_content_calendar_updated_at BEFORE UPDATE ON content_calendar_items FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_marketplace_services_updated_at ON marketplace_services;
CREATE TRIGGER update_marketplace_services_updated_at BEFORE UPDATE ON marketplace_services FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_service_bookings_updated_at ON service_bookings;
CREATE TRIGGER update_service_bookings_updated_at BEFORE UPDATE ON service_bookings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_smart_recommendations_updated_at ON smart_recommendations;
CREATE TRIGGER update_smart_recommendations_updated_at BEFORE UPDATE ON smart_recommendations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
