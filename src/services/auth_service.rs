use chrono::{DateTime, Duration, Utc};
use ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AuthResponse, ChangePasswordRequest, CreateUserRequest, 
    EmailVerificationRequest, EmailVerificationToken,
    GoogleUserInfo, LoginRequest, MessageResponse, PasswordReset, PasswordResetRequest,
    PasswordUpdateRequest, ProfileResponse, RateLimitConfig, RateLimitResult,
    RefreshTokenRequest, ResendVerificationRequest, Session, TokenPair, 
    TokenRefreshResponse, UpdateAvatarRequest, UpdateProfileRequest, User,
    UserProfile, UserResponse, UserSession, VerificationStatusResponse,
    AUDIT_CATEGORY_AUTH, AUDIT_CATEGORY_PROFILE, AUDIT_CATEGORY_SECURITY,
    AUDIT_EVENT_ACCOUNT_LOCKED, AUDIT_EVENT_EMAIL_VERIFIED, AUDIT_EVENT_EMAIL_VERIFICATION_SENT,
    AUDIT_EVENT_LOGIN, AUDIT_EVENT_LOGIN_FAILED, AUDIT_EVENT_LOGOUT, AUDIT_EVENT_OAUTH_LOGIN,
    AUDIT_EVENT_PASSWORD_CHANGED, AUDIT_EVENT_PASSWORD_RESET, AUDIT_EVENT_PASSWORD_RESET_REQUEST,
    AUDIT_EVENT_PROFILE_UPDATED, AUDIT_EVENT_REGISTRATION, AUDIT_EVENT_SESSION_REVOKED,
    AUDIT_SEVERITY_INFO, AUDIT_SEVERITY_WARNING,
};
use crate::services::EmailService;
use crate::utils::{hash_password, verify_password, AppError, Jwt, Result};

pub struct AuthService {
    db: PgPool,
    jwt: Jwt,
    pub(crate) google_client_id: String,
    google_client_secret: String,
    email_service: Option<EmailService>,
    pub(crate) frontend_url: String,
}

impl AuthService {
    pub fn new(
        db: PgPool, 
        jwt_secret: impl Into<String>, 
        google_client_id: impl Into<String>,
        google_client_secret: impl Into<String>,
        frontend_url: impl Into<String>,
    ) -> Self {
        Self {
            db,
            jwt: Jwt::new(jwt_secret),
            google_client_id: google_client_id.into(),
            google_client_secret: google_client_secret.into(),
            email_service: None,
            frontend_url: frontend_url.into(),
        }
    }

    pub fn with_email_service(mut self, email_service: EmailService) -> Self {
        self.email_service = Some(email_service);
        self
    }

    pub async fn init_email_service(&mut self) {
        if self.email_service.is_none() {
            self.email_service = Some(EmailService::new(self.db.clone()).await);
        }
    }

    // ============================================================================
    // RATE LIMITING
    // ============================================================================

