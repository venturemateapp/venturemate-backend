-- ============================================================================
-- MIGRATION: Blob Storage Support + Missing Modules
-- Purpose: Store files directly in PostgreSQL + Complete missing features
-- ============================================================================

-- ============================================================================
-- 1. BLOB STORAGE FOR FILES
-- ============================================================================

-- Create file_blobs table for storing binary data
CREATE TABLE IF NOT EXISTS file_blobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- File metadata
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    file_hash VARCHAR(64) UNIQUE NOT NULL, -- SHA256 for deduplication
    
    -- The actual binary data
    data BYTEA NOT NULL,
    
    -- Compression info
    is_compressed BOOLEAN DEFAULT FALSE,
    compression_algorithm VARCHAR(20), -- 'gzip', 'lz4', etc.
    original_size BIGINT, -- Size before compression
    
    -- Access tracking
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    
    -- Cleanup
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ, -- For temporary files
    
    -- Soft delete for cleanup jobs
    deleted_at TIMESTAMPTZ
);

-- Indexes for blob lookups
CREATE INDEX IF NOT EXISTS idx_file_blobs_hash ON file_blobs(file_hash);
CREATE INDEX IF NOT EXISTS idx_file_blobs_mime ON file_blobs(mime_type);
CREATE INDEX IF NOT EXISTS idx_file_blobs_expires ON file_blobs(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================================
-- 2. UPDATE UPLOADS TABLE FOR BLOB STORAGE
-- ============================================================================

-- Modify uploads table to reference blobs
ALTER TABLE uploads 
    ADD COLUMN IF NOT EXISTS blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS storage_path VARCHAR(500), -- Keep for compatibility, but store blob_id
    ADD COLUMN IF NOT EXISTS is_blob_stored BOOLEAN DEFAULT TRUE,
    DROP COLUMN IF EXISTS public_url;

-- Index for blob lookups
CREATE INDEX IF NOT EXISTS idx_uploads_blob ON uploads(blob_id);

-- ============================================================================
-- 3. UPDATE BRAND_ASSETS FOR BLOB STORAGE
-- ============================================================================

ALTER TABLE brand_assets
    ADD COLUMN IF NOT EXISTS blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS thumbnail_blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_brand_assets_blob ON brand_assets(blob_id);

-- ============================================================================
-- 4. USER AVATARS AS BLOBS
-- ============================================================================

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS avatar_blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL;

-- Migrate existing avatar_url if needed (will be handled in application layer)

-- ============================================================================
-- 5. WEBSITE BUILDER TABLES
-- ============================================================================

-- Website templates reference table
CREATE TABLE IF NOT EXISTS website_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Preview
    thumbnail_blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL,
    
    -- Configuration
    category VARCHAR(50),
    industries JSONB DEFAULT '[]',
    
    -- Template structure
    default_sections JSONB DEFAULT '[]',
    default_styles JSONB DEFAULT '{}',
    
    -- Features
    features JSONB DEFAULT '[]',
    
    -- Availability
    is_active BOOLEAN DEFAULT TRUE,
    is_premium BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed default templates
INSERT INTO website_templates (code, name, description, category, default_sections, features) VALUES
('startup-modern', 'Modern Startup', 'Clean, professional startup template', 'startup', 
 '[{"id": "hero", "type": "hero"}, {"id": "about", "type": "about"}, {"id": "features", "type": "features"}, {"id": "contact", "type": "contact"}]',
 '["Responsive", "SEO Optimized", "Fast Loading"]'),

('business-classic', 'Business Classic', 'Traditional business layout', 'business',
 '[{"id": "hero", "type": "hero"}, {"id": "services", "type": "services"}, {"id": "about", "type": "about"}, {"id": "testimonials", "type": "testimonials"}, {"id": "contact", "type": "contact"}]',
 '["Professional", "Corporate", "Multi-page"]'),

('creative-portfolio', 'Creative Portfolio', 'Showcase your work', 'portfolio',
 '[{"id": "hero", "type": "hero"}, {"id": "portfolio", "type": "portfolio"}, {"id": "about", "type": "about"}, {"id": "contact", "type": "contact"}]',
 '["Visual", "Gallery", "Creative"]'),

('minimal-single', 'Minimal Single Page', 'Simple one-page design', 'minimal',
 '[{"id": "hero", "type": "hero"}, {"id": "about", "type": "about"}, {"id": "contact", "type": "contact"}]',
 '["Single Page", "Minimal", "Clean"]')
ON CONFLICT (code) DO NOTHING;

-- Websites table
CREATE TABLE IF NOT EXISTS websites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Domain configuration
    subdomain VARCHAR(100) UNIQUE NOT NULL,
    custom_domain VARCHAR(255),
    domain_status VARCHAR(20) DEFAULT 'not_connected' 
        CHECK (domain_status IN ('not_connected', 'pending_dns', 'active', 'ssl_pending', 'error')),
    
    -- Template
    template_id UUID REFERENCES website_templates(id),
    template_config JSONB DEFAULT '{}',
    
    -- Global styles
    global_styles JSONB DEFAULT '{}', -- colors, fonts, spacing
    
    -- Status
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'published', 'unpublished')),
    
    -- SEO
    seo_title VARCHAR(100),
    seo_description VARCHAR(300),
    seo_keywords JSONB DEFAULT '[]',
    og_image_blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL,
    
    -- Analytics
    analytics_config JSONB DEFAULT '{}',
    
    -- Publishing
    published_at TIMESTAMPTZ,
    last_modified_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- AI generation reference
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id)
);

