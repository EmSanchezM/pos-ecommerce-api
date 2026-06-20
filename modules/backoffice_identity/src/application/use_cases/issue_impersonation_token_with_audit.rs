// IssueImpersonationTokenWithAuditUseCase
//
// Orchestrates the impersonation flow:
//   1. Validate reason (non-empty guard).
//   2. Load BackofficeUser (the operator) — must exist and be active.
//   3. Issue the impersonation token (pure CPU — delegated to token service).
//   4. Begin a transaction.
//   5. Write the audit outbox event inside the transaction (FR-AUD-1, C-7).
//   6. Commit.
//   7. Return the token DTO.
//
// # Tenant user existence check
//
// The tenant_user_id existence check is intentionally NOT done here because
// it would require a dep on `modules/identity` (tenant domain), which we avoid
// to keep `backoffice_identity` isolated. The handler validates existence
// BEFORE calling this use case (Clean Architecture: handler coordinates,
// use case enforces its own invariants).
//
// # Circular dependency note
//
// `audit_infra` depends on `backoffice_identity` (BackofficeUserId), so
// `backoffice_identity` cannot import `audit_infra`. Same workaround as
// `SuspendOrganizationWithAuditUseCase`: build the outbox payload as
// `serde_json::json!` with the wire format that `BackofficeAuditEvent`
// serialises to. See `suspend_organization_with_audit.rs` lines 111-124.

use std::sync::Arc;

use events::PublishEventUseCase;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::dtos::ImpersonationTokenResponse;
use crate::application::use_cases::IMPERSONATION_TOKEN_EXPIRY_SECONDS;
use crate::domain::auth::BackofficeTokenService;
use crate::domain::repositories::BackofficeUserRepository;
use crate::domain::value_objects::BackofficeUserId;
use crate::error::BackofficeIdentityError;

/// Prefix used by BackofficeAuditSubscriber to match events.
/// Must match `audit_infra::AUDIT_EVENT_TYPE_PREFIX`.
const AUDIT_EVENT_TYPE_PREFIX: &str = "backoffice.audit.";

pub struct IssueImpersonationTokenWithAuditUseCase {
    pool: PgPool,
    user_repo: Arc<dyn BackofficeUserRepository>,
    token_service: Arc<dyn BackofficeTokenService>,
    publish_event: Arc<PublishEventUseCase>,
    tenant_secret: String,
}

impl IssueImpersonationTokenWithAuditUseCase {
    pub fn new(
        pool: PgPool,
        user_repo: Arc<dyn BackofficeUserRepository>,
        token_service: Arc<dyn BackofficeTokenService>,
        publish_event: Arc<PublishEventUseCase>,
        tenant_secret: String,
    ) -> Self {
        Self {
            pool,
            user_repo,
            token_service,
            publish_event,
            tenant_secret,
        }
    }

