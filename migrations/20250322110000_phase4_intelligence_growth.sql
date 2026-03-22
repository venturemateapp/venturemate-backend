-- ============================================================================
-- PHASE 4: INTELLIGENCE & GROWTH
-- Health Score, Smart Recommendations, Media Marketplace
-- ============================================================================

-- ============================================================================
-- 1. HEALTH SCORES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS health_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Component scores (0-100)
    overall_score INTEGER NOT NULL DEFAULT 0 CHECK (overall_score >= 0 AND overall_score <= 100),
    compliance_score INTEGER NOT NULL DEFAULT 0 CHECK (compliance_score >= 0 AND compliance_score <= 100),
    revenue_score INTEGER NOT NULL DEFAULT 0 CHECK (revenue_score >= 0 AND revenue_score <= 100),
    market_fit_score INTEGER NOT NULL DEFAULT 0 CHECK (market_fit_score >= 0 AND market_fit_score <= 100),
    team_score INTEGER NOT NULL DEFAULT 0 CHECK (team_score >= 0 AND team_score <= 100),
    operations_score INTEGER NOT NULL DEFAULT 0 CHECK (operations_score >= 0 AND operations_score <= 100),
    funding_readiness_score INTEGER NOT NULL DEFAULT 0 CHECK (funding_readiness_score >= 0 AND funding_readiness_score <= 100),
    
    -- Detailed breakdown
    score_breakdown JSONB DEFAULT '{}',
    -- Example: {
    --   "compliance": {"business_registration": 30, "tax_id": 20, "licenses": 15, ...},
    --   "revenue": {"bank_connected": 25, "payment_gateway": 25, ...}
    -- }
    
    contributing_factors JSONB DEFAULT '{}',
    -- Example: {
    --   "positive": ["Business registered", "Website published"],
    --   "negative": ["No payment gateway", "Incomplete team profiles"]
    -- }
    
    recommendations_count INTEGER DEFAULT 0,
    
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id)
);

CREATE INDEX idx_health_scores_business ON health_scores(business_id);
CREATE INDEX idx_health_scores_overall ON health_scores(overall_score);
CREATE INDEX idx_health_scores_calculated ON health_scores(calculated_at);

-- Trigger for updated_at
CREATE TRIGGER update_health_scores_updated_at 
    BEFORE UPDATE ON health_scores 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 2. HEALTH SCORE HISTORY TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS health_score_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    overall_score INTEGER NOT NULL,
    compliance_score INTEGER NOT NULL,
    revenue_score INTEGER NOT NULL,
    market_fit_score INTEGER NOT NULL,
    team_score INTEGER NOT NULL,
    operations_score INTEGER NOT NULL,
    funding_readiness_score INTEGER NOT NULL,
    
    calculated_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_health_score_history_business ON health_score_history(business_id);
CREATE INDEX idx_health_score_history_calculated ON health_score_history(calculated_at);

-- ============================================================================
-- 3. MARKET FIT ANALYSIS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS market_fit_analysis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    analysis_type VARCHAR(50) NOT NULL CHECK (analysis_type IN ('website_review', 'copy_review', 'social_review', 'brand_review')),
    analyzed_content TEXT,
    content_url TEXT,
    
    ai_analysis JSONB DEFAULT '{}',
    -- Example: {
    --   "clarity_score": 85,
    --   "design_score": 70,
    --   "messaging_score": 75,
    --   "trust_score": 60,
    --   "recommendations": ["Add testimonials", "Improve CTA"]
    -- }
    
    score_contribution INTEGER DEFAULT 0,
    
    analyzed_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_market_fit_analysis_business ON market_fit_analysis(business_id);
CREATE INDEX idx_market_fit_analysis_type ON market_fit_analysis(analysis_type);
CREATE INDEX idx_market_fit_analysis_expires ON market_fit_analysis(expires_at);