CREATE INDEX IF NOT EXISTS idx_websites_business ON websites(business_id);
CREATE INDEX IF NOT EXISTS idx_websites_subdomain ON websites(subdomain);
CREATE INDEX IF NOT EXISTS idx_websites_custom_domain ON websites(custom_domain) WHERE custom_domain IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_websites_status ON websites(status);

-- Website pages table
CREATE TABLE IF NOT EXISTS website_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    website_id UUID NOT NULL REFERENCES websites(id) ON DELETE CASCADE,
    
    page_key VARCHAR(50) NOT NULL, -- home, about, contact, etc.
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    
    -- Content (sections array)
    sections JSONB NOT NULL DEFAULT '[]',
    
    -- Settings
    is_enabled BOOLEAN DEFAULT TRUE,
    is_homepage BOOLEAN DEFAULT FALSE,
    order_index INTEGER DEFAULT 0,
    
    -- SEO
    seo_title VARCHAR(100),
    seo_description VARCHAR(300),
    
    -- AI generation reference
    ai_job_id UUID REFERENCES ai_generation_jobs(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(website_id, page_key)
);

CREATE INDEX IF NOT EXISTS idx_website_pages_website ON website_pages(website_id);
CREATE INDEX IF NOT EXISTS idx_website_pages_enabled ON website_pages(website_id) WHERE is_enabled = TRUE;

-- Website images/assets (stored as blobs)
CREATE TABLE IF NOT EXISTS website_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    website_id UUID NOT NULL REFERENCES websites(id) ON DELETE CASCADE,
    
    asset_name VARCHAR(100) NOT NULL,
    asset_type VARCHAR(50) NOT NULL, -- 'image', 'background', 'icon', etc.
    
    blob_id UUID NOT NULL REFERENCES file_blobs(id) ON DELETE CASCADE,
    
    -- Usage reference (which section uses this)
    section_id VARCHAR(50),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_website_assets_website ON website_assets(website_id);

-- ============================================================================
-- 6. DOCUMENT VAULT ENHANCEMENTS
-- ============================================================================

-- Ensure uploads table has all needed columns for document vault
ALTER TABLE uploads
    ADD COLUMN IF NOT EXISTS document_type VARCHAR(50), -- 'contract', 'receipt', 'legal', etc.
    ADD COLUMN IF NOT EXISTS description TEXT,
    ADD COLUMN IF NOT EXISTS version INTEGER DEFAULT 1,
    ADD COLUMN IF NOT EXISTS previous_version_id UUID REFERENCES uploads(id);

-- Document tags for organization
CREATE TABLE IF NOT EXISTS document_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    color VARCHAR(7) DEFAULT '#6366F1', -- hex color
    
    UNIQUE(business_id, name)
);

-- Link uploads to tags
CREATE TABLE IF NOT EXISTS upload_tag_links (
    upload_id UUID REFERENCES uploads(id) ON DELETE CASCADE,
    tag_id UUID REFERENCES document_tags(id) ON DELETE CASCADE,
    
    PRIMARY KEY (upload_id, tag_id)
);

-- Document sharing tokens
CREATE TABLE IF NOT EXISTS document_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upload_id UUID NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
    
    -- Share configuration
    share_token VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255), -- optional password protection
    
    -- Permissions
    allow_download BOOLEAN DEFAULT TRUE,
    allow_preview BOOLEAN DEFAULT TRUE,
    
    -- Expiration
    expires_at TIMESTAMPTZ,
    max_downloads INTEGER, -- null = unlimited
    download_count INTEGER DEFAULT 0,
    
    -- Tracking
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_document_shares_token ON document_shares(share_token);
CREATE INDEX IF NOT EXISTS idx_document_shares_upload ON document_shares(upload_id);
CREATE INDEX IF NOT EXISTS idx_document_shares_expires ON document_shares(expires_at) WHERE expires_at IS NOT NULL;

-- Document templates
CREATE TABLE IF NOT EXISTS document_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    category VARCHAR(50), -- 'legal', 'financial', 'hr', etc.
    
    -- Template file (stored as blob)
    blob_id UUID REFERENCES file_blobs(id) ON DELETE SET NULL,
    
    -- Template variables (JSON schema)
    variables_schema JSONB DEFAULT '{}',
    
    -- Availability
    country_code CHAR(2), -- null = global
    is_active BOOLEAN DEFAULT TRUE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_document_templates_category ON document_templates(category);
CREATE INDEX IF NOT EXISTS idx_document_templates_country ON document_templates(country_code);

