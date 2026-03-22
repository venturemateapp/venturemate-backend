use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

use crate::utils::AppError;

/// Hash a password using Argon2
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
    
    Ok(password_hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
    
    let result = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash);
    
    match result {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password123";
        let hash = hash_password(password).unwrap();
        assert_ne!(hash, password);
        
        // Verify correct password
        assert!(verify_password(password, &hash).unwrap());
        
        // Verify wrong password fails
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