    async fn check_rate_limit(
        &self,
        identifier: &str,
        identifier_type: &str,
        action: &str,
        config: &RateLimitConfig,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<RateLimitResult> {
        let window_start = Utc::now() - Duration::seconds(config.window_seconds);
        
        // Count recent attempts
        let attempt_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM rate_limit_logs
            WHERE identifier = $1 
            AND action = $2
            AND created_at > $3
            AND allowed = true
            "#
        )
        .bind(identifier)
        .bind(action)
        .bind(window_start)
        .fetch_one(&self.db)
        .await?;

        // Check if currently blocked
        let blocked: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            SELECT blocked_until FROM rate_limit_logs
            WHERE identifier = $1 
            AND action = $2
            AND blocked_until > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .bind(identifier)
        .bind(action)
        .fetch_optional(&self.db)
        .await?
        .flatten();

        if let Some(blocked_until) = blocked {
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at: blocked_until,
                blocked_until: Some(blocked_until),
            });
        }

        let allowed = attempt_count < config.max_attempts as i64;
        let remaining = (config.max_attempts as i64 - attempt_count).max(0) as i32;
        let reset_at = Utc::now() + Duration::seconds(config.window_seconds);

        // Log the attempt
        let blocked_until = if !allowed {
            Some(Utc::now() + Duration::seconds(config.block_duration_seconds))
        } else {
            None
        };

        // Convert IpAddr to IpNetwork for PostgreSQL INET type
        let ip_network: Option<IpNetwork> = ip.map(|addr| addr.into());

        sqlx::query(
            r#"
            INSERT INTO rate_limit_logs (identifier, identifier_type, action, ip_address, user_agent, allowed, blocked_until)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(identifier)
        .bind(identifier_type)
        .bind(action)
        .bind(ip_network)
        .bind(user_agent)
        .bind(allowed)
        .bind(blocked_until)
        .execute(&self.db)
        .await?;

        Ok(RateLimitResult {
            allowed,
            remaining,
            reset_at,
            blocked_until,
        })
    }

    // ============================================================================
    // AUDIT LOGGING
    // ============================================================================

    async fn log_audit(
        &self,
        user_id: Option<Uuid>,
        event_type: &str,
        category: &str,
        description: &str,
        severity: &str,
        success: bool,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<()> {
        // Convert IpAddr to IpNetwork for PostgreSQL INET type
        let ip_network: Option<IpNetwork> = ip.map(|addr| addr.into());

        sqlx::query(
            r#"
            INSERT INTO audit_logs (user_id, event_type, event_category, description, severity, success, ip_address, user_agent, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(user_id)
        .bind(event_type)
        .bind(category)
        .bind(description)
        .bind(severity)
        .bind(success)
        .bind(ip_network)
        .bind(user_agent)
        .bind(error_message)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ============================================================================
    // ACCOUNT LOCKING
    // ============================================================================

    async fn increment_failed_login(&self, email: &str, ip: Option<std::net::IpAddr>) -> Result<bool> {
        // Get user by email
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.db)
            .await?;

        if let Some(user) = user {
            let new_count = user.failed_login_attempts + 1;
            let lock_threshold = 5;

            if new_count >= lock_threshold {
                // Lock account for 30 minutes
                let locked_until = Utc::now() + Duration::minutes(30);
                sqlx::query(
                    "UPDATE users SET failed_login_attempts = $1, locked_until = $2 WHERE id = $3"
                )
                .bind(new_count)
                .bind(locked_until)
                .bind(user.id)
                .execute(&self.db)
                .await?;

                // Log security event
                self.log_audit(
                    Some(user.id),
                    AUDIT_EVENT_ACCOUNT_LOCKED,
                    AUDIT_CATEGORY_SECURITY,
                    &format!("Account locked due to {} failed login attempts", new_count),
                    AUDIT_SEVERITY_WARNING,
                    true,
                    ip,
                    None,
                    None,
                ).await?;

                return Ok(true); // Account is now locked
            } else {
                sqlx::query(
                    "UPDATE users SET failed_login_attempts = $1 WHERE id = $2"
                )
                .bind(new_count)
                .bind(user.id)
                .execute(&self.db)
                .await?;
            }
        }

        Ok(false)
    }

    async fn reset_failed_logins(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL WHERE id = $1"
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn is_account_locked(&self, user: &User) -> bool {
        if let Some(locked_until) = user.locked_until {
            return Utc::now() < locked_until;
        }
        false
    }

    // ============================================================================
    // REGISTRATION
    // ============================================================================

    pub async fn register(
        &self, 
        req: CreateUserRequest,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<AuthResponse> {
        // Rate limit check
        let rate_limit = self.check_rate_limit(
            &ip.map(|i| i.to_string()).unwrap_or_else(|| req.email.clone()),
            if ip.is_some() { "ip" } else { "email" },
            "registration",
            &RateLimitConfig::registration(),
            ip,
            user_agent,
        ).await?;

        if !rate_limit.allowed {
            return Err(AppError::RateLimit("Too many registration attempts. Please try again later.".to_string()));
        }

        // Check if email already exists
        let existing = sqlx::query("SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL")
            .bind(&req.email)
            .fetch_optional(&self.db)
            .await?;

        if existing.is_some() {
            self.log_audit(
                None,
                AUDIT_EVENT_REGISTRATION,
                AUDIT_CATEGORY_AUTH,
                &format!("Registration failed: email {} already exists", req.email),
                AUDIT_SEVERITY_INFO,
                false,
                ip,
                user_agent,
                Some("Email already registered"),
            ).await?;
            return Err(AppError::Conflict("An account with this email already exists".to_string()));
        }

        // Hash password
        let password_hash = hash_password(&req.password)?;

        // Create user transaction
        let mut tx = self.db.begin().await?;

        // Create user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, first_name, last_name, country_code, status, consent_tracking)
            VALUES ($1, $2, $3, $4, $5, 'active', '{"registration_date": "now()"}'::jsonb)
            RETURNING *
            "#,
        )
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.country_code)
        .fetch_one(&mut *tx)
        .await?;

        // Create user profile
        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, language_preference, profile_visibility)
            VALUES ($1, 'en', 'private')
            "#
        )
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Generate tokens
        let tokens = self.create_session(user.id, ip, user_agent, 30).await?;

        // Send emails in background to not block response
        let user_id_clone = user.id;
        let email_service_clone = self.email_service.clone();
        let email = user.email.clone();
        let first_name = user.first_name.clone();
        let db_clone = self.db.clone();
        tokio::spawn(async move {
            // Send welcome email
            if let Some(ref email_service) = email_service_clone {
                let name = if first_name.is_empty() { "there" } else { &first_name };
                let _ = email_service.send_welcome(&email, user_id_clone, name).await;
            }
            
            // Create and send verification token
            // Invalidate existing tokens
            let _ = sqlx::query(
                "UPDATE email_verification_tokens SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL"
            )
            .bind(user_id_clone)
            .execute(&db_clone)
            .await;
            
            // Generate new token
            let token = generate_secure_token();
            let expires_at = Utc::now() + Duration::hours(24);
            
            let _ = sqlx::query(
                r#"
                INSERT INTO email_verification_tokens (user_id, token, expires_at)
                VALUES ($1, $2, $3)
                "#
            )
            .bind(user_id_clone)
            .bind(&token)
            .bind(expires_at)
            .execute(&db_clone)
            .await;
            
            // Send verification email
            if let Some(ref email_service) = email_service_clone {
                let _ = email_service.send_email_verification(&email, user_id_clone, &token).await;
            } else {
                tracing::info!("Email verification token for {}: {}", email, token);
            }
        });

        // Log audit
        self.log_audit(
            Some(user.id),
            AUDIT_EVENT_REGISTRATION,
            AUDIT_CATEGORY_AUTH,
            "User registration successful",
            AUDIT_SEVERITY_INFO,
            true,
            ip,
            user_agent,
            None,
        ).await?;

        let mut user_response: UserResponse = user.into();
        user_response.profile_visibility = "private".to_string();

        Ok(AuthResponse {
            user: user_response,
            tokens,
        })
    }

    // ============================================================================
    // LOGIN
    // ============================================================================

    pub async fn login(
        &self, 
        req: LoginRequest, 
        ip: Option<std::net::IpAddr>, 
        user_agent: Option<&str>,
    ) -> Result<AuthResponse> {
        // Rate limit check by IP
        let rate_limit = self.check_rate_limit(
            &ip.map(|i| i.to_string()).unwrap_or_else(|| "unknown".to_string()),
            "ip",
            "login",
            &RateLimitConfig::login(),
            ip,
            user_agent,
        ).await?;

        if !rate_limit.allowed {
            return Err(AppError::RateLimit("Too many login attempts. Please try again later.".to_string()));
        }

        // Find user
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL"
        )
        .bind(&req.email)
        .fetch_optional(&self.db)
        .await?;

        let user = match user {
            Some(u) => u,
            None => {
                self.log_audit(
                    None,
                    AUDIT_EVENT_LOGIN_FAILED,
                    AUDIT_CATEGORY_AUTH,
                    &format!("Login failed: user not found for email {}", req.email),
                    AUDIT_SEVERITY_INFO,
                    false,
                    ip,
                    user_agent,
                    Some("Invalid credentials"),
                ).await?;
                return Err(AppError::Unauthorized("Invalid email or password".to_string()));
            }
        };

        // Check if account is suspended
        if user.status == "suspended" {
            self.log_audit(
                Some(user.id),
                AUDIT_EVENT_LOGIN_FAILED,
                AUDIT_CATEGORY_AUTH,
                "Login failed: account suspended",
                AUDIT_SEVERITY_WARNING,
                false,
                ip,
                user_agent,
                Some("Account suspended"),
            ).await?;
            return Err(AppError::Forbidden("Account has been suspended".to_string()));
        }

        // Check if account is locked
        if self.is_account_locked(&user).await {
            self.log_audit(
                Some(user.id),
                AUDIT_EVENT_LOGIN_FAILED,
                AUDIT_CATEGORY_AUTH,
                "Login failed: account locked",
                AUDIT_SEVERITY_WARNING,
                false,
                ip,
                user_agent,
                Some("Account temporarily locked"),
            ).await?;
            return Err(AppError::Forbidden("Account is temporarily locked due to too many failed attempts. Please try again later.".to_string()));
        }

        // Verify password
        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        if !verify_password(&req.password, password_hash)? {
            // Increment failed login count
            let is_locked = self.increment_failed_login(&req.email, ip).await?;
            
            let error_msg = if is_locked {
                "Account is temporarily locked due to too many failed attempts"
            } else {
                "Invalid email or password"
            };

            self.log_audit(
                Some(user.id),
                AUDIT_EVENT_LOGIN_FAILED,
                AUDIT_CATEGORY_AUTH,
                "Login failed: invalid password",
                AUDIT_SEVERITY_INFO,
                false,
                ip,
                user_agent,
                Some("Invalid credentials"),
            ).await?;

            return Err(AppError::Unauthorized(error_msg.to_string()));
        }

        // Check if email is verified
        if user.email_verified_at.is_none() {
            self.log_audit(
                Some(user.id),
                AUDIT_EVENT_LOGIN_FAILED,
                AUDIT_CATEGORY_AUTH,
                "Login failed: email not verified",
                AUDIT_SEVERITY_INFO,
                false,
                ip,
                user_agent,
                Some("Email not verified"),
            ).await?;
            return Err(AppError::Forbidden("Please verify your email before logging in".to_string()));
        }

        // Reset failed login count
        self.reset_failed_logins(user.id).await?;

        // Update last login info
        let ip_network: Option<IpNetwork> = ip.map(|addr| addr.into());
        let ip_str = ip.map(|i| i.to_string());
        sqlx::query(
            "UPDATE users SET last_login_at = NOW(), last_login_ip = $1 WHERE id = $2"
        )
        .bind(ip_network)
        .bind(user.id)
        .execute(&self.db)
        .await?;

        // Generate tokens with remember me option
        let refresh_days = if req.remember_me == Some(true) { 30 } else { 7 };
        let tokens = self.create_session(user.id, ip, user_agent, refresh_days).await?;

        // Get full user response with profile
        let user_response = self.get_user_response(user.id).await?;

        // Log successful login
        self.log_audit(
            Some(user.id),
            AUDIT_EVENT_LOGIN,
            AUDIT_CATEGORY_AUTH,
            "User login successful",
            AUDIT_SEVERITY_INFO,
            true,
            ip,
            user_agent,
            None,
        ).await?;

        // Send login alert email
        if let Some(ref email_service) = self.email_service {
            let ip_display = ip_str.unwrap_or_else(|| "Unknown".to_string());
            let ua_display = user_agent.unwrap_or("Unknown").to_string();
            let _ = email_service.send_login_alert(
                &user.email, 
                user.id, 
                &ip_display, 
                &ua_display, 
                Utc::now()
            ).await;
        }

        Ok(AuthResponse {
            user: user_response,
            tokens,
        })
    }

    // ============================================================================
    // GOOGLE OAUTH
    // ============================================================================

    pub async fn google_oauth(
        &self, 
        code: &str,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<AuthResponse> {
        // Exchange code for tokens
        let token_response = self.exchange_google_code(code).await?;
        
        // Get user info from Google
        let google_user = self.get_google_user_info(&token_response.access_token).await?;

        // Check rate limit
        let rate_limit = self.check_rate_limit(
            &ip.map(|i| i.to_string()).unwrap_or_else(|| "unknown".to_string()),
            "ip",
            "oauth_login",
            &RateLimitConfig::login(),
            ip,
            user_agent,
        ).await?;

        if !rate_limit.allowed {
            return Err(AppError::RateLimit("Too many login attempts".to_string()));
        }

        // Check if user exists
        let existing_user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE google_id = $1 OR email = $2"
        )
        .bind(&google_user.sub)
        .bind(&google_user.email)
        .fetch_optional(&self.db)
        .await?;

        let user = match existing_user {
            Some(mut user) => {
                // Link Google ID if not set
                if user.google_id.is_none() {
                    user = sqlx::query_as::<_, User>(
                        "UPDATE users SET google_id = $1, email_verified_at = COALESCE(email_verified_at, NOW()) WHERE id = $2 RETURNING *"
                    )
                    .bind(&google_user.sub)
                    .bind(user.id)
                    .fetch_one(&self.db)
                    .await?;
                }

                // Check account status
                if user.status == "suspended" {
                    return Err(AppError::Forbidden("Account has been suspended".to_string()));
                }

                user
            }
            None => {
                // Create new user from Google data
                let first_name = google_user.given_name.unwrap_or_else(|| {
                    google_user.name.as_ref()
                        .and_then(|n| n.split_whitespace().next())
                        .unwrap_or("User")
                        .to_string()
                });
                let last_name = google_user.family_name.unwrap_or_else(|| {
                    google_user.name.as_ref()
                        .and_then(|n| n.split_whitespace().nth(1))
                        .unwrap_or("")
                        .to_string()
                });

                let mut tx = self.db.begin().await?;

                let new_user = sqlx::query_as::<_, User>(
                    r#"
                    INSERT INTO users (
                        email, first_name, last_name, google_id, 
                        avatar_url, email_verified_at, status, country_code
                    )
                    VALUES ($1, $2, $3, $4, $5, NOW(), 'active', 'ZA')
                    RETURNING *
                    "#
                )
                .bind(&google_user.email)
                .bind(&first_name)
                .bind(&last_name)
                .bind(&google_user.sub)
                .bind(google_user.picture)
                .fetch_one(&mut *tx)
                .await?;

                // Create profile
                sqlx::query(
                    "INSERT INTO user_profiles (user_id, language_preference, profile_visibility) VALUES ($1, 'en', 'private')"
                )
                .bind(new_user.id)
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;

                // Send welcome email
                if let Some(ref email_service) = self.email_service {
                    let _ = email_service.send_welcome(&new_user.email, new_user.id, &first_name).await;
                }

                new_user
            }
        };

        // Update last login
        let ip_network2: Option<IpNetwork> = ip.map(|addr| addr.into());
        sqlx::query(
            "UPDATE users SET last_login_at = NOW(), last_login_ip = $1 WHERE id = $2"
        )
        .bind(ip_network2)
        .bind(user.id)
        .execute(&self.db)
        .await?;

        // Generate tokens
        let tokens = self.create_session(user.id, ip, user_agent, 30).await?;

        let user_response = self.get_user_response(user.id).await?;

        // Log OAuth login
        self.log_audit(
            Some(user.id),
            AUDIT_EVENT_OAUTH_LOGIN,
            AUDIT_CATEGORY_AUTH,
            "Google OAuth login successful",
            AUDIT_SEVERITY_INFO,
            true,
            ip,
            user_agent,
            None,
        ).await?;

        Ok(AuthResponse {
            user: user_response,
            tokens,
        })
    }

    async fn exchange_google_code(&self, code: &str) -> Result<OAuthTokenResponse> {
        let client = reqwest::Client::new();
        let params = [
            ("code", code),
            ("client_id", &self.google_client_id),
            ("client_secret", &self.google_client_secret),
            ("redirect_uri", &format!("{}/auth/google/callback", self.frontend_url)),
            ("grant_type", "authorization_code"),
        ];

        let response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to exchange Google code: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApi(format!("Google OAuth error: {}", error_text)));
        }

        let token_response: OAuthTokenResponse = response.json().await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Google token: {}", e)))?;

        Ok(token_response)
    }

    async fn get_google_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to get Google user info: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Unauthorized("Invalid Google token".to_string()));
        }

        let user_info: GoogleUserInfo = response.json().await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Google user: {}", e)))?;

        Ok(user_info)
    }

    // ============================================================================
    // SESSION MANAGEMENT
    // ============================================================================

    async fn create_session(
        &self,
        user_id: Uuid,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
        refresh_days: i64,
    ) -> Result<TokenPair> {
        // Generate JWT tokens with session ID
        let session_id = Uuid::new_v4();
        let (access_token, access_exp) = self.jwt.generate_access_token_with_session(user_id, session_id)?;
        let (refresh_token, _) = self.jwt.generate_refresh_token_with_session(user_id, session_id, refresh_days)?;

        // Hash tokens for storage
        let access_hash = hash_password(&access_token)?;
        let refresh_hash = hash_password(&refresh_token)?;

        // Store session
        let expires_at = Utc::now() + Duration::hours(1);
        let refresh_expires_at = Utc::now() + Duration::days(refresh_days);

        // Convert IpAddr to IpNetwork for PostgreSQL INET type
        let ip_network: Option<IpNetwork> = ip.map(|addr| addr.into());

        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, token_hash, refresh_token_hash, ip_address, user_agent, expires_at, refresh_expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(session_id)
        .bind(user_id)
        .bind(&access_hash)
        .bind(&refresh_hash)
        .bind(ip_network)
        .bind(user_agent)
        .bind(expires_at)
        .bind(refresh_expires_at)
        .execute(&self.db)
        .await?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: access_exp - Utc::now().timestamp(),
        })
    }

    pub async fn refresh_token(&self, req: RefreshTokenRequest) -> Result<TokenRefreshResponse> {
        // Verify refresh token
        let claims = self.jwt.verify_token(&req.refresh_token)?;

        let session_id = Uuid::parse_str(&claims.session_id)
            .map_err(|_| AppError::Unauthorized("Invalid session".to_string()))?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID".to_string()))?;

        // Find session by checking refresh token hash
        let sessions = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE user_id = $1 AND revoked_at IS NULL AND refresh_expires_at > NOW()"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let session = sessions.into_iter()
            .find(|s| verify_password(&req.refresh_token, &s.refresh_token_hash).unwrap_or(false))
            .ok_or_else(|| AppError::Unauthorized("Invalid or expired session".to_string()))?;

        if session.id != session_id {
            return Err(AppError::Unauthorized("Session mismatch".to_string()));
        }

        // Check if user is still active
        let user_status: String = sqlx::query_scalar(
            "SELECT status FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        if user_status != "active" {
            return Err(AppError::Forbidden("Account is not active".to_string()));
        }

        // Generate new access token
        let (access_token, access_exp) = self.jwt.generate_access_token_with_session(user_id, session_id)?;
        let access_hash = hash_password(&access_token)?;

        // Update session with new access token and last activity
        sqlx::query(
            "UPDATE sessions SET token_hash = $1, last_used_at = NOW() WHERE id = $2"
        )
        .bind(&access_hash)
        .bind(session_id)
        .execute(&self.db)
        .await?;

        Ok(TokenRefreshResponse {
            access_token,
            expires_in: access_exp - Utc::now().timestamp(),
        })
    }

    pub async fn logout(&self, user_id: Uuid, token: &str) -> Result<()> {
        // Find session by token hash
        let sessions = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        for session in sessions {
            if verify_password(token, &session.token_hash).unwrap_or(false) {
                // Revoke this session
                sqlx::query(
                    "UPDATE sessions SET revoked_at = NOW(), revoked_reason = 'logout' WHERE id = $1"
                )
                .bind(session.id)
                .execute(&self.db)
                .await?;

                self.log_audit(
                    Some(user_id),
                    AUDIT_EVENT_LOGOUT,
                    AUDIT_CATEGORY_AUTH,
                    "User logged out",
                    AUDIT_SEVERITY_INFO,
                    true,
                    None,
                    None,
                    None,
                ).await?;

                return Ok(());
            }
        }

        Ok(())
    }

    pub async fn revoke_session(&self, user_id: Uuid, session_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE sessions SET revoked_at = NOW(), revoked_reason = 'user_revoked' WHERE id = $1 AND user_id = $2"
        )
        .bind(session_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        self.log_audit(
            Some(user_id),
            AUDIT_EVENT_SESSION_REVOKED,
            AUDIT_CATEGORY_AUTH,
            &format!("Session {} revoked by user", session_id),
            AUDIT_SEVERITY_INFO,
            true,
            None,
            None,
            None,
        ).await?;

        Ok(())
    }

    pub async fn list_active_sessions(&self, user_id: Uuid, current_session_id: Option<Uuid>) -> Result<Vec<UserSession>> {
        let sessions = sqlx::query_as::<_, Session>(
            r#"
            SELECT * FROM sessions 
            WHERE user_id = $1 
            AND revoked_at IS NULL 
            AND refresh_expires_at > NOW()
            ORDER BY last_used_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let user_sessions: Vec<UserSession> = sessions.into_iter()
            .map(|s| UserSession {
                id: s.id,
                device_info: s.user_agent,
                ip_address: s.ip_address.map(|ip| ip.to_string()),
                created_at: s.created_at,
                last_used_at: s.last_used_at,
                is_current: current_session_id.map(|id| id == s.id).unwrap_or(false),
            })
            .collect();

        Ok(user_sessions)
    }

    // ============================================================================
    // PASSWORD RESET
    // ============================================================================

    pub async fn request_password_reset(
        &self, 
        req: PasswordResetRequest,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<MessageResponse> {
        // Rate limit check
        let rate_limit = self.check_rate_limit(
            &req.email,
            "email",
            "password_reset",
            &RateLimitConfig::password_reset(),
            ip,
            user_agent,
        ).await?;

        if !rate_limit.allowed {
            // Return generic success to prevent email enumeration
            return Ok(MessageResponse {
                message: "If an account exists with this email, you will receive a password reset link.".to_string(),
            });
        }

        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL"
        )
        .bind(&req.email)
        .fetch_optional(&self.db)
        .await?;

        if let Some(user) = user {
            // Generate reset token
            let token = generate_secure_token();
            let token_hash = hash_password(&token)?;
            let expires_at = Utc::now() + Duration::hours(1);

            // Invalidate existing tokens
            sqlx::query(
                "UPDATE password_resets SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL"
            )
            .bind(user.id)
            .execute(&self.db)
            .await?;

            // Create new token
            sqlx::query(
                r#"
                INSERT INTO password_resets (user_id, token_hash, expires_at)
                VALUES ($1, $2, $3)
                "#
            )
            .bind(user.id)
            .bind(&token_hash)
            .bind(expires_at)
            .execute(&self.db)
            .await?;

            // Send email
            if let Some(ref email_service) = self.email_service {
                let _ = email_service.send_password_reset(&user.email, user.id, &token, expires_at).await;
            } else {
                tracing::info!("Password reset token for {}: {}", req.email, token);
            }

            self.log_audit(
                Some(user.id),
                AUDIT_EVENT_PASSWORD_RESET_REQUEST,
                AUDIT_CATEGORY_SECURITY,
                "Password reset requested",
                AUDIT_SEVERITY_INFO,
                true,
                ip,
                user_agent,
                None,
            ).await?;
        }

        // Always return same message to prevent email enumeration
        Ok(MessageResponse {
            message: "If an account exists with this email, you will receive a password reset link.".to_string(),
        })
    }

    pub async fn reset_password(&self, req: PasswordUpdateRequest) -> Result<MessageResponse> {
        // Validate password confirmation
        if req.new_password != req.confirm_password {
            return Err(AppError::BadRequest("Passwords do not match".to_string()));
        }

        // Find valid reset token
        let reset_records = sqlx::query_as::<_, PasswordReset>(
            r#"
            SELECT * FROM password_resets
            WHERE expires_at > NOW()
            AND used_at IS NULL
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        let reset = reset_records.into_iter()
            .find(|r| verify_password(&req.token, &r.token_hash).unwrap_or(false))
            .ok_or_else(|| AppError::BadRequest("Invalid or expired reset token".to_string()))?;

        // Hash new password
        let new_password_hash = hash_password(&req.new_password)?;

        let mut tx = self.db.begin().await?;

        // Update user password
        sqlx::query(
            "UPDATE users SET password_hash = $1 WHERE id = $2"
        )
        .bind(&new_password_hash)
        .bind(reset.user_id)
        .execute(&mut *tx)
        .await?;

        // Mark token as used
        sqlx::query(
            "UPDATE password_resets SET used_at = NOW() WHERE id = $1"
        )
        .bind(reset.id)
        .execute(&mut *tx)
        .await?;

        // Revoke all sessions
        sqlx::query(
            "UPDATE sessions SET revoked_at = NOW(), revoked_reason = 'password_reset' WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(reset.user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Get user for email
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(reset.user_id)
            .fetch_one(&self.db)
            .await?;

        // Send confirmation
        if let Some(ref email_service) = self.email_service {
            let _ = email_service.send_password_changed_confirmation(&user.email, user.id).await;
        }

        self.log_audit(
            Some(reset.user_id),
            AUDIT_EVENT_PASSWORD_RESET,
            AUDIT_CATEGORY_SECURITY,
            "Password reset completed",
            AUDIT_SEVERITY_INFO,
            true,
            None,
            None,
            None,
        ).await?;

        Ok(MessageResponse {
            message: "Password has been reset successfully. Please log in with your new password.".to_string(),
        })
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        req: ChangePasswordRequest,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<MessageResponse> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&self.db)
            .await?;

        // Verify current password
        let current_hash = user
            .password_hash
            .ok_or_else(|| AppError::BadRequest("No password set".to_string()))?;

        if !verify_password(&req.current_password, &current_hash)? {
            self.log_audit(
                Some(user_id),
                AUDIT_EVENT_PASSWORD_CHANGED,
                AUDIT_CATEGORY_SECURITY,
                "Password change failed: incorrect current password",
                AUDIT_SEVERITY_WARNING,
                false,
                ip,
                user_agent,
                Some("Incorrect current password"),
            ).await?;
            return Err(AppError::Unauthorized("Current password is incorrect".to_string()));
        }

        // Hash and update new password
        let new_hash = hash_password(&req.new_password)?;
        
        let mut tx = self.db.begin().await?;

        sqlx::query(
            "UPDATE users SET password_hash = $1 WHERE id = $2"
        )
        .bind(&new_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Revoke all other sessions (force re-login)
        sqlx::query(
            "UPDATE sessions SET revoked_at = NOW(), revoked_reason = 'password_change' WHERE user_id = $1 AND revoked_at IS NULL"
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Send confirmation
        if let Some(ref email_service) = self.email_service {
            let _ = email_service.send_password_changed_confirmation(&user.email, user.id).await;
        }

        self.log_audit(
            Some(user_id),
            AUDIT_EVENT_PASSWORD_CHANGED,
            AUDIT_CATEGORY_SECURITY,
            "Password changed successfully",
            AUDIT_SEVERITY_INFO,
            true,
            ip,
            user_agent,
            None,
        ).await?;

        Ok(MessageResponse {
            message: "Password changed successfully. Please log in again.".to_string(),
        })
    }

    // ============================================================================
    // EMAIL VERIFICATION
    // ============================================================================

    async fn create_email_verification(&self, user_id: Uuid, email: &str) -> Result<String> {
        // Invalidate existing tokens
        sqlx::query(
            "UPDATE email_verification_tokens SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL"
        )
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Generate new token (24 hour expiry per spec)
        let token = generate_secure_token();
        let expires_at = Utc::now() + Duration::hours(24);

        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (user_id, token, expires_at)
            VALUES ($1, $2, $3)
            "#
        )
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.db)
        .await?;

        // Send email
        if let Some(ref email_service) = self.email_service {
            let _ = email_service.send_email_verification(email, user_id, &token).await;
        } else {
            tracing::info!("Email verification token for {}: {}", email, token);
        }

        self.log_audit(
            Some(user_id),
            AUDIT_EVENT_EMAIL_VERIFICATION_SENT,
            AUDIT_CATEGORY_AUTH,
            "Verification email sent",
            AUDIT_SEVERITY_INFO,
            true,
            None,
            None,
            None,
        ).await?;

        Ok(token)
    }

    pub async fn verify_email(&self, req: EmailVerificationRequest) -> Result<MessageResponse> {
        let verification = sqlx::query_as::<_, EmailVerificationToken>(
            r#"
            SELECT * FROM email_verification_tokens
            WHERE token = $1
            AND expires_at > NOW()
            AND used_at IS NULL
            "#
        )
        .bind(&req.token)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired verification token".to_string()))?;

        let mut tx = self.db.begin().await?;

        // Mark email as verified
        sqlx::query(
            "UPDATE users SET email_verified_at = NOW() WHERE id = $1"
        )
        .bind(verification.user_id)
        .execute(&mut *tx)
        .await?;

        // Mark token as used
        sqlx::query(
            "UPDATE email_verification_tokens SET used_at = NOW() WHERE id = $1"
        )
        .bind(verification.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Get user for email
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(verification.user_id)
            .fetch_one(&self.db)
            .await?;

        if let Some(ref email_service) = self.email_service {
            let _ = email_service.send_email_verified_confirmation(&user.email, user.id).await;
        }

        self.log_audit(
            Some(verification.user_id),
            AUDIT_EVENT_EMAIL_VERIFIED,
            AUDIT_CATEGORY_AUTH,
            "Email verified successfully",
            AUDIT_SEVERITY_INFO,
            true,
            None,
            None,
            None,
        ).await?;

        Ok(MessageResponse {
            message: "Email verified successfully. You can now log in.".to_string(),
        })
    }

    pub async fn resend_verification(
        &self, 
        req: ResendVerificationRequest,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<VerificationStatusResponse> {
        // Rate limit check
        let rate_limit = self.check_rate_limit(
            &req.email,
            "email",
            "verification_resend",
            &RateLimitConfig::verification_resend(),
            ip,
            user_agent,
        ).await?;

        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL"
        )
        .bind(&req.email)
        .fetch_optional(&self.db)
        .await?;

        let can_resend_at = if rate_limit.blocked_until.is_some() {
            rate_limit.blocked_until
        } else {
            None
        };

        if let Some(user) = user {
            if user.email_verified_at.is_some() {
                return Ok(VerificationStatusResponse {
                    email: user.email,
                    verified: true,
                    resent_at: None,
                    can_resend_at: None,
                });
            }

            if rate_limit.allowed {
                self.create_email_verification(user.id, &user.email).await?;
                
                return Ok(VerificationStatusResponse {
                    email: user.email,
                    verified: false,
                    resent_at: Some(Utc::now()),
                    can_resend_at: Some(Utc::now() + Duration::hours(1)),
                });
            }
        }

        // Return generic response to prevent enumeration
        Ok(VerificationStatusResponse {
            email: req.email,
            verified: false,
            resent_at: if rate_limit.allowed { Some(Utc::now()) } else { None },
            can_resend_at,
        })
    }

    // ============================================================================
    // PROFILE MANAGEMENT
    // ============================================================================

    pub async fn get_user_response(&self, user_id: Uuid) -> Result<UserResponse> {
        use sqlx::Row;
        
        let row = sqlx::query(
            r#"
            SELECT 
                u.id, u.email, u.first_name, u.last_name, u.phone, 
                u.country_code, u.timezone, u.email_verified_at, 
                u.onboarding_completed, u.created_at, u.updated_at,
                up.job_title, up.company_name, up.industry, up.profile_visibility
            FROM users u
            LEFT JOIN user_profiles up ON u.id = up.user_id
            WHERE u.id = $1 AND u.deleted_at IS NULL
            "#
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        let email_verified_at: Option<DateTime<Utc>> = row.try_get("email_verified_at")?;
        let profile_visibility: Option<String> = row.try_get("profile_visibility")?;

        let user_response = UserResponse {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            first_name: row.try_get("first_name")?,
            last_name: row.try_get("last_name")?,
            avatar_url: None,
            email_verified: email_verified_at.is_some(),
            phone: row.try_get("phone")?,
            country_code: row.try_get("country_code")?,
            timezone: row.try_get("timezone")?,
            subscription_tier: None,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            onboarding_completed: row.try_get("onboarding_completed")?,
            businesses_count: 0,
            job_title: row.try_get("job_title")?,
            company_name: row.try_get("company_name")?,
            industry: row.try_get("industry")?,
            profile_visibility: profile_visibility.unwrap_or_else(|| "private".to_string()),
        };

        Ok(user_response)
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<ProfileResponse> {
        let profile = sqlx::query_as::<_, UserProfile>(
            "SELECT * FROM user_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        match profile {
            Some(p) => Ok(p.into()),
            None => Err(AppError::NotFound("Profile not found".to_string())),
        }
    }

    pub async fn update_profile(
        &self, 
        user_id: Uuid, 
        req: UpdateProfileRequest,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<ProfileResponse> {
        let mut tx = self.db.begin().await?;

        // Update users table fields using individual queries to avoid trait object issues
        if let Some(ref first_name) = req.first_name {
            sqlx::query("UPDATE users SET first_name = $1 WHERE id = $2")
                .bind(first_name)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref last_name) = req.last_name {
            sqlx::query("UPDATE users SET last_name = $1 WHERE id = $2")
                .bind(last_name)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref phone) = req.phone {
            sqlx::query("UPDATE users SET phone = $1 WHERE id = $2")
                .bind(phone)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref timezone) = req.timezone {
            sqlx::query("UPDATE users SET timezone = $1 WHERE id = $2")
                .bind(timezone)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;
        }

        // Update or create profile
        let profile_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_profiles WHERE user_id = $1)"
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        if profile_exists {
            // Update profile fields individually
            if let Some(ref dob) = req.date_of_birth {
                sqlx::query("UPDATE user_profiles SET date_of_birth = $1 WHERE user_id = $2")
                    .bind(dob)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref city) = req.city {
                sqlx::query("UPDATE user_profiles SET city = $1 WHERE user_id = $2")
                    .bind(city)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref job) = req.job_title {
                sqlx::query("UPDATE user_profiles SET job_title = $1 WHERE user_id = $2")
                    .bind(job)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref company) = req.company_name {
                sqlx::query("UPDATE user_profiles SET company_name = $1 WHERE user_id = $2")
                    .bind(company)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref industry) = req.industry {
                sqlx::query("UPDATE user_profiles SET industry = $1 WHERE user_id = $2")
                    .bind(industry)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref exp) = req.years_of_experience {
                sqlx::query("UPDATE user_profiles SET years_of_experience = $1 WHERE user_id = $2")
                    .bind(exp)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref founder) = req.founder_type {
                sqlx::query("UPDATE user_profiles SET founder_type = $1 WHERE user_id = $2")
                    .bind(founder)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref level) = req.startup_experience_level {
                sqlx::query("UPDATE user_profiles SET startup_experience_level = $1 WHERE user_id = $2")
                    .bind(level)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref lang) = req.language_preference {
                sqlx::query("UPDATE user_profiles SET language_preference = $1 WHERE user_id = $2")
                    .bind(lang)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref notifications) = req.email_notifications_enabled {
                sqlx::query("UPDATE user_profiles SET email_notifications_enabled = $1 WHERE user_id = $2")
                    .bind(notifications)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref marketing) = req.marketing_emails_enabled {
                sqlx::query("UPDATE user_profiles SET marketing_emails_enabled = $1 WHERE user_id = $2")
                    .bind(marketing)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
            if let Some(ref visibility) = req.profile_visibility {
                sqlx::query("UPDATE user_profiles SET profile_visibility = $1 WHERE user_id = $2")
                    .bind(visibility)
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
            }
        } else {
            // Create new profile
            sqlx::query(
                r#"
                INSERT INTO user_profiles (
                    user_id, date_of_birth, city, job_title, company_name, industry,
                    years_of_experience, founder_type, startup_experience_level,
                    language_preference, email_notifications_enabled, marketing_emails_enabled, profile_visibility
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#
            )
            .bind(user_id)
            .bind(req.date_of_birth)
            .bind(&req.city)
            .bind(&req.job_title)
            .bind(&req.company_name)
            .bind(&req.industry)
            .bind(req.years_of_experience)
            .bind(&req.founder_type)
            .bind(&req.startup_experience_level)
            .bind(req.language_preference.as_deref().unwrap_or("en"))
            .bind(req.email_notifications_enabled.unwrap_or(true))
            .bind(req.marketing_emails_enabled.unwrap_or(false))
            .bind(req.profile_visibility.as_deref().unwrap_or("private"))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.log_audit(
            Some(user_id),
            AUDIT_EVENT_PROFILE_UPDATED,
            AUDIT_CATEGORY_PROFILE,
            "Profile updated",
            AUDIT_SEVERITY_INFO,
            true,
            ip,
            user_agent,
            None,
        ).await?;

        self.get_profile(user_id).await
    }

    pub async fn update_avatar(
        &self, 
        user_id: Uuid, 
        req: UpdateAvatarRequest,
        ip: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<MessageResponse> {
        sqlx::query(
            r#"
            INSERT INTO user_profiles (user_id, avatar_data, avatar_mime_type, avatar_updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                avatar_data = $2, 
                avatar_mime_type = $3, 
                avatar_updated_at = NOW()
            "#
        )
        .bind(user_id)
        .bind(&req.avatar_data)
        .bind(&req.mime_type)
        .execute(&self.db)
        .await?;

        self.log_audit(
            Some(user_id),
            AUDIT_EVENT_PROFILE_UPDATED,
            AUDIT_CATEGORY_PROFILE,
            "Avatar updated",
            AUDIT_SEVERITY_INFO,
            true,
            ip,
            user_agent,
            None,
        ).await?;

        Ok(MessageResponse {
            message: "Avatar updated successfully".to_string(),
        })
    }

    pub async fn get_avatar(&self, user_id: Uuid) -> Result<(Vec<u8>, String)> {
        let row: (Option<Vec<u8>>, Option<String>) = sqlx::query_as(
            "SELECT avatar_data, avatar_mime_type FROM user_profiles WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Avatar not found".to_string()))?;

        match (row.0, row.1) {
            (Some(data), Some(mime)) => Ok((data, mime)),
            _ => Err(AppError::NotFound("Avatar not found".to_string())),
        }
    }
}

// ============================================================================
// HELPER STRUCTURES
// ============================================================================

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    token_type: String,
}

/// Generate cryptographically secure random token
fn generate_secure_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..48)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
