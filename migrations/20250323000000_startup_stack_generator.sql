-- ============================================
-- STARTUP STACK GENERATOR
-- Database schema per specification
-- ============================================

-- ============================================
-- 1. STARTUPS TABLE (Core Entity)
-- ============================================
CREATE TABLE IF NOT EXISTS startups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    alternative_names JSONB NOT NULL DEFAULT '[]',
    tagline VARCHAR(255),
    elevator_pitch TEXT,
    mission_statement TEXT,
    vision_statement TEXT,
    industry VARCHAR(100),
    sub_industry VARCHAR(100),
    country VARCHAR(100) NOT NULL,
    secondary_countries JSONB NOT NULL DEFAULT '[]',
    founder_type VARCHAR(50) NOT NULL DEFAULT 'solo',
    business_stage VARCHAR(50) NOT NULL DEFAULT 'idea',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    progress_percentage INTEGER NOT NULL DEFAULT 0 CHECK (progress_percentage BETWEEN 0 AND 100),
    health_score INTEGER CHECK (health_score BETWEEN 0 AND 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    launched_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_startups_user ON startups(user_id);
CREATE INDEX IF NOT EXISTS idx_startups_status ON startups(status);
CREATE INDEX IF NOT EXISTS idx_startups_industry ON startups(industry);
CREATE INDEX IF NOT EXISTS idx_startups_country ON startups(country);

-- ============================================
-- 2. MILESTONES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS milestones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID NOT NULL REFERENCES startups(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    order_sequence INTEGER NOT NULL,
    estimated_days INTEGER,
    estimated_cost DECIMAL(12,2),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    completion_criteria JSONB NOT NULL DEFAULT '{}',
    depends_on_milestones JSONB NOT NULL DEFAULT '[]',
    assigned_to VARCHAR(100) DEFAULT 'founder',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_milestones_startup ON milestones(startup_id);
CREATE INDEX IF NOT EXISTS idx_milestones_status ON milestones(status);
CREATE INDEX IF NOT EXISTS idx_milestones_category ON milestones(category);
CREATE INDEX IF NOT EXISTS idx_milestones_order ON milestones(startup_id, order_sequence);

-- ============================================
-- 3. REQUIRED APPROVALS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS required_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID NOT NULL REFERENCES startups(id) ON DELETE CASCADE,
    approval_type VARCHAR(100) NOT NULL,
    name VARCHAR(255) NOT NULL,
    issuing_authority VARCHAR(255),
    authority_website TEXT,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'not_started',
    priority INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 10),
    estimated_days INTEGER,
    estimated_cost DECIMAL(12,2),
    actual_cost DECIMAL(12,2),
    documents_required JSONB NOT NULL DEFAULT '[]',
    documents_submitted JSONB NOT NULL DEFAULT '[]',
    submission_date TIMESTAMPTZ,
    approval_date TIMESTAMPTZ,
    expiry_date TIMESTAMPTZ,
    reference_number VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_approvals_startup ON required_approvals(startup_id);
CREATE INDEX IF NOT EXISTS idx_approvals_status ON required_approvals(status);
CREATE INDEX IF NOT EXISTS idx_approvals_priority ON required_approvals(priority);

-- ============================================
-- 4. SUGGESTED SERVICES TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS suggested_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID NOT NULL REFERENCES startups(id) ON DELETE CASCADE,
    service_category VARCHAR(100) NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    service_provider VARCHAR(255),
    description TEXT,
    features JSONB NOT NULL DEFAULT '[]',
    pricing_model VARCHAR(100),
    price_range VARCHAR(100),
    affiliate_link TEXT,
    website_url TEXT,
    integration_type VARCHAR(50),
    is_partner BOOLEAN NOT NULL DEFAULT false,
    partnership_benefits TEXT,
    priority INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 10),
    status VARCHAR(50) NOT NULL DEFAULT 'suggested',
    connected_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_services_startup ON suggested_services(startup_id);
CREATE INDEX IF NOT EXISTS idx_services_category ON suggested_services(service_category);
CREATE INDEX IF NOT EXISTS idx_services_status ON suggested_services(status);

