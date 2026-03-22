// Simple utility to create a test user directly in the database
// This bypasses rate limiting for testing purposes

use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("SUPABASE_DB_URL")?;
    
    // Initialize database
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    // Test user details - VERIFIED user for frontend testing
    let email = "test.frontend@venturemate.app";
    let password = "Test1234!";
    let first_name = "Frontend";
    let last_name = "Tester";
    
    // Check if user already exists
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL"
    )
    .bind(email)
    .fetch_optional(&db_pool)
    .await?;
    
    // Hash password using argon2 (same as auth_service)
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use rand::rngs::OsRng;
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hash error: {:?}", e))?
        .to_string();
    
    if let Some((user_id,)) = existing {
        // Update password and ensure email is verified
        sqlx::query(
            "UPDATE users SET password_hash = $1, email_verified_at = NOW() WHERE id = $2"
        )
        .bind(&password_hash)
        .bind(user_id)
        .execute(&db_pool)
        .await?;
        
        println!("✅ Test user updated with new password and verified email!");
        println!("User ID: {}", user_id);
        println!("Email: {}", email);
        println!("Password: {}", password);
        return Ok(());
    }
    
    // Create user
    let user_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, country_code, status, email_verified_at, onboarding_completed)
        VALUES ($1, $2, $3, $4, 'US', 'active', NOW(), false)
        RETURNING id
        "#
    )
    .bind(email)
    .bind(&password_hash)
    .bind(first_name)
    .bind(last_name)
    .fetch_one(&db_pool)
    .await?;
    
    // Create user profile
    sqlx::query(
        r#"
        INSERT INTO user_profiles (user_id, language_preference, profile_visibility)
        VALUES ($1, 'en', 'private')
        "#
    )
    .bind(user_id)
    .execute(&db_pool)
    .await?;
    
    println!("✅ Test user created successfully!");
    println!("User ID: {}", user_id);
    println!("Email: {}", email);
    println!("Password: {}", password);
    
    Ok(())
}
