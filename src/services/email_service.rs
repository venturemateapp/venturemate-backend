use tracing::{error, info, warn};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;
use lettre::{
    message::{header::ContentType, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

/// Email Payload
#[derive(Debug, Clone)]
pub struct EmailPayload {
    pub to: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub text_content: Option<String>,
    pub html_content: Option<String>,
    pub template_data: Option<EmailTemplateData>,
}

/// Email Template Data for dynamic content
#[derive(Debug, Clone)]
pub struct EmailTemplateData {
    pub template_name: String, // "welcome", "password_reset", "login_alert", etc.
    pub user_name: String,
    pub business_name: Option<String>,
    pub action_url: Option<String>,
    pub action_text: Option<String>,
    pub otp_code: Option<String>,
    pub expires_at: Option<String>,
    pub extra_data: Option<serde_json::Value>,
}

/// Email Log Entry from database
#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct EmailLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub business_id: Option<Uuid>,
    pub email_type: String,
    pub recipient: String,
    pub subject: String,
    pub status: String, // "sent", "failed", "pending"
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Email Service
pub struct EmailService {
    db: PgPool,
    smtp_transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
    from_name: String,
}

impl EmailService {
    /// Create new email service
    pub async fn new(db: PgPool) -> Self {
        let from_email = env::var("SMTP_FROM_EMAIL")
            .unwrap_or_else(|_| "noreply@venturemate.app".to_string());
        let from_name = env::var("SMTP_FROM_NAME")
            .unwrap_or_else(|_| "VentureMate".to_string());

        // Initialize SMTP transport
        let smtp_transport = match Self::init_smtp_transport().await {
            Ok(transport) => {
                info!("SMTP transport initialized successfully");
                Some(transport)
            }
            Err(e) => {
                warn!("SMTP not configured - emails will be logged but not sent. Error: {}", e);
                None
            }
        };

        Self {
            db,
            smtp_transport,
            from_email,
            from_name,
        }
    }

    /// Initialize SMTP transport
    async fn init_smtp_transport() -> Result<AsyncSmtpTransport<Tokio1Executor>, Box<dyn std::error::Error>> {
        let smtp_host = env::var("SMTP_HOST")
            .unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(465);
        let smtp_user = env::var("SMTP_USER")
            .map_err(|_| "SMTP_USER not set")?;
        let smtp_pass = env::var("SMTP_PASS")
            .map_err(|_| "SMTP_PASS not set")?;

        let creds = Credentials::new(smtp_user, smtp_pass);

        // Build transport with TLS wrapper (SMTPS on port 465)
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
            .port(smtp_port)
            .credentials(creds)
            .build();

        // Test connection
        transport.test_connection().await?;
        
        info!("SMTP transport initialized: {}:{}", smtp_host, smtp_port);
        Ok(transport)
    }

    /// Send email with template
    pub async fn send_template_email(
        &self,
        payload: EmailPayload,
        user_id: Option<Uuid>,
        business_id: Option<Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let email_type = payload.template_data.as_ref()
            .map(|d| d.template_name.clone())
            .unwrap_or_else(|| "generic".to_string());

        // Generate HTML body from template
        let html_body = self.generate_email_html(&payload);
        let subject = payload.subject.clone();
        let recipient = payload.to.clone();

        // Log attempt
        let log_id = self.log_email_attempt(
            user_id,
            business_id,
            &email_type,
            &recipient,
            &subject,
        ).await?;

        // Send email if SMTP is configured
        match &self.smtp_transport {
            Some(transport) => {
                let email = Message::builder()
                    .from(format!("{} <{}>", self.from_name, self.from_email).parse()?)
                    .to(format!("{} <{}>", payload.to_name.as_ref().unwrap_or(&payload.to), payload.to).parse()?)
                    .subject(&subject)
                    .header(ContentType::TEXT_HTML)
                    .body(html_body)?;

                match transport.send(email).await {
                    Ok(_) => {
                        info!("Email sent: {} to {}", email_type, recipient);
                        self.update_email_log(log_id, "sent", None).await?;
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to send email: {}", e);
                        self.update_email_log(log_id, "failed", Some(&e.to_string())).await?;
                        Err(Box::new(std::io::Error::other(format!("SMTP error: {}", e))))
                    }
                }
            }
            None => {
                // Development mode - log but don't send
                warn!("[DEV MODE] Email not sent - logged only: {}", email_type);
                info!("[DEV MODE] To: {} | Subject: {}", recipient, subject);
                self.update_email_log(log_id, "logged", Some("SMTP not configured - dev mode")).await?;
                Ok(())
            }
        }
    }

    /// Send password reset email
    pub async fn send_password_reset(
        &self,
        email: &str,
        user_id: Uuid,
        reset_token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let reset_url = format!("{}/auth/reset-password?token={}", frontend_url, reset_token);
        
        let payload = EmailPayload {
            to: email.to_string(),
            to_name: None,
            subject: "Reset Your VentureMate Password".to_string(),
            text_content: None,
            html_content: None,
            template_data: Some(EmailTemplateData {
                template_name: "password_reset".to_string(),
                user_name: "there".to_string(),
                business_name: None,
                action_url: Some(reset_url),
                action_text: Some("Reset Password".to_string()),
                otp_code: None,
                expires_at: Some(expires_at.format("%B %d, %Y at %H:%M UTC").to_string()),
                extra_data: None,
            }),
        };

        self.send_template_email(payload, Some(user_id), None).await
    }

    /// Send welcome email
    pub async fn send_welcome(
        &self,
        email: &str,
        user_id: Uuid,
        first_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let payload = EmailPayload {
            to: email.to_string(),
            to_name: Some(first_name.to_string()),
            subject: "Welcome to VentureMate!".to_string(),
            text_content: None,
            html_content: None,
            template_data: Some(EmailTemplateData {
                template_name: "welcome".to_string(),
                user_name: first_name.to_string(),
                business_name: None,
                action_url: Some(format!("{}/dashboard", frontend_url)),
                action_text: Some("Get Started".to_string()),
                otp_code: None,
                expires_at: None,
                extra_data: None,
            }),
        };

        self.send_template_email(payload, Some(user_id), None).await
    }

    /// Send login alert
    pub async fn send_login_alert(
        &self,
        email: &str,
        user_id: Uuid,
        ip_address: &str,
        user_agent: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let extra_data = serde_json::json!({
            "ip_address": ip_address,
            "user_agent": user_agent,
            "timestamp": timestamp.to_rfc3339(),
        });
        
        let payload = EmailPayload {
            to: email.to_string(),
            to_name: None,
            subject: "New Login to Your VentureMate Account".to_string(),
            text_content: None,
            html_content: None,
            template_data: Some(EmailTemplateData {
                template_name: "login_alert".to_string(),
                user_name: "there".to_string(),
                business_name: None,
                action_url: Some(format!("{}/dashboard/settings", frontend_url)),
                action_text: Some("Review Account Security".to_string()),
                otp_code: None,
                expires_at: None,
                extra_data: Some(extra_data),
            }),
        };

        self.send_template_email(payload, Some(user_id), None).await
    }

    /// Send email verification
    pub async fn send_email_verification(
        &self,
        email: &str,
        user_id: Uuid,
        verification_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let verify_url = format!("{}/auth/verify-email?token={}", frontend_url, verification_token);
        
        let payload = EmailPayload {
            to: email.to_string(),
            to_name: None,
            subject: "Verify Your VentureMate Email".to_string(),
            text_content: None,
            html_content: None,
            template_data: Some(EmailTemplateData {
                template_name: "email_verification".to_string(),
                user_name: "there".to_string(),
                business_name: None,
                action_url: Some(verify_url),
                action_text: Some("Verify Email".to_string()),
                otp_code: None,
                expires_at: None,
                extra_data: None,
            }),
        };

        self.send_template_email(payload, Some(user_id), None).await
    }

    /// Send password changed confirmation
    pub async fn send_password_changed_confirmation(
        &self,
        email: &str,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = EmailPayload {
            to: email.to_string(),
            to_name: None,
            subject: "Your VentureMate Password Has Been Changed".to_string(),
            text_content: None,
            html_content: None,
            template_data: Some(EmailTemplateData {
                template_name: "password_changed".to_string(),
                user_name: "there".to_string(),
                business_name: None,
                action_url: None,
                action_text: None,
                otp_code: None,
                expires_at: None,
                extra_data: None,
            }),
        };

        self.send_template_email(payload, Some(user_id), None).await
    }

    /// Send email verified confirmation
    pub async fn send_email_verified_confirmation(
        &self,
        email: &str,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let payload = EmailPayload {
            to: email.to_string(),
            to_name: None,
            subject: "Your VentureMate Email is Verified".to_string(),
            text_content: None,
            html_content: None,
            template_data: Some(EmailTemplateData {
                template_name: "email_verified".to_string(),
                user_name: "there".to_string(),
                business_name: None,
                action_url: Some(format!("{}/dashboard", frontend_url)),
                action_text: Some("Go to Dashboard".to_string()),
                otp_code: None,
                expires_at: None,
                extra_data: None,
            }),
        };

        self.send_template_email(payload, Some(user_id), None).await
    }

    /// Generate HTML email body
    fn generate_email_html(&self, payload: &EmailPayload) -> String {
        // If HTML content is provided directly, use it
        if let Some(html) = &payload.html_content {
            if html.contains("<html") {
                return html.clone();
            }
        }

        // Generate from template
        let template_data = payload.template_data.as_ref();
        let template_name = template_data.map(|d| d.template_name.as_str()).unwrap_or("generic");
        let user_name = template_data.as_ref().map(|d| d.user_name.as_str()).unwrap_or("there");
        let action_url = template_data.as_ref().and_then(|d| d.action_url.as_deref());
        let action_text = template_data.as_ref().and_then(|d| d.action_text.as_deref());
        let expires_at = template_data.as_ref().and_then(|d| d.expires_at.as_deref());

        let content = match template_name {
            "password_reset" => format!(
                r#"
                <p>We received a request to reset your password for your VentureMate account.</p>
                <p>Click the button below to reset your password. This link will expire on <strong>{}</strong>.</p>
                <p>If you didn't request this, please ignore this email or contact support if you have concerns.</p>
                "#,
                expires_at.unwrap_or("24 hours")
            ),
            "welcome" => r#"
                <p>Welcome to VentureMate! We're excited to help you build and scale your startup.</p>
                <p>With VentureMate, you can:</p>
                <ul>
                    <li>Generate AI-powered business plans and pitch decks</li>
                    <li>Track compliance and regulatory requirements</li>
                    <li>Manage your business banking and taxes</li>
                    <li>Build professional websites</li>
                </ul>
                <p>Click the button below to start your journey!</p>
            "#.to_string(),
            "password_changed" => r#"
                <p>This is a confirmation that your VentureMate account password has been successfully changed.</p>
                <p>If you did not make this change, please contact our support team immediately to secure your account.</p>
                <p>For security reasons, all your active sessions have been terminated. You will need to log in again with your new password.</p>
            "#.to_string(),
            "email_verified" => r#"
                <p>Congratulations! Your email address has been successfully verified.</p>
                <p>You now have full access to all VentureMate features, including:</p>
                <ul>
                    <li>Creating and managing businesses</li>
                    <li>AI-powered document generation</li>
                    <li>Banking and tax integrations</li>
                    <li>Team collaboration features</li>
                </ul>
                <p>Click the button below to go to your dashboard and start building!</p>
            "#.to_string(),
            "login_alert" => {
                let extra = template_data.as_ref().and_then(|d| d.extra_data.as_ref());
                let ip = extra.and_then(|e| e.get("ip_address")).and_then(|v| v.as_str()).unwrap_or("Unknown");
                let time = extra.and_then(|e| e.get("timestamp")).and_then(|v| v.as_str()).unwrap_or("Unknown");
                format!(
                    r#"
                    <p>We noticed a new login to your VentureMate account.</p>
                    <div style="background: rgba(255,255,255,0.05); padding: 16px; border-radius: 8px; margin: 20px 0;">
                        <p style="margin: 0;"><strong>IP Address:</strong> {}</p>
                        <p style="margin: 8px 0 0 0;"><strong>Time:</strong> {}</p>
                    </div>
                    <p>If this was you, you can ignore this email. If you don't recognize this activity, please secure your account immediately.</p>
                    "#,
                    ip, time
                )
            }
            "email_verification" => r#"
                <p>Please verify your email address to complete your VentureMate registration.</p>
                <p>Clicking the button below will confirm your email and activate your account.</p>
            "#.to_string(),
            _ => payload.text_content.clone().unwrap_or_default(),
        };

        self.wrap_in_template(&payload.subject, user_name, &content, action_url, action_text)
    }

    /// Wrap content in email template
    fn wrap_in_template(
        &self,
        subject: &str,
        user_name: &str,
        content: &str,
        action_url: Option<&str>,
        action_text: Option<&str>,
    ) -> String {
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());
        
        let action_button = if let (Some(url), Some(text)) = (action_url, action_text) {
            format!(
                r#"<div class="cta-section">
                    <a href="{}" class="cta-button">{}</a>
                </div>"#,
                url, text
            )
        } else {
            String::new()
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
  <style>
    * {{
      margin: 0;
      padding: 0;
      color: #ffffff;
      line-height: 1.6;
    }}
    body {{
      background-color: #0a0a0a;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    }}
    .email-wrapper {{
      max-width: 600px;
      margin: 20px auto;
      background: linear-gradient(135deg, #111 0%, #1e1e1e 50%, #2a2a2a 100%);
      border-radius: 16px;
      overflow: hidden;
      box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
      border: 1px solid #333;
    }}
    .header {{
      background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
      padding: 40px 30px;
      text-align: center;
      border-bottom: 2px solid #4CAF50;
    }}
    .logo {{
      font-size: 32px;
      font-weight: 700;
      background: linear-gradient(135deg, #4CAF50 0%, #2196F3 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      text-decoration: none;
    }}
    .content {{
      padding: 40px 30px;
      background: #1a1a1a;
    }}
    .content h1 {{
      color: #4CAF50;
      font-size: 24px;
      font-weight: 700;
      margin-bottom: 24px;
      text-align: center;
    }}
    .greeting {{
      font-size: 18px;
      margin-bottom: 20px;
      color: #ffffff;
    }}
    .message-text {{
      background: rgba(255,255,255,0.05);
      padding: 24px;
      border-radius: 12px;
      border-left: 4px solid #4CAF50;
      margin: 24px 0;
    }}
    .message-text ul {{
      margin: 16px 0 0 20px;
    }}
    .message-text li {{
      margin: 8px 0;
    }}
    .cta-section {{
      text-align: center;
      margin: 35px 0;
    }}
    .cta-button {{
      display: inline-block;
      background: linear-gradient(135deg, #4CAF50 0%, #2196F3 100%);
      color: #ffffff !important;
      padding: 16px 32px;
      border-radius: 8px;
      font-weight: 600;
      text-decoration: none;
      box-shadow: 0 4px 15px rgba(76, 175, 80, 0.4);
    }}
    .footer {{
      background: linear-gradient(135deg, #0a0a0a 0%, #1a1a1a 100%);
      padding: 30px;
      text-align: center;
      color: rgba(255,255,255,0.6);
      font-size: 14px;
    }}
    .footer a {{
      color: #4CAF50;
      text-decoration: none;
    }}
    .divider {{
      height: 1px;
      background: rgba(255,255,255,0.1);
      margin: 20px 0;
    }}
    @media (max-width: 600px) {{
      .content {{
        padding: 30px 20px;
      }}
      .header {{
        padding: 30px 20px;
      }}
    }}
  </style>
</head>
<body>
  <div class="email-wrapper">
    <div class="header">
      <div class="logo">VentureMate</div>
    </div>

    <div class="content">
      <h1>{}</h1>
      <div class="greeting">Hi {},</div>
      <div class="message-text">{}</div>
      {}
      <div class="divider"></div>
      <p style="color: rgba(255,255,255,0.6); font-size: 14px;">
        If you have any questions, reply to this email or contact us at <a href="mailto:support@venturemate.app">support@venturemate.app</a>
      </p>
    </div>

    <div class="footer">
      <p><strong>&copy; 2025 VentureMate - All Rights Reserved</strong></p>
      <p style="margin-top: 10px;">
        <a href="{}">Visit Website</a> | 
        <a href="{}/dashboard/settings">Settings</a> | 
        <a href="{}/privacy">Privacy Policy</a>
      </p>
      <p style="margin-top: 10px; font-size: 12px; color: rgba(255,255,255,0.4);">
        You're receiving this email because you have an account on VentureMate.
      </p>
    </div>
  </div>
</body>
</html>"#,
            subject, subject, user_name, content, action_button, frontend_url, frontend_url, frontend_url
        )
    }

    /// Log email attempt
    async fn log_email_attempt(
        &self,
        user_id: Option<Uuid>,
        business_id: Option<Uuid>,
        email_type: &str,
        recipient: &str,
        subject: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO email_logs (
                id, user_id, business_id, email_type, recipient, 
                subject, status, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW())
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(business_id)
        .bind(email_type)
        .bind(recipient)
        .bind(subject)
        .execute(&self.db)
        .await?;

        Ok(id)
    }

    /// Update email log
    async fn update_email_log(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"
            UPDATE email_logs 
            SET status = $1, error_message = $2, updated_at = NOW()
            WHERE id = $3
            "#
        )
        .bind(status)
        .bind(error_message)
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get email history for user
    pub async fn get_user_email_history(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EmailLogEntry>, Box<dyn std::error::Error>> {
        let logs = sqlx::query_as::<_, EmailLogEntry>(
            r#"
            SELECT * FROM email_logs 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(logs)
    }
}
