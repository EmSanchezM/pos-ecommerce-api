// TokenService trait - Hexagonal port for token operations
//
// This trait defines the contract for token generation and validation,
// following hexagonal architecture principles. Implementations (adapters)
// will provide the actual JWT logic.

use crate::domain::auth::{AuthError, TokenClaims};
use crate::domain::entities::User;
use crate::domain::value_objects::UserId;

/// Token service port for authentication token operations.
///
/// This trait defines the hexagonal port for token management.
/// Implementations (adapters) handle the actual JWT encoding/decoding.
///
/// # Token Types
///
/// - **Access Token**: Short-lived token (15 min) containing user claims
///   for authenticating API requests.
/// - **Refresh Token**: Long-lived token (7 days) used to obtain new
///   access tokens without re-authentication.
///
pub trait TokenService: Send + Sync {
    /// Generates an access token for the given user.
    ///
    /// The access token contains user claims (user_id, username, email)
    /// and has a short expiration time (typically 15 minutes).
    ///
    /// # Arguments
    ///
    /// * `user` - The authenticated user to generate a token for
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The encoded JWT access token
    /// * `Err(AuthError)` - If token generation fails
    ///
    fn generate_access_token(&self, user: &User) -> Result<String, AuthError>;

    /// Generates a refresh token for the given user.
    ///
    /// The refresh token has a longer expiration time (typically 7 days)
    /// and is used to obtain new access tokens without re-authentication.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to embed in the refresh token
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The encoded refresh token
    /// * `Err(AuthError)` - If token generation fails
    ///
    fn generate_refresh_token(&self, user_id: UserId) -> Result<String, AuthError>;

    /// Validates and decodes an access token.
    ///
    /// Verifies the token signature and expiration, then extracts
    /// the embedded claims.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT access token string to validate
    ///
    /// # Returns
    ///
    /// * `Ok(TokenClaims)` - The decoded token claims if valid
    /// * `Err(AuthError::TokenExpired)` - If the token has expired
    /// * `Err(AuthError::InvalidToken)` - If the token is malformed or has invalid signature
    ///
    fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthError>;

    /// Validates a refresh token and extracts the user ID.
    ///
    /// Verifies the refresh token signature and expiration, then
    /// returns the embedded user ID for generating a new access token.
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token string to validate
    ///
    /// # Returns
    ///
    /// * `Ok(UserId)` - The user ID from the token if valid
    /// * `Err(AuthError::TokenExpired)` - If the token has expired
    /// * `Err(AuthError::InvalidToken)` - If the token is malformed or has invalid signature
    ///
    fn validate_refresh_token(&self, token: &str) -> Result<UserId, AuthError>;
}
