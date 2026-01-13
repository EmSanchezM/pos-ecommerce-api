// Authentication domain module
//
// Contains value objects, error types, and traits for authentication operations.

mod error;
mod login_identifier;
mod token_claims;
mod token_service;

pub use error::AuthError;
pub use login_identifier::LoginIdentifier;
pub use token_claims::TokenClaims;
pub use token_service::TokenService;
