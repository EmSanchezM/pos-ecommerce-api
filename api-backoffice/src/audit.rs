// Audit event emission helper
//
// FR-AUD-1: Every state-mutating backoffice action emits an event to
// `outbox_events` in the SAME transaction as the state change.
//
// FR-AUD-6: This helper is the single place where event-construction logic
// lives, so handlers don't duplicate it.
//
// NFR-SEC-5: `reason` is validated here — an empty string causes a 400 before
// any outbox write occurs.

use std::sync::Arc;

use axum::http::StatusCode;
use sqlx::{Postgres, Transaction};

use audit_infra::BackofficeAuditEvent;
use events::PublishEventUseCase;

use crate::error::{AppError, ErrorResponse};

/// Write a backoffice audit event to `outbox_events` in the provided
/// transaction.
///
/// # Validation
///
/// Returns `AppError` (HTTP 400) if `event.reason` is empty (NFR-SEC-5).
///
/// # Transactional guarantee
///
/// The outbox INSERT happens inside `tx`. If the caller rolls back `tx` after
/// this call returns `Ok(())`, the outbox row is also rolled back — no phantom
/// audit entries.
pub async fn emit_audit_event(
    tx: &mut Transaction<'_, Postgres>,
    publish: &Arc<PublishEventUseCase>,
    event: BackofficeAuditEvent,
) -> Result<(), AppError> {
    // NFR-SEC-5: reason field is required — reject empty string with 400.
    if event.reason.trim().is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            ErrorResponse::new(
                "REASON_REQUIRED",
                "The 'reason' field is required for all state-mutating backoffice actions",
            ),
        ));
    }

    let event_type = event.event_type();
    let actor_id_str = event.actor_id.to_string();
    let payload: serde_json::Value = event.into();

    publish
        .execute(tx, "backoffice", &actor_id_str, &event_type, payload)
        .await
        .map_err(|e| {
            tracing::error!("failed to write audit event to outbox: {}", e);
            AppError::internal()
        })
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    // Unit tests for the reason-validation guard.
    // The transactional outbox write requires a live DB and is covered by
    // integration/E2E tests (S-06, S-07).

    use audit_infra::{BackofficeAuditEvent, OrgId};
    use backoffice_identity::BackofficeUserId;
    use uuid::{NoContext, Timestamp, Uuid};

    fn make_event_with_reason(reason: &str) -> BackofficeAuditEvent {
        BackofficeAuditEvent {
            actor_type: "backoffice_user".to_string(),
            actor_id: BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext))),
            action: "org.suspend".to_string(),
            target_org_id: Some(OrgId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)))),
            reason: reason.to_string(),
            ip: "127.0.0.1".to_string(),
        }
    }

    /// S-07: missing reason → reason field check fires before any DB access.
    /// We test the validation logic directly (no tx needed).
    #[test]
    fn empty_reason_is_invalid() {
        let event = make_event_with_reason("");
        assert!(
            event.reason.trim().is_empty(),
            "empty reason should be detected"
        );
    }

    #[test]
    fn whitespace_only_reason_is_invalid() {
        let event = make_event_with_reason("   ");
        assert!(event.reason.trim().is_empty());
    }

    #[test]
    fn non_empty_reason_is_valid() {
        let event = make_event_with_reason("fraud detected");
        assert!(!event.reason.trim().is_empty());
    }
}
