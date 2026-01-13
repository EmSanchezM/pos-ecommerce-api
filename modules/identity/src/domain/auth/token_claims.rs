// TokenClaims - JWT token claims structure
//
// Contains the claims embedded in JWT access tokens for user authentication.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Claims contained in JWT access token.
///
/// These claims are embedded in the JWT payload and contain
/// essential user information for authentication and authorization.
///
/// # Fields
///
/// * `sub` - Subject (user_id as UUID)
/// * `username` - User's username
/// * `email` - User's email address
/// * `exp` - Expiration timestamp (Unix epoch seconds)
/// * `iat` - Issued at timestamp (Unix epoch seconds)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject - the user ID
    pub sub: Uuid,
    /// User's username
    pub username: String,
    /// User's email address
    pub email: String,
    /// Expiration time (Unix timestamp in seconds)
    pub exp: i64,
    /// Issued at time (Unix timestamp in seconds)
    pub iat: i64,
}

impl TokenClaims {
    /// Creates new token claims.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user's unique identifier
    /// * `username` - The user's username
    /// * `email` - The user's email address
    /// * `exp` - Expiration timestamp (Unix epoch seconds)
    /// * `iat` - Issued at timestamp (Unix epoch seconds)
    pub fn new(
        user_id: Uuid,
        username: String,
        email: String,
        exp: i64,
        iat: i64,
    ) -> Self {
        Self {
            sub: user_id,
            username,
            email,
            exp,
            iat,
        }
    }

    /// Returns the user ID from the claims
    pub fn user_id(&self) -> Uuid {
        self.sub
    }

    /// Checks if the token has expired based on current time
    pub fn is_expired(&self, current_time: i64) -> bool {
        self.exp < current_time
    }

    /// Returns the token's remaining lifetime in seconds
    /// Returns 0 if the token has already expired
    pub fn remaining_lifetime(&self, current_time: i64) -> i64 {
        if self.exp > current_time {
            self.exp - current_time
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_claims() -> TokenClaims {
        TokenClaims::new(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            "john_doe".to_string(),
            "john@example.com".to_string(),
            1705150000, // exp
            1705140000, // iat
        )
    }

    #[test]
    fn test_new_claims() {
        let claims = sample_claims();
        
        assert_eq!(
            claims.sub,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(claims.username, "john_doe");
        assert_eq!(claims.email, "john@example.com");
        assert_eq!(claims.exp, 1705150000);
        assert_eq!(claims.iat, 1705140000);
    }

    #[test]
    fn test_user_id() {
        let claims = sample_claims();
        assert_eq!(
            claims.user_id(),
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_is_expired_false() {
        let claims = sample_claims();
        // Current time before expiration
        assert!(!claims.is_expired(1705140500));
    }

    #[test]
    fn test_is_expired_true() {
        let claims = sample_claims();
        // Current time after expiration
        assert!(claims.is_expired(1705160000));
    }

    #[test]
    fn test_is_expired_at_boundary() {
        let claims = sample_claims();
        // Current time exactly at expiration - token is NOT expired yet
        // (exp is exclusive - token expires AFTER this time)
        assert!(!claims.is_expired(1705150000));
        // One second after expiration - token IS expired
        assert!(claims.is_expired(1705150001));
    }

    #[test]
    fn test_remaining_lifetime_positive() {
        let claims = sample_claims();
        // 5000 seconds before expiration
        assert_eq!(claims.remaining_lifetime(1705145000), 5000);
    }

    #[test]
    fn test_remaining_lifetime_zero_when_expired() {
        let claims = sample_claims();
        // After expiration
        assert_eq!(claims.remaining_lifetime(1705160000), 0);
    }

    #[test]
    fn test_serialize_deserialize() {
        let claims = sample_claims();
        
        // Serialize to JSON
        let json = serde_json::to_string(&claims).unwrap();
        
        // Deserialize back
        let deserialized: TokenClaims = serde_json::from_str(&json).unwrap();
        
        assert_eq!(claims, deserialized);
    }

    #[test]
    fn test_json_structure() {
        let claims = sample_claims();
        let json = serde_json::to_value(&claims).unwrap();
        
        assert_eq!(
            json["sub"],
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(json["username"], "john_doe");
        assert_eq!(json["email"], "john@example.com");
        assert_eq!(json["exp"], 1705150000);
        assert_eq!(json["iat"], 1705140000);
    }

    #[test]
    fn test_claims_equality() {
        let claims1 = sample_claims();
        let claims2 = sample_claims();
        let claims3 = TokenClaims::new(
            Uuid::parse_str("660e8400-e29b-41d4-a716-446655440001").unwrap(),
            "other_user".to_string(),
            "other@example.com".to_string(),
            1705150000,
            1705140000,
        );

        assert_eq!(claims1, claims2);
        assert_ne!(claims1, claims3);
    }

    #[test]
    fn test_clone() {
        let claims = sample_claims();
        let cloned = claims.clone();
        
        assert_eq!(claims, cloned);
    }
}