-- ============================================================================
-- 7. GOOGLE OAUTH SUPPORT
-- ============================================================================

-- Already have google_id in users table, but let's add more OAuth providers support
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS oauth_provider VARCHAR(20), -- 'google', 'github', etc.
    ADD COLUMN IF NOT EXISTS oauth_subject VARCHAR(255), -- Provider's user ID
    ADD COLUMN IF NOT EXISTS is_oauth_user BOOLEAN DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_users_oauth ON users(oauth_provider, oauth_subject) WHERE oauth_provider IS NOT NULL;

-- OAuth state tokens (for CSRF protection during OAuth flow)
CREATE TABLE IF NOT EXISTS oauth_state_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token VARCHAR(255) UNIQUE NOT NULL,
    provider VARCHAR(20) NOT NULL,
    
    -- PKCE for security
    code_challenge VARCHAR(255),
    code_challenge_method VARCHAR(10),
    
    -- User intent
    intended_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    
    -- Expiration (short-lived, 10 minutes)
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Usage tracking
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_oauth_state_token ON oauth_state_tokens(token);
CREATE INDEX IF NOT EXISTS idx_oauth_state_expires ON oauth_state_tokens(expires_at);

-- ============================================================================
-- 8. EMAIL VERIFICATION ENHANCEMENTS
-- ============================================================================

-- Email verification is already in schema, add tracking
ALTER TABLE email_verification_tokens
    ADD COLUMN IF NOT EXISTS ip_address INET,
    ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- Email logs for tracking sends
CREATE TABLE IF NOT EXISTS email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    email_type VARCHAR(50) NOT NULL, -- 'verification', 'password_reset', 'welcome', etc.
    recipient_email VARCHAR(255) NOT NULL,
    
    -- Status
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'delivered', 'bounced', 'failed')),
    
    -- Metadata
    template_id VARCHAR(100),
    subject VARCHAR(255),
    
    -- Tracking
    sent_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    opened_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ,
    
    -- Error info
    error_message TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_logs_user ON email_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_email_logs_type ON email_logs(email_type);
CREATE INDEX IF NOT EXISTS idx_email_logs_status ON email_logs(status);

-- ============================================================================
-- 9. WEBHOOKS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Event source
    provider VARCHAR(50) NOT NULL, -- 'stripe', 'claude', 'custom', etc.
    event_type VARCHAR(100) NOT NULL,
    
    -- Event data
    payload JSONB NOT NULL,
    payload_signature VARCHAR(500), -- For verification
    
    -- Processing
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'ignored')),
    processed_at TIMESTAMPTZ,
    processing_attempts INTEGER DEFAULT 0,
    last_error TEXT,
    
    -- Idempotency
    external_id VARCHAR(255), -- Provider's event ID for deduplication
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhook_events_provider ON webhook_events(provider);
CREATE INDEX IF NOT EXISTS idx_webhook_events_type ON webhook_events(event_type);
CREATE INDEX IF NOT EXISTS idx_webhook_events_status ON webhook_events(status);
CREATE INDEX IF NOT EXISTS idx_webhook_events_external ON webhook_events(provider, external_id);

-- ============================================================================
-- 10. UPDATE TRIGGERS FOR TIMESTAMPS
-- ============================================================================

-- Function to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply to new tables
DROP TRIGGER IF EXISTS update_websites_updated_at ON websites;
CREATE TRIGGER update_websites_updated_at BEFORE UPDATE ON websites
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_website_pages_updated_at ON website_pages;
CREATE TRIGGER update_website_pages_updated_at BEFORE UPDATE ON website_pages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_document_shares_updated_at ON document_shares;
CREATE TRIGGER update_document_shares_updated_at BEFORE UPDATE ON document_shares
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_webhook_events_updated_at ON webhook_events;
CREATE TRIGGER update_webhook_events_updated_at BEFORE UPDATE ON webhook_events
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 11. FUNCTIONS FOR BLOB STORAGE
-- ============================================================================

-- Function to get file with access tracking
CREATE OR REPLACE FUNCTION get_file_blob(p_blob_id UUID)
RETURNS TABLE(
    id UUID,
    original_name VARCHAR,
    mime_type VARCHAR,
    file_size BIGINT,
    data BYTEA,
    is_compressed BOOLEAN
) AS $$
BEGIN
    -- Update access stats
    UPDATE file_blobs 
    SET access_count = access_count + 1,
        last_accessed_at = NOW()
    WHERE file_blobs.id = p_blob_id;
    
    -- Return file data
    RETURN QUERY
    SELECT fb.id, fb.original_name, fb.mime_type, fb.file_size, fb.data, fb.is_compressed
    FROM file_blobs fb
    WHERE fb.id = p_blob_id AND fb.deleted_at IS NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up expired blobs (run via cron)
CREATE OR REPLACE FUNCTION cleanup_expired_blobs()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    UPDATE file_blobs 
    SET deleted_at = NOW()
    WHERE expires_at IS NOT NULL 
      AND expires_at < NOW() 
      AND deleted_at IS NULL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
