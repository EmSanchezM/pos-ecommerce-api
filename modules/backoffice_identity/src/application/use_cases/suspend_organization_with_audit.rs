// SuspendOrganizationWithAuditUseCase
//
// Phase 4 use case that suspends an organization in a single atomic transaction
// that also writes the audit event to the outbox.
//
// # Clean Architecture placement
//
// This use case lives in `backoffice_identity` (application layer) rather than
// `tenancy` because it orchestrates a cross-cutting concern: the org state
// change (tenancy domain) AND the audit pipeline. Neither tenancy nor events
// should know about each other.
//
// # Circular dependency note
//
// `audit_infra` depends on `backoffice_identity` (for BackofficeUserId), so
// `backoffice_identity` cannot depend on `audit_infra` in return. Instead we
// construct the outbox payload as a `serde_json::Value` inline using the same
// field layout that `BackofficeAuditEvent` serialises to. This keeps the same
// wire format without the cycle.
//
// # Transactional guarantee (C-7)
//
// Steps executed inside ONE `sqlx::Transaction`:
//   1. find_by_id_in_tx  — read for update inside tx
//   2. org.suspend()     — mutate in memory (domain method enforces state machine)
//   3. update_in_tx      — persist the updated status
//   4. emit outbox event — write to outbox_events in the SAME tx
//   5. tx.commit()       — both DB rows land atomically

use std::sync::Arc;

use events::PublishEventUseCase;
use sqlx::PgPool;
use tenancy::{Organization, OrganizationId, OrganizationRepository};
use uuid::Uuid;

use crate::domain::value_objects::BackofficeUserId;
use crate::error::BackofficeIdentityError;

/// Prefix used by BackofficeAuditSubscriber to match events.
/// Must match `audit_infra::AUDIT_EVENT_TYPE_PREFIX`.
const AUDIT_EVENT_TYPE_PREFIX: &str = "backoffice.audit.";

pub struct SuspendOrganizationWithAuditUseCase {
    pool: PgPool,
    org_repo: Arc<dyn OrganizationRepository>,
    publish_event: Arc<PublishEventUseCase>,
}

impl SuspendOrganizationWithAuditUseCase {
    pub fn new(
        pool: PgPool,
        org_repo: Arc<dyn OrganizationRepository>,
        publish_event: Arc<PublishEventUseCase>,
    ) -> Self {
        Self {
            pool,
            org_repo,
            publish_event,
        }
    }

    /// Suspend an organization and emit an audit event, both in the same transaction.
    ///
    /// # Errors
    ///
    /// - `InvalidInput` — reason is empty or blank.
    /// - `OrgNotFound`  — org with the given id does not exist.
    /// - `Tenancy`      — org state machine rejects the transition (e.g. already suspended).
    /// - `Outbox`       — outbox write failed.
    /// - `Database`     — any other SQLx error.
    pub async fn execute(
        &self,
        actor_id: BackofficeUserId,
        org_id: OrganizationId,
        reason: String,
        ip: String,
    ) -> Result<Organization, BackofficeIdentityError> {
        // 1. Validate reason — return InvalidInput before touching the DB.
        if reason.trim().is_empty() {
            return Err(BackofficeIdentityError::InvalidInput(
                "reason is required".to_string(),
            ));
        }

        // 2. Begin transaction.
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(BackofficeIdentityError::Database)?;

        // 3. Load org inside the transaction (consistent read).
        let mut org = self
            .org_repo
            .find_by_id_in_tx(&mut tx, org_id)
            .await
            .map_err(tenancy_to_domain_error)?
            .ok_or_else(|| BackofficeIdentityError::OrgNotFound(org_id.into_uuid()))?;

        // 4. Apply the domain state transition (enforces the state machine).
        org.suspend()
            .map_err(|e| BackofficeIdentityError::Tenancy(e.to_string()))?;

        // 5. Persist the updated org inside the transaction.
        self.org_repo
            .update_in_tx(&mut tx, &org)
            .await
            .map_err(tenancy_to_domain_error)?;

        // 6. Build the outbox payload in the same layout that
        //    `BackofficeAuditEvent` serialises to (audit_infra wire format).
        //    We cannot import audit_infra directly because it depends on us.
        let actor_id_uuid: Uuid = actor_id.into_uuid();
        let event_action = "org.suspend";
        let event_type = format!("{}{}", AUDIT_EVENT_TYPE_PREFIX, event_action);
        let payload = serde_json::json!({
            "actor_type": "backoffice_user",
            "actor_id": actor_id_uuid,
            "action": event_action,
            "target_org_id": org_id.into_uuid(),
            "reason": reason,
            "ip": ip,
        });

        // 7. Write the outbox event INSIDE the transaction (FR-AUD-1, C-7).
        self.publish_event
            .execute(
                &mut tx,
                "backoffice",
                &actor_id_uuid.to_string(),
                &event_type,
                payload,
            )
            .await
            .map_err(|e| BackofficeIdentityError::Outbox(e.to_string()))?;

        // 8. Commit — org update + outbox INSERT land atomically.
        tx.commit()
            .await
            .map_err(BackofficeIdentityError::Database)?;

        // 9. Return the updated org.
        Ok(org)
    }
}

