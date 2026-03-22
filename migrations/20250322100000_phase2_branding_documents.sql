-- ============================================================================
-- PHASE 2: CORE DOCUMENT & ASSET GENERATION
-- Branding Kit, Documents, Data Room, Website Builder
-- ============================================================================

-- ============================================================================
-- 1. BRAND ASSETS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS brand_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    -- Logo assets (stored as BLOBs per requirements)
    logo_data BYTEA,
    logo_mime_type VARCHAR(100),
    logo_variants JSONB DEFAULT '{}',
    -- Example: {"full_color": "data:image/png;base64,...", "icon": "...", "white": "..."}
    
    -- Generation info
    logo_generation_prompt TEXT,
    logo_generation_model VARCHAR(50) DEFAULT 'claude',
    
    -- Color palette
    color_palette JSONB DEFAULT '{}',
    -- Example: [
    --   {"name": "Primary", "hex": "#6B46C1", "rgb": "107,70,193", "usage": "buttons, headers"},
    --   {"name": "Secondary", "hex": "#F6AD55", "usage": "accents"}
    -- ]
    
    -- Font pairings
    font_pairings JSONB DEFAULT '{}',
    -- Example: {
    --   "heading": {"name": "Poppins", "google_url": "...", "weights": [400, 600]},
    --   "body": {"name": "Open Sans", "google_url": "...", "weights": [400]}
    -- }
    
    -- Brand guidelines (PDF stored as BLOB)
    brand_guidelines_pdf BYTEA,
    
    -- Status tracking
    status VARCHAR(20) DEFAULT 'generating' CHECK (status IN ('generating', 'ready', 'failed')),
    
    -- Timestamps
    generated_at TIMESTAMPTZ,
    downloaded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id)
);

CREATE INDEX idx_brand_assets_business ON brand_assets(business_id);
CREATE INDEX idx_brand_assets_status ON brand_assets(status);

-- Trigger for updated_at
CREATE TRIGGER update_brand_assets_updated_at 
    BEFORE UPDATE ON brand_assets 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 2. BRAND ASSETS LOGS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS brand_assets_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    generation_type VARCHAR(50) NOT NULL, -- 'logo', 'palette', 'fonts', 'guidelines'
    prompt_used TEXT,
    model_used VARCHAR(50),
    response_time_ms INTEGER,
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_brand_assets_logs_business ON brand_assets_logs(business_id);
CREATE INDEX idx_brand_assets_logs_type ON brand_assets_logs(generation_type);

-- ============================================================================
-- 3. GENERATED DOCUMENTS TABLE (Enhanced)
-- ============================================================================

CREATE TABLE IF NOT EXISTS generated_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    
    -- Document info
    document_type VARCHAR(50) NOT NULL CHECK (document_type IN ('business_plan', 'pitch_deck', 'one_pager', 'brand_guidelines', 'financial_model')),
    document_name VARCHAR(255),
    
    -- File stored as BLOB (per requirements)
    file_data BYTEA,
    file_format VARCHAR(20) CHECK (file_format IN ('pdf', 'docx', 'pptx', 'xlsx')),
    file_size BIGINT,
    
    -- Version control
    version INTEGER DEFAULT 1,
    
    -- Generation params
    generation_params JSONB DEFAULT '{}',
    template_used VARCHAR(100),
    
    -- AI info
    ai_model VARCHAR(100),
    token_usage INTEGER,
    
    -- Status
    status VARCHAR(20) DEFAULT 'generating' CHECK (status IN ('generating', 'ready', 'failed', 'expired')),
    
    -- Usage tracking
    download_count INTEGER DEFAULT 0,
    
    -- Timestamps
    generated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(business_id, document_type, version)
);

CREATE INDEX idx_generated_docs_business ON generated_documents(business_id);
CREATE INDEX idx_generated_docs_type ON generated_documents(document_type);
CREATE INDEX idx_generated_docs_status ON generated_documents(status);

-- Trigger for updated_at
CREATE TRIGGER update_generated_docs_updated_at 
    BEFORE UPDATE ON generated_documents 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 4. DATA ROOMS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id UUID NOT NULL REFERENCES businesses(id) ON DELETE CASCADE,
    
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Sharing
    shareable_link VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255),
    password_protected BOOLEAN DEFAULT FALSE,
    
    -- Access control
    expires_at TIMESTAMPTZ,
    access_count INTEGER DEFAULT 0,
    download_limit INTEGER,
    
    -- Settings
    watermark_enabled BOOLEAN DEFAULT FALSE,
    watermark_text VARCHAR(255),
    
    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_data_rooms_business ON data_rooms(business_id);
CREATE INDEX idx_data_rooms_link ON data_rooms(shareable_link);

