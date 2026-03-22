-- ============================================================================
-- AUTH SYSTEM COMPLIANCE MIGRATION
-- Implements full authentication system per specification
-- ============================================================================

-- ============================================================================
-- 1. UPDATE USERS TABLE - Add missing security fields
-- ============================================================================

ALTER TABLE users ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0;
ALTER TABLE users ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login_ip INET;
ALTER TABLE users ADD COLUMN IF NOT EXISTS consent_tracking JSONB DEFAULT '{}';

-- Create index for security lookups
CREATE INDEX IF NOT EXISTS idx_users_failed_login ON users(failed_login_attempts) WHERE failed_login_attempts > 0;
CREATE INDEX IF NOT EXISTS idx_users_locked ON users(locked_until) WHERE locked_until IS NOT NULL;

-- ============================================================================
-- 2. USER PROFILES TABLE - Extended user details
-- ============================================================================

CREATE TABLE IF NOT EXISTS user_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Personal Info
    date_of_birth DATE,
    city VARCHAR(100),
    
    -- Professional Info
    job_title VARCHAR(200),
    company_name VARCHAR(200),
    industry VARCHAR(100),
    years_of_experience INTEGER CHECK (years_of_experience >= 0),
    founder_type VARCHAR(50) CHECK (founder_type IN ('solo_founder', 'co_founder', 'team_member')),
    startup_experience_level VARCHAR(50) CHECK (startup_experience_level IN ('first_time', 'experienced', 'serial')),
    
    -- Preferences
    language_preference VARCHAR(10) DEFAULT 'en',
    email_notifications_enabled BOOLEAN DEFAULT TRUE,
    marketing_emails_enabled BOOLEAN DEFAULT FALSE,
    profile_visibility VARCHAR(20) DEFAULT 'private' CHECK (profile_visibility IN ('public', 'private', 'connections_only')),
    
    -- Avatar stored as BLOB (BYTEA) per requirements
    avatar_data BYTEA,
    avatar_mime_type VARCHAR(100),
    avatar_updated_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(user_id)
);

CREATE INDEX idx_user_profiles_user ON user_profiles(user_id);
CREATE INDEX idx_user_profiles_visibility ON user_profiles(profile_visibility);
CREATE INDEX idx_user_profiles_industry ON user_profiles(industry) WHERE industry IS NOT NULL;

-- Trigger for updated_at
CREATE TRIGGER update_user_profiles_updated_at 
    BEFORE UPDATE ON user_profiles 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 3. AUDIT LOGS TABLE - Security auditing
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Event details
    event_type VARCHAR(100) NOT NULL,
    event_category VARCHAR(50) NOT NULL CHECK (event_category IN ('auth', 'security', 'profile', 'business', 'system')),
    
    -- Request context
    ip_address INET,
    user_agent TEXT,
    device_fingerprint VARCHAR(255),
    
    -- Event data
    description TEXT,
    metadata JSONB DEFAULT '{}',
    
    -- For security events
    severity VARCHAR(20) DEFAULT 'info' CHECK (severity IN ('info', 'warning', 'critical')),
    
    -- Success/failure tracking
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for audit log queries
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_category ON audit_logs(event_category);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_ip ON audit_logs(ip_address);

-- Index for security monitoring queries
CREATE INDEX idx_audit_logs_security ON audit_logs(event_type, created_at) 
    WHERE event_category = 'security';

-- ============================================================================
-- 4. RATE LIMITING TABLE - For auth rate limiting
-- ============================================================================

CREATE TABLE IF NOT EXISTS rate_limit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Identifier (IP, email, or user_id)
    identifier VARCHAR(255) NOT NULL,
    identifier_type VARCHAR(50) NOT NULL CHECK (identifier_type IN ('ip', 'email', 'user_id')),
    
    -- Action being rate limited
    action VARCHAR(100) NOT NULL,
    
    -- Request context
    ip_address INET,
    user_agent TEXT,
    
    -- Result
    allowed BOOLEAN NOT NULL,
    blocked_until TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_rate_limit_logs_identifier ON rate_limit_logs(identifier, action);