/// Map TenancyError to BackofficeIdentityError.
///
/// DB errors are forwarded as `Database`; all other tenancy errors are
/// reported as `Tenancy` with their display string.
fn tenancy_to_domain_error(e: tenancy::TenancyError) -> BackofficeIdentityError {
    match e {
        tenancy::TenancyError::Database(db_err) => BackofficeIdentityError::Database(db_err),
        other => BackofficeIdentityError::Tenancy(other.to_string()),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    // -------------------------------------------------------------------------
    // RED → GREEN: empty reason is rejected BEFORE any DB operation.
    //
    // These tests do NOT require a live database because the validation fires
    // before `pool.begin()`. We construct the use case with a lazy pool that
    // will never actually connect.
    // -------------------------------------------------------------------------

    fn make_use_case() -> SuspendOrganizationWithAuditUseCase {
        // Lazy pool — no real DB connection is made.
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");

        let org_repo: Arc<dyn OrganizationRepository> =
            Arc::new(tenancy::PgOrganizationRepository::new(pool.clone()));

        let outbox_repo = Arc::new(events::PgOutboxRepository::new(pool.clone()));
        let publish_event = Arc::new(PublishEventUseCase::new(outbox_repo));

        SuspendOrganizationWithAuditUseCase::new(pool, org_repo, publish_event)
    }

    #[tokio::test]
    async fn empty_reason_is_rejected() {
        let uc = make_use_case();

        let actor_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));
        let org_id = OrganizationId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));

        let result = uc
            .execute(actor_id, org_id, "".to_string(), "127.0.0.1".to_string())
            .await;

        match result {
            Err(BackofficeIdentityError::InvalidInput(msg)) => {
                assert!(
                    msg.contains("reason"),
                    "error message should mention 'reason'"
                );
            }
            other => panic!("expected InvalidInput, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn whitespace_reason_is_rejected() {
        let uc = make_use_case();

        let actor_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));
        let org_id = OrganizationId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));

        let result = uc
            .execute(
                actor_id,
                org_id,
                "   \t\n".to_string(),
                "127.0.0.1".to_string(),
            )
            .await;

        assert!(
            matches!(result, Err(BackofficeIdentityError::InvalidInput(_))),
            "whitespace-only reason should yield InvalidInput"
        );
    }

    #[tokio::test]
    async fn non_empty_reason_passes_validation_guard() {
        // We only assert the validation guard does not fire.
        // The DB call will fail since there's no live DB, but we get a
        // Database error, NOT InvalidInput.
        let uc = make_use_case();

        let actor_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));
        let org_id = OrganizationId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));

        let result = uc
            .execute(
                actor_id,
                org_id,
                "fraud detected by compliance".to_string(),
                "127.0.0.1".to_string(),
            )
            .await;

        // MUST NOT be InvalidInput — any other error is acceptable here.
        assert!(
            !matches!(result, Err(BackofficeIdentityError::InvalidInput(_))),
            "non-empty reason must not be rejected by the validation guard"
        );
    }
}