-- Trigger for updated_at
CREATE TRIGGER update_data_rooms_updated_at 
    BEFORE UPDATE ON data_rooms 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 5. DATA ROOM FILES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_room_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_room_id UUID NOT NULL REFERENCES data_rooms(id) ON DELETE CASCADE,
    
    -- Organization
    folder VARCHAR(100) NOT NULL CHECK (folder IN ('executive_summary', 'pitch_deck', 'business_plan', 'financials', 'legal', 'team', 'product', 'market_research', 'other')),
    
    -- File info
    file_name VARCHAR(255) NOT NULL,
    file_data BYTEA NOT NULL, -- BLOB storage
    file_mime_type VARCHAR(100),
    file_size BIGINT,
    
    -- Versioning
    version INTEGER DEFAULT 1,
    
    -- Metadata
    description TEXT,
    uploaded_by UUID REFERENCES users(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_data_room_files_room ON data_room_files(data_room_id);
CREATE INDEX idx_data_room_files_folder ON data_room_files(folder);

-- Trigger for updated_at
CREATE TRIGGER update_data_room_files_updated_at 
    BEFORE UPDATE ON data_room_files 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 6. DATA ROOM ACCESS LOGS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_room_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data_room_id UUID NOT NULL REFERENCES data_rooms(id) ON DELETE CASCADE,
    
    -- Access info
    ip_address INET,
    user_agent TEXT,
    email VARCHAR(255), -- If investor provided email
    
    -- Action
    action VARCHAR(50) NOT NULL, -- 'viewed', 'downloaded', 'previewed'
    file_id UUID REFERENCES data_room_files(id),
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_data_room_access_room ON data_room_access_logs(data_room_id);
CREATE INDEX idx_data_room_access_created ON data_room_access_logs(created_at);

-- ============================================================================
-- 7. WEBSITE TEMPLATES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS website_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    template_code VARCHAR(50) UNIQUE NOT NULL,
    template_name VARCHAR(100) NOT NULL,
    description TEXT,
    
    -- Category
    category VARCHAR(50) CHECK (category IN ('saas', 'ecommerce', 'marketplace', 'service', 'content', 'landing', 'portfolio')),
    
    -- Preview
    thumbnail_data BYTEA,
    preview_url TEXT,
    
    -- Structure
    default_sections JSONB DEFAULT '[]',
    -- Example: ["hero", "features", "about", "testimonials", "pricing", "contact", "footer"]
    
    -- Settings
    is_active BOOLEAN DEFAULT TRUE,
    is_premium BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_website_templates_category ON website_templates(category);
CREATE INDEX idx_website_templates_active ON website_templates(is_active);

-- Seed default templates
INSERT INTO website_templates (template_code, template_name, description, category, default_sections) VALUES
('saas-modern', 'SaaS Modern', 'Clean, modern template for software products', 'saas', '["hero", "features", "pricing", "testimonials", "faq", "cta", "footer"]'),
('ecommerce-classic', 'E-commerce Classic', 'Product-focused template with cart functionality', 'ecommerce', '["hero", "products", "features", "about", "testimonials", "footer"]'),
('marketplace-clean', 'Marketplace Clean', 'Search and listing focused marketplace template', 'marketplace', '["hero", "search", "listings", "how-it-works", "footer"]'),
('service-professional', 'Service Professional', 'Service business template with booking focus', 'service', '["hero", "services", "about", "testimonials", "contact", "footer"]'),
('landing-conversion', 'Landing Conversion', 'High-conversion single page template', 'landing', '["hero", "problem", "solution", "features", "testimonials", "pricing", "cta"]'),
('content-blog', 'Content Blog', 'Blog and content-focused template', 'content', '["hero", "featured", "categories", "about", "subscribe", "footer"]')
ON CONFLICT (template_code) DO NOTHING;

-- ============================================================================
-- 8. WEBSITES TABLE (Enhanced)
-- ============================================================================

ALTER TABLE websites ADD COLUMN IF NOT EXISTS ai_generated_copy JSONB DEFAULT '{}';
-- Example: {
--   "hero": {"headline": "...", "subheadline": "...", "cta_text": "..."},
--   "features": [{"title": "...", "description": "..."}],
--   "about": "...",
--   "testimonials": [...],
--   "faq": [...]
-- }

ALTER TABLE websites ADD COLUMN IF NOT EXISTS custom_domain_verified BOOLEAN DEFAULT FALSE;
ALTER TABLE websites ADD COLUMN IF NOT EXISTS domain_verification_code VARCHAR(100);
ALTER TABLE websites ADD COLUMN IF NOT EXISTS last_published_at TIMESTAMPTZ;
ALTER TABLE websites ADD COLUMN IF NOT EXISTS published_version INTEGER DEFAULT 0;

-- ============================================================================
-- 9. WEBSITE PAGES TABLE (Enhanced)
-- ============================================================================

ALTER TABLE website_pages ADD COLUMN IF NOT EXISTS ai_generated BOOLEAN DEFAULT FALSE;
ALTER TABLE website_pages ADD COLUMN IF NOT EXISTS generation_prompt TEXT;

-- ============================================================================
-- 10. COLOR PALETTE PRESETS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS color_palette_presets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50), -- 'fintech', 'agritech', 'healthtech', etc.
    
    -- Colors
    primary_hex VARCHAR(7) NOT NULL,
    secondary_hex VARCHAR(7),
    accent_hex VARCHAR(7),
    neutral_dark_hex VARCHAR(7),
    neutral_light_hex VARCHAR(7),
    
    -- Functional colors
    success_hex VARCHAR(7) DEFAULT '#48BB78',
    warning_hex VARCHAR(7) DEFAULT '#ED8936',
    error_hex VARCHAR(7) DEFAULT '#F56565',
    info_hex VARCHAR(7) DEFAULT '#4299E1',
    
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed industry-specific color palettes
INSERT INTO color_palette_presets (name, category, primary_hex, secondary_hex, accent_hex, neutral_dark_hex, neutral_light_hex) VALUES
('Fintech Trust', 'fintech', '#2563EB', '#7C3AED', '#10B981', '#1E293B', '#F8FAFC'),
('Agritech Growth', 'agritech', '#059669', '#D97706', '#84CC16', '#1F2937', '#F9FAFB'),
('Healthtech Care', 'healthtech', '#0D9488', '#3B82F6', '#14B8A6', '#1E293B', '#F0FDFA'),
('Edtech Learning', 'edtech', '#F59E0B', '#10B981', '#8B5CF6', '#1F2937', '#FFFBEB'),
('E-commerce Energy', 'ecommerce', '#EA580C', '#2563EB', '#DB2777', '#1F2937', '#FFF7ED'),
('SaaS Professional', 'saas', '#4F46E5', '#06B6D4', '#8B5CF6', '#111827', '#F8FAFC'),
('Marketplace Connection', 'marketplace', '#7C3AED', '#EC4899', '#F59E0B', '#1E293B', '#FAF5FF'),
('CleanTech Green', 'cleantech', '#16A34A', '#0891B2', '#84CC16', '#14532D', '#F0FDF4')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 11. FONT PAIRING PRESETS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS font_pairing_presets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    name VARCHAR(100) NOT NULL,
    category VARCHAR(50),
    
    heading_font VARCHAR(100) NOT NULL,
    heading_weights INTEGER[] DEFAULT '{400,600,700}',
    heading_google_url TEXT,
    
    body_font VARCHAR(100) NOT NULL,
    body_weights INTEGER[] DEFAULT '{400,600}',
    body_google_url TEXT,
    
    fallback_font VARCHAR(100) DEFAULT 'system-ui, -apple-system, sans-serif',
    
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed font pairings
INSERT INTO font_pairing_presets (name, category, heading_font, heading_google_url, body_font, body_google_url) VALUES
('Modern Clean', 'saas', 'Inter', 'https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap', 'Inter', 'https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap'),
('Professional Fintech', 'fintech', 'Montserrat', 'https://fonts.googleapis.com/css2?family=Montserrat:wght@400;600;700&display=swap', 'Open Sans', 'https://fonts.googleapis.com/css2?family=Open+Sans:wght@400;600&display=swap'),
('Friendly Agritech', 'agritech', 'Poppins', 'https://fonts.googleapis.com/css2?family=Poppins:wght@400;600;700&display=swap', 'Lato', 'https://fonts.googleapis.com/css2?family=Lato:wght@400;600&display=swap'),
('Trustworthy Health', 'healthtech', 'Raleway', 'https://fonts.googleapis.com/css2?family=Raleway:wght@400;600;700&display=swap', 'Source Sans Pro', 'https://fonts.googleapis.com/css2?family=Source+Sans+Pro:wght@400;600&display=swap'),
('Educational Friendly', 'edtech', 'Nunito', 'https://fonts.googleapis.com/css2?family=Nunito:wght@400;600;700&display=swap', 'Roboto', 'https://fonts.googleapis.com/css2?family=Roboto:wght@400;600&display=swap'),
('Elegant Commerce', 'ecommerce', 'Playfair Display', 'https://fonts.googleapis.com/css2?family=Playfair+Display:wght@400;600;700&display=swap', 'Inter', 'https://fonts.googleapis.com/css2?family=Inter:wght@400;600&display=swap'),
('Bold Marketplace', 'marketplace', 'Space Grotesk', 'https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;600;700&display=swap', 'Work Sans', 'https://fonts.googleapis.com/css2?family=Work+Sans:wght@400;600&display=swap')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 12. VIEWS FOR CONVENIENCE
-- ============================================================================

-- View for complete brand assets
CREATE OR REPLACE VIEW brand_assets_complete AS
SELECT 
    ba.*,
    b.name as business_name,
    b.industry,
    b.slug
FROM brand_assets ba
JOIN businesses b ON ba.business_id = b.id;

-- View for data room summary
CREATE OR REPLACE VIEW data_room_summary AS
SELECT 
    dr.*,
    b.name as business_name,
    (SELECT COUNT(*) FROM data_room_files drf WHERE drf.data_room_id = dr.id) as file_count,
    (SELECT COUNT(*) FROM data_room_access_logs dral WHERE dral.data_room_id = dr.id) as access_count
FROM data_rooms dr
JOIN businesses b ON dr.business_id = b.id;

-- ============================================================================
-- END OF MIGRATION
-- ============================================================================