-- Trigger for updated_at
CREATE TRIGGER update_market_fit_analysis_updated_at 
    BEFORE UPDATE ON market_fit_analysis 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 4. SMART RECOMMENDATIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    recommendation_type VARCHAR(50) NOT NULL, -- 'compliance', 'revenue', 'market_fit', 'team', 'operations', 'timing', 'behavioral'
    trigger_source VARCHAR(100) NOT NULL, -- what triggered this recommendation
    
    -- Content
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    impact_description TEXT, -- e.g., "+15 to Revenue Score"
    
    -- Action
    cta_text VARCHAR(100),
    cta_link TEXT,
    action_type VARCHAR(50), -- 'link', 'modal', 'form'
    
    -- Priority and status
    priority VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (priority IN ('high', 'medium', 'low')),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'acted', 'dismissed', 'expired')),
    
    -- Scoring
    priority_score INTEGER DEFAULT 0, -- calculated score for sorting
    has_financial_impact BOOLEAN DEFAULT FALSE,
    unblocks_features BOOLEAN DEFAULT FALSE,
    is_time_sensitive BOOLEAN DEFAULT FALSE,
    
    -- Timestamps
    dismissed_at TIMESTAMPTZ,
    acted_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_recommendations_business ON recommendations(business_id);
CREATE INDEX idx_recommendations_status ON recommendations(status);
CREATE INDEX idx_recommendations_priority ON recommendations(priority);
CREATE INDEX idx_recommendations_type ON recommendations(recommendation_type);
CREATE INDEX idx_recommendations_created ON recommendations(created_at);

-- Trigger for updated_at
CREATE TRIGGER update_recommendations_updated_at 
    BEFORE UPDATE ON recommendations 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 5. RECOMMENDATION ACTIONS LOG
-- ============================================================================

CREATE TABLE IF NOT EXISTS recommendation_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recommendation_id UUID NOT NULL REFERENCES recommendations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL, -- 'viewed', 'clicked', 'acted', 'dismissed'
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_recommendation_actions_recommendation ON recommendation_actions(recommendation_id);
CREATE INDEX idx_recommendation_actions_user ON recommendation_actions(user_id);

-- ============================================================================
-- 6. MEDIA MARKETPLACE - SERVICE LISTINGS
-- ============================================================================

CREATE TABLE IF NOT EXISTS service_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES users(id),
    
    service_category VARCHAR(50) NOT NULL CHECK (service_category IN ('logo_design', 'social_media', 'ad_management', 'copywriting', 'web_design', 'video_production', 'business_plan', 'pitch_deck')),
    service_name VARCHAR(255) NOT NULL,
    description TEXT,
    
    pricing JSONB DEFAULT '{}',
    -- Example: {"base_price": 150, "currency": "USD", "price_tiers": [...]}
    
    delivery_time_days INTEGER,
    portfolio_urls JSONB DEFAULT '[]',
    
    -- Ratings
    rating DECIMAL(3,2) DEFAULT 0.0,
    review_count INTEGER DEFAULT 0,
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'suspended')),
    is_verified BOOLEAN DEFAULT FALSE,
    featured BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_service_listings_category ON service_listings(service_category);
CREATE INDEX idx_service_listings_provider ON service_listings(provider_id);
CREATE INDEX idx_service_listings_status ON service_listings(status);
CREATE INDEX idx_service_listings_featured ON service_listings(featured);

-- Trigger for updated_at
CREATE TRIGGER update_service_listings_updated_at 
    BEFORE UPDATE ON service_listings 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 7. MARKETPLACE ORDERS
-- ============================================================================

CREATE TABLE IF NOT EXISTS marketplace_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id),
    service_id UUID NOT NULL REFERENCES service_listings(id),
    buyer_id UUID NOT NULL REFERENCES users(id),
    provider_id UUID NOT NULL REFERENCES users(id),
    
    requirements TEXT,
    attachments JSONB DEFAULT '[]',
    
    total_amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'delivered', 'completed', 'cancelled', 'disputed')),
    
    delivery_date TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_marketplace_orders_business ON marketplace_orders(business_id);
CREATE INDEX idx_marketplace_orders_service ON marketplace_orders(service_id);
CREATE INDEX idx_marketplace_orders_buyer ON marketplace_orders(buyer_id);
CREATE INDEX idx_marketplace_orders_provider ON marketplace_orders(provider_id);
CREATE INDEX idx_marketplace_orders_status ON marketplace_orders(status);

-- Trigger for updated_at
CREATE TRIGGER update_marketplace_orders_updated_at 
    BEFORE UPDATE ON marketplace_orders 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 8. MARKETPLACE REVIEWS
