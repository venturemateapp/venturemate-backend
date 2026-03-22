use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // User ID
    pub exp: i64,            // Expiration time
    pub iat: i64,            // Issued at
    pub jti: String,         // JWT ID (for token revocation)
    pub token_type: String,  // "access" or "refresh"
    pub session_id: String,  // Session ID for session management
}

pub struct Jwt {
    secret: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
}

impl Jwt {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            access_token_expiry: Duration::hours(1),    // 1 hour per spec
            refresh_token_expiry: Duration::days(30),   // 30 days per spec
        }
    }

    pub fn with_expiry(
        mut self,
        access_hours: i64,
        refresh_days: i64,
    ) -> Self {
        self.access_token_expiry = Duration::hours(access_hours);
        self.refresh_token_expiry = Duration::days(refresh_days);
        self
    }

    /// Generate a new access token
    pub fn generate_access_token(&self, user_id: Uuid) -> Result<(String, i64), AppError> {
        let session_id = Uuid::new_v4();
        self.generate_access_token_with_session(user_id, session_id)
    }

    /// Generate access token with specific session ID
    pub fn generate_access_token_with_session(
        &self, 
        user_id: Uuid, 
        session_id: Uuid
    ) -> Result<(String, i64), AppError> {
        self.generate_token(user_id, session_id, "access", self.access_token_expiry)
    }

    /// Generate a new refresh token
    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<(String, i64), AppError> {
        let session_id = Uuid::new_v4();
        self.generate_refresh_token_with_session(user_id, session_id, 30)
    }

    /// Generate refresh token with specific session ID and expiry days
    pub fn generate_refresh_token_with_session(
        &self, 
        user_id: Uuid, 
        session_id: Uuid,
        expiry_days: i64,
    ) -> Result<(String, i64), AppError> {
        self.generate_token(user_id, session_id, "refresh", Duration::days(expiry_days))
    }

    fn generate_token(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        token_type: &str,
        expiry: Duration,
    ) -> Result<(String, i64), AppError> {
        let now = Utc::now();
        let exp = now + expiry;
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: token_type.to_string(),
            session_id: session_id.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to encode JWT: {}", e)))?;

        Ok((token, exp.timestamp()))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::default();
        
        let token_data: TokenData<Claims> = decode(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            if e.to_string().contains("ExpiredSignature") {
                AppError::Unauthorized("Token has expired".to_string())
            } else {
                AppError::Unauthorized("Invalid token".to_string())
            }
        })?;

        Ok(token_data.claims)
    }

    /// Extract user ID from token
    pub fn extract_user_id(&self, token: &str) -> Result<Uuid, AppError> {
        let claims = self.validate_token(token)?;
        Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))
    }

    /// Extract session ID from token
    pub fn extract_session_id(&self, token: &str) -> Result<Uuid, AppError> {
        let claims = self.validate_token(token)?;
        Uuid::parse_str(&claims.session_id)
            .map_err(|_| AppError::Unauthorized("Invalid session ID in token".to_string()))
    }
    
    /// Alias for validate_token
    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        self.validate_token(token)
    }

    /// Check if token is expired
    pub fn is_expired(&self, token: &str) -> bool {
        self.validate_token(token).is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_validation() {
        let jwt = Jwt::new("test_secret_key_for_testing_only");
        let user_id = Uuid::new_v4();

        // Generate token
        let (token, _) = jwt.generate_access_token(user_id).unwrap();
        assert!(!token.is_empty());

        // Validate token
        let claims = jwt.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "access");
        assert!(!claims.session_id.is_empty());

        // Extract user ID
        let extracted_id = jwt.extract_user_id(&token).unwrap();
        assert_eq!(extracted_id, user_id);
    }

    #[test]
    fn test_session_based_tokens() {
        let jwt = Jwt::new("test_secret_key_for_testing_only");
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        // Generate tokens with specific session
        let (access, _) = jwt.generate_access_token_with_session(user_id, session_id).unwrap();
        let (refresh, _) = jwt.generate_refresh_token_with_session(user_id, session_id, 30).unwrap();

        // Validate both have same session_id
        let access_claims = jwt.validate_token(&access).unwrap();
        let refresh_claims = jwt.validate_token(&refresh).unwrap();

        assert_eq!(access_claims.session_id, session_id.to_string());
        assert_eq!(refresh_claims.session_id, session_id.to_string());
        assert_eq!(access_claims.sub, refresh_claims.sub);
    }

    #[test]
    fn test_invalid_token() {
        let jwt = Jwt::new("test_secret_key_for_testing_only");
        
        let result = jwt.validate_token("invalid.token.here");
        assert!(result.is_err());
    }
}
