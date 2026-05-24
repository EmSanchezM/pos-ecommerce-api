use serde::{Deserialize, Serialize};

/// Identifies which service a JWT was issued for.
///
/// Required on every token. `api-gateway` rejects anything that is not
/// `Tenant`. `backoffice-api` rejects anything that is not `Backoffice`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenAudience {
    /// Token issued by api-gateway for tenant users.
    Tenant,
    /// Token issued by backoffice-api for platform operators.
    Backoffice,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_serializes_to_snake_case_string() {
        let json = serde_json::to_string(&TokenAudience::Tenant).unwrap();
        assert_eq!(json, "\"tenant\"");
    }

    #[test]
    fn backoffice_serializes_to_snake_case_string() {
        let json = serde_json::to_string(&TokenAudience::Backoffice).unwrap();
        assert_eq!(json, "\"backoffice\"");
    }

    #[test]
    fn tenant_deserializes_from_snake_case_string() {
        let aud: TokenAudience = serde_json::from_str("\"tenant\"").unwrap();
        assert_eq!(aud, TokenAudience::Tenant);
    }

    #[test]
    fn backoffice_deserializes_from_snake_case_string() {
        let aud: TokenAudience = serde_json::from_str("\"backoffice\"").unwrap();
        assert_eq!(aud, TokenAudience::Backoffice);
    }

    #[test]
    fn round_trip_tenant() {
        let original = TokenAudience::Tenant;
        let json = serde_json::to_string(&original).unwrap();
        let decoded: TokenAudience = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn round_trip_backoffice() {
        let original = TokenAudience::Backoffice;
        let json = serde_json::to_string(&original).unwrap();
        let decoded: TokenAudience = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }
}
