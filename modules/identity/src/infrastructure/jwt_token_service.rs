// JwtTokenService - JWT implementation of TokenService trait
//
// This is the infrastructure adapter that implements the TokenService port
// using the jsonwebtoken crate with HS256 algorithm.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::domain::auth::{AuthError, TokenClaims, TokenService};
use crate::domain::entities::User;
use crate::domain::value_objects::UserId;

/// JWT implementation of the TokenService trait.
///
/// This adapter handles JWT token generation and validation using HS256 algorithm.
/// It follows hexagonal architecture as an infrastructure adapter implementing
/// the domain port (TokenService trait).
///
/// # Configuration
///
/// - `secret`: The secret key used for signing tokens (HS256)
/// - `access_token_duration`: Duration for access tokens (default: 15 minutes)
/// - `refresh_token_duration`: Duration for refresh tokens (default: 7 days)
///
/// # Requirements Coverage
///
/// - Requirement 4.2: Access token expiration of 15 minutes
/// - Requirement 4.3: Refresh token expiration of 7 days
/// - Requirement 4.4: HS256 algorithm with configurable secret
pub struct JwtTokenService {
    secret: String,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
}

/// Claims for refresh tokens (minimal - only user_id and expiration)
#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    /// Subject - the user ID
    sub: uuid::Uuid,
    /// Expiration time (Unix timestamp in seconds)
    exp: i64,
    /// Issued at time (Unix timestamp in seconds)
    iat: i64,
    /// Token type marker to distinguish from access tokens
    token_type: String,
}

impl JwtTokenService {
    /// Creates a new JwtTokenService with the given secret.
    ///
    /// Uses default durations:
    /// - Access token: 15 minutes
    /// - Refresh token: 7 days
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret key for signing tokens (should be at least 32 bytes)
    ///
    /// # Example
    ///
    /// ```
    /// use identity::infrastructure::JwtTokenService;
    ///
    /// let service = JwtTokenService::new("your-secret-key-at-least-32-bytes-long".to_string());
    /// ```
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            access_token_duration: Duration::minutes(15),
            refresh_token_duration: Duration::days(7),
        }
    }

    /// Creates a new JwtTokenService with custom durations.
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret key for signing tokens
    /// * `access_token_duration` - Duration for access tokens
    /// * `refresh_token_duration` - Duration for refresh tokens
    pub fn with_durations(
        secret: String,
        access_token_duration: Duration,
        refresh_token_duration: Duration,
    ) -> Self {
        Self {
            secret,
            access_token_duration,
            refresh_token_duration,
        }
    }

    /// Returns the access token duration in seconds.
    pub fn access_token_duration_secs(&self) -> i64 {
        self.access_token_duration.num_seconds()
    }

    /// Returns the refresh token duration in seconds.
    pub fn refresh_token_duration_secs(&self) -> i64 {
        self.refresh_token_duration.num_seconds()
    }
}

impl TokenService for JwtTokenService {
    /// Generates an access token for the given user.
    ///
    /// The token contains user claims (user_id, username, email) and expires
    /// after the configured access_token_duration (default: 15 minutes).
    ///
    /// # Requirements
    ///
    /// - Requirement 4.1: Token contains user_id, username, and email claims
    /// - Requirement 4.2: Access token expires in 15 minutes
    /// - Requirement 4.4: Uses HS256 algorithm
    fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + self.access_token_duration;

        let claims = TokenClaims::new(
            user.id().as_uuid().to_owned(),
            user.username().as_str().to_string(),
            user.email().as_str().to_string(),
            exp.timestamp(),
            now.timestamp(),
        );

