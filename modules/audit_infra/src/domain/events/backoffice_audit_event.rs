//! BackofficeAuditEvent — domain event payload for the audit outbox.
//!
//! FR-AUD-2: Every state-mutating backoffice action emits this event via
//! `modules/events::PublishEventUseCase` inside the same `sqlx::Transaction`
//! that applies the state change.
//!
//! The `event_type` for all audit events follows the `"backoffice.audit.*"`
//! naming convention so the `BackofficeAuditSubscriber` can match them with
//! `interested_in`.

use backoffice_identity::BackofficeUserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable event type prefix matched by `BackofficeAuditSubscriber`.
pub const AUDIT_EVENT_TYPE_PREFIX: &str = "backoffice.audit.";

/// Payload for every backoffice audit domain event.
///
/// Fields follow FR-AUD-2 exactly:
/// - `actor_type`: always `"backoffice_user"` in Phase 4; extensible later.
/// - `actor_id`: the operator who performed the action (newtype ID, not raw Uuid).
/// - `action`: what was done, e.g. `"org.suspend"`, `"user.impersonate"`.
/// - `target_org_id`: the organisation affected, when applicable.
/// - `reason`: REQUIRED — callers must validate this is non-empty before
///   constructing the event (or let `emit_audit_event` reject an empty string).
/// - `ip`: the client IP extracted from the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackofficeAuditEvent {
    pub actor_type: String,
    pub actor_id: BackofficeUserId,
    pub action: String,
    pub target_org_id: Option<OrgId>,
    pub reason: String,
    pub ip: String,
}

/// Thin newtype so `audit_infra` does NOT take a hard dep on `tenancy`.
/// The caller (backoffice-api handler) extracts the org UUID from the path
/// parameter and wraps it here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrgId(Uuid);

impl OrgId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for OrgId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl BackofficeAuditEvent {
    /// Returns the full `event_type` string for this action, e.g.
    /// `"backoffice.audit.org.suspend"`.
    pub fn event_type(&self) -> String {
        format!("{}{}", AUDIT_EVENT_TYPE_PREFIX, self.action)
    }
}

impl From<BackofficeAuditEvent> for serde_json::Value {
    fn from(event: BackofficeAuditEvent) -> Self {
        serde_json::to_value(&event).expect("BackofficeAuditEvent is always serializable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    fn make_event(action: &str) -> BackofficeAuditEvent {
        BackofficeAuditEvent {
            actor_type: "backoffice_user".to_string(),
            actor_id: BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            action: action.to_string(),
            target_org_id: Some(OrgId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)))),
            reason: "routine maintenance".to_string(),
            ip: "192.168.1.1".to_string(),
        }
    }

    #[test]
    fn event_type_has_backoffice_audit_prefix() {
        let evt = make_event("org.suspend");
        assert_eq!(evt.event_type(), "backoffice.audit.org.suspend");
    }

    #[test]
    fn serde_roundtrip_preserves_all_fields() {
        let original = make_event("org.suspend");
        let json: serde_json::Value = original.clone().into();
        let restored: BackofficeAuditEvent = serde_json::from_value(json).unwrap();

        assert_eq!(restored.actor_type, original.actor_type);
        assert_eq!(restored.actor_id, original.actor_id);
        assert_eq!(restored.action, original.action);
        assert_eq!(
            restored.target_org_id.map(|id| id.into_uuid()),
            original.target_org_id.map(|id| id.into_uuid())
        );
        assert_eq!(restored.reason, original.reason);
        assert_eq!(restored.ip, original.ip);
    }

    #[test]
    fn event_without_target_org_serializes_cleanly() {
        let mut evt = make_event("user.impersonate");
        evt.target_org_id = None;

        let json: serde_json::Value = evt.into();
        assert!(json.get("target_org_id").unwrap().is_null());
    }

    #[test]
    fn org_id_roundtrip() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let org_id = OrgId::from_uuid(uuid);
        assert_eq!(org_id.into_uuid(), uuid);
    }
}
