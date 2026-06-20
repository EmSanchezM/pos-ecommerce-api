//! BackofficeAuditSubscriber — EventSubscriber that persists audit events to
//! `backoffice_audit_log` via `BackofficeAuditLogRepository`.
//!
//! FR-AUD-3: Registered in both binaries' `event_dispatcher` job.
//! Matches event_type starting with `"backoffice.audit."`.

use std::sync::Arc;

use async_trait::async_trait;
use events::{EventSubscriber, EventsError, OutboxEvent};

use crate::AuditInfraError;
use crate::domain::events::{AUDIT_EVENT_TYPE_PREFIX, BackofficeAuditEvent};
use crate::domain::repositories::{BackofficeAuditLogEntry, BackofficeAuditLogRepository};

/// Subscribes to `backoffice.audit.*` events and writes them to
/// `backoffice_audit_log`.
pub struct BackofficeAuditSubscriber {
    repo: Arc<dyn BackofficeAuditLogRepository>,
}

impl BackofficeAuditSubscriber {
    pub fn new(repo: Arc<dyn BackofficeAuditLogRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl EventSubscriber for BackofficeAuditSubscriber {
    fn name(&self) -> &'static str {
        "backoffice_audit_subscriber"
    }

    fn interested_in(&self, event_type: &str) -> bool {
        event_type.starts_with(AUDIT_EVENT_TYPE_PREFIX)
    }

    async fn handle(&self, event: &OutboxEvent) -> Result<(), EventsError> {
        let audit_event: BackofficeAuditEvent = serde_json::from_value(event.payload().clone())
            .map_err(|e| {
                EventsError::SubscriberFailed(format!(
                    "backoffice_audit_subscriber: failed to deserialize payload: {}",
                    e
                ))
            })?;

        let entry = BackofficeAuditLogEntry::from(audit_event);
        self.repo.append(entry).await.map_err(|e| match e {
            AuditInfraError::Database(db_err) => {
                EventsError::SubscriberFailed(format!("audit log DB write failed: {}", db_err))
            }
            other => EventsError::SubscriberFailed(format!("audit log error: {}", other)),
        })
    }
}

// =============================================================================
// P4-T04: Audit subscriber tests (TEST FIRST — tests were written before impl)
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use events::OutboxEvent;
    use serde_json::json;
    use uuid::{NoContext, Timestamp, Uuid};

    use backoffice_identity::BackofficeUserId;

    use crate::domain::events::{BackofficeAuditEvent, OrgId};

    // -------------------------------------------------------------------------
    // Mock repo
    // -------------------------------------------------------------------------

    struct MockAuditRepo {
        appended: Arc<Mutex<Vec<BackofficeAuditLogEntry>>>,
    }

    impl MockAuditRepo {
        fn new() -> Self {
            Self {
                appended: Arc::new(Mutex::new(vec![])),
            }
        }

        fn rows(&self) -> Vec<BackofficeAuditLogEntry> {
            self.appended.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl BackofficeAuditLogRepository for MockAuditRepo {
        async fn append(&self, entry: BackofficeAuditLogEntry) -> Result<(), AuditInfraError> {
            self.appended.lock().unwrap().push(entry);
            Ok(())
        }

        async fn find_paginated(
            &self,
            _filters: crate::domain::repositories::AuditLogFilters,
            _page: u32,
            _page_size: u32,
        ) -> Result<Vec<BackofficeAuditLogEntry>, AuditInfraError> {
            Ok(self.rows())
        }
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn make_outbox_event(audit_event: &BackofficeAuditEvent) -> OutboxEvent {
        let actor_id_str = audit_event.actor_id.to_string();
        let payload: serde_json::Value = audit_event.clone().into();
        OutboxEvent::create(
            "backoffice",
            &actor_id_str,
            audit_event.event_type(),
            payload,
        )
    }

    fn make_audit_event(action: &str) -> BackofficeAuditEvent {
        BackofficeAuditEvent {
            actor_type: "backoffice_user".to_string(),
            actor_id: BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            action: action.to_string(),
            target_org_id: Some(OrgId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)))),
            reason: "integration test suspend".to_string(),
            ip: "10.0.0.1".to_string(),
        }
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    /// P4-T04a: Subscriber is interested in backoffice.audit.* events.
    #[test]
    fn subscriber_interested_in_audit_events() {
        let repo = Arc::new(MockAuditRepo::new());
        let sub = BackofficeAuditSubscriber::new(repo);

        assert!(sub.interested_in("backoffice.audit.org.suspend"));
        assert!(sub.interested_in("backoffice.audit.user.impersonate"));
        assert!(!sub.interested_in("sale.completed"));
        assert!(!sub.interested_in("backoffice.other.event"));
    }

    /// P4-T04b: Subscriber persists the audit event to the repository.
    #[tokio::test]
    async fn subscriber_persists_event_to_repo() {
        let mock_repo = Arc::new(MockAuditRepo::new());
        let sub = BackofficeAuditSubscriber::new(mock_repo.clone());

        let audit_event = make_audit_event("org.suspend");
        let outbox_event = make_outbox_event(&audit_event);

        sub.handle(&outbox_event).await.unwrap();

        let rows = mock_repo.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, "org.suspend");
        assert_eq!(rows[0].actor_type, "backoffice_user");
        assert_eq!(rows[0].reason, "integration test suspend");
    }

    /// P4-T04c: Subscriber correctly maps actor_id from the event.
    #[tokio::test]
    async fn subscriber_maps_actor_id_correctly() {
        let actor_uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let mock_repo = Arc::new(MockAuditRepo::new());
        let sub = BackofficeAuditSubscriber::new(mock_repo.clone());

        let audit_event = BackofficeAuditEvent {
            actor_type: "backoffice_user".to_string(),
            actor_id: BackofficeUserId::from_uuid(actor_uuid),
            action: "org.suspend".to_string(),
            target_org_id: None,
            reason: "test".to_string(),
            ip: "127.0.0.1".to_string(),
        };
        let outbox_event = make_outbox_event(&audit_event);

        sub.handle(&outbox_event).await.unwrap();

        let rows = mock_repo.rows();
        assert_eq!(rows[0].actor_id, actor_uuid);
        assert!(rows[0].target_org_id.is_none());
    }

    /// P4-T04d: Subscriber returns SubscriberFailed on malformed payload.
    #[tokio::test]
    async fn subscriber_errors_on_malformed_payload() {
        let repo = Arc::new(MockAuditRepo::new());
        let sub = BackofficeAuditSubscriber::new(repo);

        let bad_event = OutboxEvent::create(
            "backoffice",
            "some-id",
            "backoffice.audit.org.suspend",
            json!({"garbage": true}),
        );

        let result = sub.handle(&bad_event).await;
        assert!(
            matches!(result, Err(EventsError::SubscriberFailed(_))),
            "expected SubscriberFailed on malformed payload"
        );
    }
}