-- ============================================
-- 5. STARTUP DOCUMENTS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS startup_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID NOT NULL REFERENCES startups(id) ON DELETE CASCADE,
    document_type VARCHAR(100) NOT NULL,
    document_name VARCHAR(255) NOT NULL,
    file_url TEXT,
    file_size INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(50) NOT NULL DEFAULT 'generating',
    generated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_documents_startup ON startup_documents(startup_id);
CREATE INDEX IF NOT EXISTS idx_documents_type ON startup_documents(document_type);
CREATE INDEX IF NOT EXISTS idx_documents_status ON startup_documents(status);

-- ============================================
-- 6. STARTUP METRICS TABLE
-- ============================================
CREATE TABLE IF NOT EXISTS startup_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    startup_id UUID NOT NULL REFERENCES startups(id) ON DELETE CASCADE,
    metric_type VARCHAR(100) NOT NULL,
    metric_value DECIMAL(12,2) NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_metrics_startup ON startup_metrics(startup_id);
CREATE INDEX IF NOT EXISTS idx_metrics_type ON startup_metrics(metric_type);
CREATE INDEX IF NOT EXISTS idx_metrics_recorded ON startup_metrics(recorded_at);

-- ============================================
-- 7. TRIGGERS FOR UPDATED_AT
-- ============================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers
DROP TRIGGER IF EXISTS update_startups_updated_at ON startups;
CREATE TRIGGER update_startups_updated_at BEFORE UPDATE ON startups FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_milestones_updated_at ON milestones;
CREATE TRIGGER update_milestones_updated_at BEFORE UPDATE ON milestones FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_approvals_updated_at ON required_approvals;
CREATE TRIGGER update_approvals_updated_at BEFORE UPDATE ON required_approvals FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_services_updated_at ON suggested_services;
CREATE TRIGGER update_services_updated_at BEFORE UPDATE ON suggested_services FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_startup_documents_updated_at ON startup_documents;
CREATE TRIGGER update_startup_documents_updated_at BEFORE UPDATE ON startup_documents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- 8. DEFAULT MILESTONE TEMPLATES
-- ============================================
CREATE TABLE IF NOT EXISTS milestone_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL,
    estimated_days INTEGER,
    estimated_cost DECIMAL(12,2),
    completion_criteria JSONB NOT NULL DEFAULT '{}',
    is_default BOOLEAN NOT NULL DEFAULT false,
    applicable_industries JSONB NOT NULL DEFAULT '["all"]',
    applicable_countries JSONB NOT NULL DEFAULT '["all"]',
    order_weight INTEGER NOT NULL DEFAULT 100,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default milestones for all startups
INSERT INTO milestone_templates (name, description, category, estimated_days, is_default, applicable_industries, order_weight) VALUES
('Validate Business Idea', 'Confirm your business idea is viable and has market potential', 'strategy', 2, true, '["all"]', 10),
('Register Business Entity', 'Register your business with the appropriate government authority', 'legal', 7, true, '["all"]', 20),
('Get Tax Identification Number', 'Obtain your TIN for tax compliance', 'legal', 5, true, '["all"]', 30),
('Open Business Bank Account', 'Set up a dedicated business banking account', 'finance', 7, true, '["all"]', 40),
('Create Brand Identity', 'Design logo, choose colors, establish brand guidelines', 'branding', 5, true, '["all"]', 50),
('Build Website/MVP', 'Create your online presence or minimum viable product', 'technical', 21, true, '["all"]', 60),
('Set Up Payment Processing', 'Enable customers to pay you online', 'finance', 3, true, '["all"]', 70),
('Create Marketing Strategy', 'Plan how you will acquire customers', 'marketing', 7, true, '["all"]', 80);