CREATE INDEX idx_rate_limit_logs_created ON rate_limit_logs(created_at);
CREATE INDEX idx_rate_limit_logs_blocked ON rate_limit_logs(blocked_until) WHERE blocked_until IS NOT NULL;

-- ============================================================================
-- 5. UPDATE PASSWORD_RESETS TABLE - Add used_at tracking
-- ============================================================================

-- Already exists with used_at, just ensure it's correct
COMMENT ON COLUMN password_resets.used_at IS 'When the token was consumed';

-- ============================================================================
-- 6. UPDATE EMAIL_VERIFICATION_TOKENS TABLE - Enhance tracking
-- ============================================================================

COMMENT ON COLUMN email_verification_tokens.used_at IS 'When the token was consumed';

-- ============================================================================
-- 7. SESSIONS TABLE - Ensure all fields per spec
-- ============================================================================

-- Ensure last_activity is tracked (last_used_at already exists)
COMMENT ON COLUMN sessions.last_used_at IS 'Last activity timestamp for session';

-- Add user_id index if not exists
CREATE INDEX IF NOT EXISTS idx_sessions_user_active ON sessions(user_id, revoked_at) 
    WHERE revoked_at IS NULL;

-- ============================================================================
-- 8. FUNCTIONS FOR AUDIT LOGGING
-- ============================================================================

-- Function to auto-log auth events
CREATE OR REPLACE FUNCTION log_auth_event()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND TG_TABLE_NAME = 'sessions' THEN
        INSERT INTO audit_logs (user_id, event_type, event_category, ip_address, user_agent, description, success)
        VALUES (NEW.user_id, 'session_created', 'auth', NEW.ip_address, NEW.user_agent, 'New session created', TRUE);
    END IF;
    
    IF TG_OP = 'UPDATE' AND TG_TABLE_NAME = 'sessions' AND NEW.revoked_at IS NOT NULL AND OLD.revoked_at IS NULL THEN
        INSERT INTO audit_logs (user_id, event_type, event_category, ip_address, user_agent, description, success)
        VALUES (NEW.user_id, 'session_revoked', 'auth', NEW.ip_address, NEW.user_agent, 
                COALESCE(NEW.revoked_reason, 'Session revoked'), TRUE);
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for session logging
DROP TRIGGER IF EXISTS log_session_created ON sessions;
CREATE TRIGGER log_session_created
    AFTER INSERT ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION log_auth_event();

DROP TRIGGER IF EXISTS log_session_revoked ON sessions;
CREATE TRIGGER log_session_revoked
    AFTER UPDATE ON sessions
    FOR EACH ROW
    WHEN (NEW.revoked_at IS NOT NULL AND OLD.revoked_at IS NULL)
    EXECUTE FUNCTION log_auth_event();

-- ============================================================================
-- 9. INITIAL DATA - Create default profiles for existing users
-- ============================================================================

INSERT INTO user_profiles (user_id, language_preference, email_notifications_enabled)
SELECT id, 'en', TRUE 
FROM users 
WHERE id NOT IN (SELECT user_id FROM user_profiles)
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 10. VIEWS FOR CONVENIENT QUERIES
-- ============================================================================

-- View for user with profile
CREATE OR REPLACE VIEW user_with_profile AS
SELECT 
    u.id,
    u.email,
    u.email_verified_at,
    u.password_hash,
    u.first_name,
    u.last_name,
    u.phone,
    u.country_code,
    u.timezone,
    u.google_id,
    u.status,
    u.onboarding_completed,
    u.failed_login_attempts,
    u.locked_until,
    u.last_login_at,
    u.last_login_ip,
    u.created_at,
    u.updated_at,
    u.deleted_at,
    up.job_title,
    up.company_name,
    up.industry,
    up.years_of_experience,
    up.founder_type,
    up.startup_experience_level,
    up.language_preference,
    up.email_notifications_enabled,
    up.marketing_emails_enabled,
    up.profile_visibility,
    up.avatar_mime_type,
    up.avatar_updated_at
FROM users u
LEFT JOIN user_profiles up ON u.id = up.user_id
WHERE u.deleted_at IS NULL;

-- ============================================================================
-- END OF MIGRATION
-- ============================================================================