        encode(
            &Header::default(), // HS256 is the default
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::Internal(format!("Failed to generate access token: {}", e)))
    }

    /// Generates a refresh token for the given user ID.
    ///
    /// The refresh token contains only the user_id and expires after the
    /// configured refresh_token_duration (default: 7 days).
    ///
    /// # Requirements
    ///
    /// - Requirement 4.3: Refresh token expires in 7 days
    /// - Requirement 4.4: Uses HS256 algorithm
    fn generate_refresh_token(&self, user_id: UserId) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + self.refresh_token_duration;

        let claims = RefreshTokenClaims {
            sub: user_id.into_uuid(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AuthError::Internal(format!("Failed to generate refresh token: {}", e)))
    }

    /// Validates and decodes an access token.
    ///
    /// Verifies the token signature using HS256 and checks expiration.
    ///
    /// # Requirements
    ///
    /// - Requirement 4.6: Returns TokenExpired for expired tokens
    /// - Requirement 4.6: Returns InvalidToken for malformed/invalid signature
    fn validate_access_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        let validation = Validation::default();

        decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        })
    }

    /// Validates a refresh token and extracts the user ID.
    ///
    /// Verifies the token signature and expiration, then returns the user_id
    /// for generating a new access token.
    ///
    /// # Requirements
    ///
    /// - Requirement 4.5: Valid refresh token allows new access token generation
    /// - Requirement 4.6: Returns TokenExpired for expired tokens
    /// - Requirement 4.6: Returns InvalidToken for malformed/invalid signature
    fn validate_refresh_token(&self, token: &str) -> Result<UserId, AuthError> {
        let validation = Validation::default();

        let token_data = decode::<RefreshTokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        })?;

        // Verify this is actually a refresh token
        if token_data.claims.token_type != "refresh" {
            return Err(AuthError::InvalidToken);
        }

        Ok(UserId::from_uuid(token_data.claims.sub))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{Email, Username};

    fn create_test_user() -> User {
        User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hashed_password".to_string(),
        )
    }

    fn create_service() -> JwtTokenService {
        JwtTokenService::new("test-secret-key-at-least-32-bytes-long".to_string())
    }

    #[test]
    fn test_new_creates_service_with_default_durations() {
        let service = create_service();
        
        // Access token: 15 minutes = 900 seconds
        assert_eq!(service.access_token_duration_secs(), 900);
        
        // Refresh token: 7 days = 604800 seconds
        assert_eq!(service.refresh_token_duration_secs(), 604800);
    }

    #[test]
    fn test_with_durations_creates_custom_service() {
        let service = JwtTokenService::with_durations(
            "secret".to_string(),
            Duration::minutes(30),
            Duration::days(14),
        );
        
        assert_eq!(service.access_token_duration_secs(), 1800); // 30 minutes
        assert_eq!(service.refresh_token_duration_secs(), 1209600); // 14 days
    }

    #[test]
    fn test_generate_access_token_success() {
        let service = create_service();
        let user = create_test_user();
        
        let token = service.generate_access_token(&user);
        
        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(!token.is_empty());
        // JWT tokens have 3 parts separated by dots
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_generate_refresh_token_success() {
        let service = create_service();
        let user_id = UserId::new();
        
        let token = service.generate_refresh_token(user_id);
        
        assert!(token.is_ok());
        let token = token.unwrap();
        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_validate_access_token_success() {
        let service = create_service();
        let user = create_test_user();
        
        let token = service.generate_access_token(&user).unwrap();
        let claims = service.validate_access_token(&token);
        
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, *user.id().as_uuid());
        assert_eq!(claims.username, user.username().as_str());
        assert_eq!(claims.email, user.email().as_str());
    }

    #[test]
    fn test_validate_access_token_invalid_signature() {
        let service = create_service();
        let other_service = JwtTokenService::new("different-secret-key-also-32-bytes".to_string());
        let user = create_test_user();
        
        // Generate token with different secret
        let token = other_service.generate_access_token(&user).unwrap();
        
        // Validate with original service should fail
        let result = service.validate_access_token(&token);
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_validate_access_token_malformed() {
        let service = create_service();
        
        let result = service.validate_access_token("not.a.valid.token");
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_validate_access_token_empty() {
        let service = create_service();
        
        let result = service.validate_access_token("");
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_validate_refresh_token_success() {
        let service = create_service();
        let user_id = UserId::new();
        
        let token = service.generate_refresh_token(user_id).unwrap();
        let result = service.validate_refresh_token(&token);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_uuid(), user_id.as_uuid());
    }

    #[test]
    fn test_validate_refresh_token_invalid_signature() {
        let service = create_service();
        let other_service = JwtTokenService::new("different-secret-key-also-32-bytes".to_string());
        let user_id = UserId::new();
        
        let token = other_service.generate_refresh_token(user_id).unwrap();
        let result = service.validate_refresh_token(&token);
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_validate_refresh_token_rejects_access_token() {
        let service = create_service();
        let user = create_test_user();
        
        // Generate an access token
        let access_token = service.generate_access_token(&user).unwrap();
        
        // Try to validate it as a refresh token - should fail
        // because access tokens don't have the "token_type": "refresh" claim
        let result = service.validate_refresh_token(&access_token);
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_access_token_contains_correct_claims() {
        let service = create_service();
        let user = create_test_user();
        
        let token = service.generate_access_token(&user).unwrap();
        let claims = service.validate_access_token(&token).unwrap();
        
        // Verify all required claims are present
        assert_eq!(claims.sub, *user.id().as_uuid());
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.email, "test@example.com");
        
        // Verify expiration is approximately 15 minutes from now
        let now = Utc::now().timestamp();
        let expected_exp = now + 900; // 15 minutes
        // Allow 5 second tolerance for test execution time
        assert!((claims.exp - expected_exp).abs() < 5);
        
        // Verify iat is approximately now
        assert!((claims.iat - now).abs() < 5);
    }

    #[test]
    fn test_expired_access_token() {
        // Create a token with an expiration in the past by manually crafting claims
        let service = create_service();
        let now = Utc::now();
        let past = now - Duration::hours(1); // 1 hour ago
        
        let claims = TokenClaims::new(
            UserId::new().into_uuid(),
            "testuser".to_string(),
            "test@example.com".to_string(),
            past.timestamp(), // Already expired
            (past - Duration::minutes(15)).timestamp(),
        );
        
        // Manually encode the expired token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test-secret-key-at-least-32-bytes-long".as_bytes()),
        )
        .unwrap();
        
        let result = service.validate_access_token(&token);
        
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_expired_refresh_token() {
        // Create a token with an expiration in the past by manually crafting claims
        let service = create_service();
        let now = Utc::now();
        let past = now - Duration::hours(1); // 1 hour ago
        
        let claims = RefreshTokenClaims {
            sub: UserId::new().into_uuid(),
            exp: past.timestamp(), // Already expired
            iat: (past - Duration::days(7)).timestamp(),
            token_type: "refresh".to_string(),
        };
        
        // Manually encode the expired token
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret("test-secret-key-at-least-32-bytes-long".as_bytes()),
        )
        .unwrap();
        
        let result = service.validate_refresh_token(&token);
        
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_different_users_get_different_tokens() {
        let service = create_service();
        
        let user1 = User::create(
            Username::new("user1").unwrap(),
            Email::new("user1@example.com").unwrap(),
            "User".to_string(),
            "One".to_string(),
            "hash1".to_string(),
        );
        
        let user2 = User::create(
            Username::new("user2").unwrap(),
            Email::new("user2@example.com").unwrap(),
            "User".to_string(),
            "Two".to_string(),
            "hash2".to_string(),
        );
        
        let token1 = service.generate_access_token(&user1).unwrap();
        let token2 = service.generate_access_token(&user2).unwrap();
        
        assert_ne!(token1, token2);
        
        let claims1 = service.validate_access_token(&token1).unwrap();
        let claims2 = service.validate_access_token(&token2).unwrap();
        
        assert_ne!(claims1.sub, claims2.sub);
        assert_eq!(claims1.username, "user1");
        assert_eq!(claims2.username, "user2");
    }
}