    /// Issue an impersonation token and write an audit event.
    ///
    /// # Arguments
    ///
    /// * `actor_id` — The authenticated backoffice operator's user ID.
    /// * `tenant_user_id` — The ID of the tenant user to be impersonated.
    /// * `reason` — Required reason for impersonation (NFR-SEC-5).
    /// * `ip` — IP address of the request (for audit log).
    ///
    /// # Errors
    ///
    /// - `InvalidInput` — reason is empty or blank.
    /// - `UserNotFound` — backoffice_user with `actor_id` does not exist.
    /// - `Database`     — SQLx errors.
    /// - `Outbox`       — outbox write failed.
    pub async fn execute(
        &self,
        actor_id: BackofficeUserId,
        tenant_user_id: Uuid,
        reason: String,
        ip: String,
    ) -> Result<ImpersonationTokenResponse, BackofficeIdentityError> {
        // 1. Validate reason before touching the DB.
        if reason.trim().is_empty() {
            return Err(BackofficeIdentityError::InvalidInput(
                "reason is required".to_string(),
            ));
        }

        // 2. Load the backoffice operator.
        let backoffice_user = self
            .user_repo
            .find_by_id(actor_id)
            .await?
            .ok_or_else(|| BackofficeIdentityError::UserNotFound(*actor_id.as_uuid()))?;

        // 3. Issue the impersonation token (pure CPU, no DB).
        let access_token = self.token_service.issue_impersonation_token(
            &backoffice_user,
            tenant_user_id,
            &self.tenant_secret,
        )?;

        // 4. Begin transaction for the audit outbox write.
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(BackofficeIdentityError::Database)?;

        // 5. Build audit payload in the same wire format as BackofficeAuditEvent.
        //    (Cannot import audit_infra directly — see circular dep note above.)
        let actor_id_uuid: Uuid = actor_id.into_uuid();
        let event_action = "user.impersonate";
        let event_type = format!("{}{}", AUDIT_EVENT_TYPE_PREFIX, event_action);
        let payload = serde_json::json!({
            "actor_type": "backoffice_user",
            "actor_id": actor_id_uuid,
            "action": event_action,
            "target_org_id": null,
            "target_tenant_user_id": tenant_user_id,
            "reason": reason,
            "ip": ip,
        });

        // 6. Write outbox event INSIDE transaction (FR-AUD-1, C-7).
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

        // 7. Commit.
        tx.commit()
            .await
            .map_err(BackofficeIdentityError::Database)?;

        // 8. Return the token DTO.
        Ok(ImpersonationTokenResponse {
            access_token,
            expires_in: IMPERSONATION_TOKEN_EXPIRY_SECONDS,
        })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    fn make_use_case() -> IssueImpersonationTokenWithAuditUseCase {
        use crate::infrastructure::{
            JwtBackofficeTokenService, persistence::PgBackofficeUserRepository,
        };

        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");

        let user_repo: Arc<dyn BackofficeUserRepository> =
            Arc::new(PgBackofficeUserRepository::new(pool.clone()));

        let token_service: Arc<dyn BackofficeTokenService> =
            Arc::new(JwtBackofficeTokenService::with_issuer(
                "backoffice-secret-at-least-32-bytes".to_string(),
                "backoffice-api:test".to_string(),
            ));

        let outbox_repo = Arc::new(events::PgOutboxRepository::new(pool.clone()));
        let publish_event = Arc::new(PublishEventUseCase::new(outbox_repo));

        IssueImpersonationTokenWithAuditUseCase::new(
            pool,
            user_repo,
            token_service,
            publish_event,
            "tenant-secret-at-least-32-bytes-long".to_string(),
        )
    }

    /// Empty reason is rejected BEFORE any DB access.
    #[tokio::test]
    async fn empty_reason_is_rejected() {
        let uc = make_use_case();
        let actor_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let result = uc
            .execute(actor_id, tenant_user_id, "".to_string(), "127.0.0.1".to_string())
            .await;

        assert!(
            matches!(result, Err(BackofficeIdentityError::InvalidInput(_))),
            "empty reason must yield InvalidInput"
        );
    }

    /// Whitespace-only reason is rejected.
    #[tokio::test]
    async fn whitespace_reason_is_rejected() {
        let uc = make_use_case();
        let actor_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let result = uc
            .execute(
                actor_id,
                tenant_user_id,
                "   ".to_string(),
                "127.0.0.1".to_string(),
            )
            .await;

        assert!(
            matches!(result, Err(BackofficeIdentityError::InvalidInput(_))),
            "whitespace-only reason must yield InvalidInput"
        );
    }

    /// Non-empty reason passes the validation guard (DB call will fail — no live DB).
    #[tokio::test]
    async fn non_empty_reason_passes_validation_guard() {
        let uc = make_use_case();
        let actor_id = BackofficeUserId::from_uuid(Uuid::new_v7(Timestamp::now(NoContext)));
        let tenant_user_id = Uuid::new_v7(Timestamp::now(NoContext));

        let result = uc
            .execute(
                actor_id,
                tenant_user_id,
                "Investigating support ticket #1234".to_string(),
                "127.0.0.1".to_string(),
            )
            .await;

        // Must NOT be InvalidInput — any other error is fine here (no live DB).
        assert!(
            !matches!(result, Err(BackofficeIdentityError::InvalidInput(_))),
            "non-empty reason must not be rejected by the validation guard"
        );
    }
}
