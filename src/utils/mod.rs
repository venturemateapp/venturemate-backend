pub mod error;
pub mod jwt;
pub mod password;
pub mod response;

pub use error::{AppError, Result};
pub use jwt::Jwt;
pub use password::{hash_password, verify_password};
pub use response::{ResponseBuilder, success_response, error_response};

// Re-export get_user_id from middleware
pub use crate::middleware::auth::get_user_id;
