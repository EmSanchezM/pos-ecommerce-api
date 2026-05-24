use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// RFC 8693 actor claim — identifies the real actor behind an impersonation token.
///
/// Present only on tokens with `aud: Tenant` issued via the impersonation flow.
/// Phase 2 leaves this `None` on all issued tokens; Phase 5 populates it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorClaim {
    /// The real actor's ID (backoffice_user_id on impersonation tokens).
    pub sub: Uuid,
    /// Identifies the type of the real actor.
    pub sub_type: String,
    /// The real actor's email address (for audit context).
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_actor() -> ActorClaim {
        ActorClaim {
            sub: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            sub_type: "backoffice_user".to_string(),
            email: "operator@platform.com".to_string(),
        }
    }

    #[test]
    fn actor_claim_has_sub_sub_type_email_fields() {
        let actor = sample_actor();
        assert_eq!(
            actor.sub,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(actor.sub_type, "backoffice_user");
        assert_eq!(actor.email, "operator@platform.com");
    }

    #[test]
    fn actor_claim_round_trips_through_serde() {
        let original = sample_actor();
        let json = serde_json::to_string(&original).unwrap();
        let decoded: ActorClaim = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn actor_claim_json_keys_are_snake_case() {
        let actor = sample_actor();
        let value = serde_json::to_value(&actor).unwrap();
        assert!(value.get("sub").is_some());
        assert!(value.get("sub_type").is_some());
        assert!(value.get("email").is_some());
    }
}
