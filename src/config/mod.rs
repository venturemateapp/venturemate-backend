use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_service_role_key: String,
    pub anthropic_api_key: String,
    pub claude_model: String,
    pub claude_max_tokens: usize,
    pub jwt_secret: String,
    pub jwt_expiry_minutes: i64,
    pub stripe_secret_key: Option<String>,
    pub stripe_publishable_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub frontend_url: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: std::env::var("SUPABASE_DB_URL")?,
            supabase_url: std::env::var("NEXT_PUBLIC_SUPABASE_URL")?,
            supabase_anon_key: std::env::var("NEXT_PUBLIC_SUPABASE_ANON_KEY")?,
            supabase_service_role_key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")?,
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")?,
            claude_model: std::env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            claude_max_tokens: std::env::var("CLAUDE_MAX_TOKENS")
                .unwrap_or_else(|_| "4096".to_string())
                .parse()?,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            jwt_expiry_minutes: std::env::var("JWT_EXPIRY_MINUTES")
                .unwrap_or_else(|_| "15".to_string())
                .parse()?,
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY").ok(),
            stripe_publishable_key: std::env::var("STRIPE_PUBLISHABLE_KEY").ok(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").ok(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            frontend_url: std::env::var("FRONTEND_URL").ok(),
        })
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