-- ============================================================================

CREATE TABLE IF NOT EXISTS marketplace_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES marketplace_orders(id),
    business_id UUID NOT NULL REFERENCES businesses(id),
    provider_id UUID NOT NULL REFERENCES users(id),
    
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review_text TEXT,
    is_public BOOLEAN DEFAULT TRUE,
    response_text TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(order_id)
);

CREATE INDEX idx_marketplace_reviews_provider ON marketplace_reviews(provider_id);
CREATE INDEX idx_marketplace_reviews_rating ON marketplace_reviews(rating);

-- Trigger for updated_at
CREATE TRIGGER update_marketplace_reviews_updated_at 
    BEFORE UPDATE ON marketplace_reviews 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 9. MARKETPLACE CHAT MESSAGES
-- ============================================================================

CREATE TABLE IF NOT EXISTS marketplace_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES marketplace_orders(id),
    sender_id UUID NOT NULL REFERENCES users(id),
    
    message TEXT NOT NULL,
    attachment_url TEXT,
    is_read BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_marketplace_messages_order ON marketplace_messages(order_id);
CREATE INDEX idx_marketplace_messages_sender ON marketplace_messages(sender_id);
CREATE INDEX idx_marketplace_messages_read ON marketplace_messages(is_read);

-- ============================================================================
-- 10. AI CONTENT GENERATION
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    content_type VARCHAR(50) NOT NULL CHECK (content_type IN ('social_post', 'ad_copy', 'blog_post', 'email_copy')),
    platform VARCHAR(50), -- 'instagram', 'twitter', 'linkedin', 'facebook'
    
    generated_content TEXT NOT NULL,
    image_url TEXT,
    hashtags JSONB DEFAULT '[]',
    
    -- Scheduling
    scheduled_date TIMESTAMPTZ,
    posted_at TIMESTAMPTZ,
    
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'approved', 'scheduled', 'published')),
    
    -- Metadata
    generation_params JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_ai_content_business ON ai_content(business_id);
CREATE INDEX idx_ai_content_type ON ai_content(content_type);
CREATE INDEX idx_ai_content_platform ON ai_content(platform);
CREATE INDEX idx_ai_content_status ON ai_content(status);
CREATE INDEX idx_ai_content_scheduled ON ai_content(scheduled_date);

-- Trigger for updated_at
CREATE TRIGGER update_ai_content_updated_at 
    BEFORE UPDATE ON ai_content 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 11. HEALTH SCORE CALCULATION LOG
-- ============================================================================

CREATE TABLE IF NOT EXISTS health_score_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    calculation_type VARCHAR(50) NOT NULL, -- 'scheduled', 'manual', 'triggered'
    component_calculated VARCHAR(50), -- 'compliance', 'revenue', etc. or 'all'
    
    old_score INTEGER,
    new_score INTEGER,
    
    calculation_details JSONB DEFAULT '{}',
    -- What was calculated, what data was used
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_health_score_calculations_business ON health_score_calculations(business_id);
CREATE INDEX idx_health_score_calculations_created ON health_score_calculations(created_at);

-- ============================================================================
-- 12. VIEWS
-- ============================================================================

-- View for health score with business info
CREATE OR REPLACE VIEW health_scores_with_business AS
SELECT 
    hs.*,
    b.name as business_name,
    b.industry,
    b.business_stage
FROM health_scores hs
JOIN businesses b ON hs.business_id = b.id;

-- View for recommendations with business info
CREATE OR REPLACE VIEW recommendations_with_business AS
SELECT 
    r.*,
    b.name as business_name,
    b.user_id
FROM recommendations r
JOIN businesses b ON r.business_id = b.id;

-- View for service listings with provider info
CREATE OR REPLACE VIEW service_listings_complete AS
SELECT 
    sl.*,
    u.email as provider_email,
    up.first_name as provider_first_name,
    up.last_name as provider_last_name,
    up.avatar_url as provider_avatar
FROM service_listings sl
JOIN users u ON sl.provider_id = u.id
LEFT JOIN user_profiles up ON u.id = up.user_id
WHERE sl.status = 'active';

-- ============================================================================
-- END OF MIGRATION
-- ============================================================================
