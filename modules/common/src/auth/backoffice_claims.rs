use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TokenAudience;

/// JWT claims embedded in backoffice tokens (aud: Backoffice).
///
/// Distinct from tenant `TokenClaims` — backoffice tokens carry platform
/// permissions instead of store-scoped permissions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackofficeClaims {
    /// Subject — the backoffice_user_id.
    pub sub: Uuid,
    /// Audience — always `Backoffice` for these tokens.
    pub aud: TokenAudience,
    /// Issuer string.
    pub iss: String,
    /// Expiration time (Unix timestamp in seconds).
    pub exp: i64,
    /// Issued-at time (Unix timestamp in seconds).
    pub iat: i64,
    /// Flat list of platform permission codes granted to this user.
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_claims() -> BackofficeClaims {
        BackofficeClaims {
            sub: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            aud: TokenAudience::Backoffice,
            iss: "backoffice-api:test".to_string(),
            exp: 1705150000,
            iat: 1705140000,
            permissions: vec!["platform:org.list".to_string()],
        }
    }

    #[test]
    fn backoffice_claims_round_trips_through_serde() {
        let original = sample_claims();
        let json = serde_json::to_string(&original).unwrap();
        let decoded: BackofficeClaims = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn backoffice_claims_aud_is_backoffice() {
        let claims = sample_claims();
        assert_eq!(claims.aud, TokenAudience::Backoffice);
    }
}