-- ============================================
-- 9. DEFAULT SERVICE TEMPLATES
-- ============================================
CREATE TABLE IF NOT EXISTS service_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_category VARCHAR(100) NOT NULL,
    service_name VARCHAR(255) NOT NULL,
    service_provider VARCHAR(255),
    description TEXT,
    features JSONB NOT NULL DEFAULT '[]',
    pricing_model VARCHAR(100),
    price_range VARCHAR(100),
    website_url TEXT,
    integration_type VARCHAR(50),
    is_partner BOOLEAN NOT NULL DEFAULT false,
    partnership_benefits TEXT,
    priority INTEGER NOT NULL DEFAULT 5,
    applicable_countries JSONB NOT NULL DEFAULT '["all"]',
    applicable_industries JSONB NOT NULL DEFAULT '["all"]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default suggested services
INSERT INTO service_templates (service_category, service_name, service_provider, description, pricing_model, price_range, website_url, priority) VALUES
('banking', 'Business Account', 'Flutterwave', 'Accept payments across Africa', 'transaction', '$0-50/month', 'https://flutterwave.com', 1),
('banking', 'Business Account', 'Paystack', 'Modern payment infrastructure for Africa', 'transaction', '$0-50/month', 'https://paystack.com', 2),
('accounting', 'Accounting Software', 'Wave', 'Free accounting software', 'free', 'Free', 'https://waveapps.com', 1),
('accounting', 'Accounting Software', 'QuickBooks', 'Full-featured business accounting', 'subscription', '$15-80/month', 'https://quickbooks.intuit.com', 2),
('legal', 'Legal Templates', 'Rocket Lawyer', 'Legal documents and attorney advice', 'subscription', '$39/month', 'https://rocketlawyer.com', 1),
('branding', 'Logo Design', 'Canva', 'DIY design tool for logos and branding', 'freemium', 'Free-$13/month', 'https://canva.com', 1),
('marketing', 'Email Marketing', 'Mailchimp', 'Email marketing and automation', 'freemium', 'Free-$300/month', 'https://mailchimp.com', 1),
('crm', 'CRM', 'HubSpot', 'Customer relationship management', 'freemium', 'Free-$45/month', 'https://hubspot.com', 1),
('productivity', 'Project Management', 'Trello', 'Visual project management', 'freemium', 'Free-$10/month', 'https://trello.com', 1),
('communication', 'Team Chat', 'Slack', 'Team communication platform', 'freemium', 'Free-$15/user/month', 'https://slack.com', 1);

-- ============================================
-- 10. VIEWS FOR DASHBOARD
-- ============================================

-- View: Startup overview with counts
CREATE OR REPLACE VIEW startup_overview AS
SELECT 
    s.id,
    s.name,
    s.status,
    s.progress_percentage,
    s.health_score,
    COUNT(DISTINCT m.id) FILTER (WHERE m.status = 'completed') as completed_milestones,
    COUNT(DISTINCT m.id) as total_milestones,
    COUNT(DISTINCT ra.id) FILTER (WHERE ra.status = 'approved') as completed_approvals,
    COUNT(DISTINCT ra.id) as total_approvals,
    COUNT(DISTINCT ss.id) FILTER (WHERE ss.status = 'connected') as connected_services,
    COUNT(DISTINCT ss.id) as total_services
FROM startups s
LEFT JOIN milestones m ON s.id = m.startup_id
LEFT JOIN required_approvals ra ON s.id = ra.startup_id
LEFT JOIN suggested_services ss ON s.id = ss.startup_id
GROUP BY s.id, s.name, s.status, s.progress_percentage, s.health_score;

-- View: Upcoming deadlines
CREATE OR REPLACE VIEW upcoming_deadlines AS
SELECT 
    s.id as startup_id,
    s.name as startup_name,
    m.id as milestone_id,
    m.title as milestone_title,
    m.due_date,
    m.status,
    CASE 
        WHEN m.due_date < NOW() THEN 'overdue'
        WHEN m.due_date < NOW() + INTERVAL '7 days' THEN 'this_week'
        ELSE 'upcoming'
    END as urgency
FROM startups s
JOIN milestones m ON s.id = m.startup_id
WHERE m.status NOT IN ('completed', 'skipped')
AND m.due_date IS NOT NULL;
