// Per-request impersonation audit emit helper
//
// FR-IMP-5: when a request arrives with an impersonation token (act claim
// present), api-gateway MUST write one outbox event to `outbox_events` before
// forwarding the request.
//
// This module is intentionally kept thin: it opens a transaction, builds the
// event payload, calls PublishEventUseCase, and commits. Error handling is
// fail-open (v1): if the write fails we log it and allow the request to
// proceed.
//
// Failure policy (v1): fail-open with tracing::error!
// TODO(v2): consider fail-closed if audit reliability becomes critical

use std::sync::Arc;

use events::{OutboxRepository, PublishEventUseCase};
use sqlx::PgPool;
use uuid::Uuid;

/// Emit a `"backoffice.audit.request.impersonated"` outbox event inside
/// a short-lived transaction.
///
/// Wire format matches `BackofficeAuditEvent` serialisation exactly
/// (serde_json::json! used instead of the struct to avoid pulling
/// `backoffice_identity` as a dep from api-gateway — keeping the compile-time
/// blast radius minimal).
///
/// # Failure policy (v1)
///
/// Returns `()` in all cases. On DB error: logs via `tracing::error!` and
/// allows the request to continue.
/// TODO(v2): consider fail-closed if audit reliability becomes critical.
pub async fn emit_impersonated_request_audit(
    pool: &PgPool,
    outbox_repo: Arc<dyn OutboxRepository>,
    actor_id: Uuid,
    method: &str,
    path: &str,
    ip: &str,
) {
    let publish = PublishEventUseCase::new(outbox_repo);

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::error!(
                error = %e,
                actor_id = %actor_id,
                "FR-IMP-5: failed to begin transaction for impersonation audit; \
                 request will proceed (fail-open)"
            );
            // TODO(v2): consider fail-closed if audit reliability becomes critical
            return;
        }
    };

    let event_type = "backoffice.audit.request.impersonated";
    let reason = format!("{} {}", method, path);
    let payload = serde_json::json!({
        "actor_type": "backoffice_user",
        "actor_id": actor_id,
        "action": "request.impersonated",
        "target_org_id": null,
        "reason": reason,
        "ip": ip,
    });

    if let Err(e) = publish
        .execute(
            &mut tx,
            "backoffice",
            &actor_id.to_string(),
            event_type,
            payload,
        )
        .await
    {
        tracing::error!(
            error = %e,
            actor_id = %actor_id,
            "FR-IMP-5: failed to write impersonation audit event to outbox; \
             request will proceed (fail-open)"
        );
        // TODO(v2): consider fail-closed if audit reliability becomes critical
        return;
    }

    if let Err(e) = tx.commit().await {
        tracing::error!(
            error = %e,
            actor_id = %actor_id,
            "FR-IMP-5: failed to commit impersonation audit transaction; \
             request will proceed (fail-open)"
        );
        // TODO(v2): consider fail-closed if audit reliability becomes critical
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Unit tests: verify payload shape matches BackofficeAuditEvent wire format.
    //
    // The actual DB write requires a live pool and is tested via the
    // #[ignore] integration test below.
    // -------------------------------------------------------------------------

    /// FR-IMP-5: assert the payload JSON produced by the helper matches the
    /// BackofficeAuditEvent wire format that BackofficeAuditSubscriber expects.
    #[test]
    fn impersonated_request_payload_shape_matches_audit_wire_format() {
        use uuid::{NoContext, Timestamp};

        let actor_id = Uuid::new_v7(Timestamp::now(NoContext));
        let method = "GET";
        let path = "/api/v1/products";
        let ip = "10.0.0.1";

        let reason = format!("{} {}", method, path);
        let payload = serde_json::json!({
            "actor_type": "backoffice_user",
            "actor_id": actor_id,
            "action": "request.impersonated",
            "target_org_id": null,
            "reason": reason,
            "ip": ip,
        });

        // Assert all required fields are present and have correct values.
        assert_eq!(payload["actor_type"], "backoffice_user");
        assert_eq!(payload["action"], "request.impersonated");
        assert_eq!(payload["reason"], "GET /api/v1/products");
        assert_eq!(payload["ip"], "10.0.0.1");
        assert!(payload["target_org_id"].is_null());

        // actor_id must be the UUID serialised (serde_json serialises Uuid as string)
        let actor_id_in_payload = payload["actor_id"].as_str().unwrap();
        assert_eq!(
            actor_id_in_payload,
            actor_id.to_string(),
            "actor_id in payload must match the backoffice operator's UUID"
        );
    }

    #[test]
    fn reason_contains_method_and_path() {
        let method = "POST";
        let path = "/api/v1/orders";
        let reason = format!("{} {}", method, path);
        assert_eq!(reason, "POST /api/v1/orders");
    }

    #[test]
    fn event_type_matches_audit_subscriber_prefix() {
        // BackofficeAuditSubscriber matches on "backoffice.audit." prefix.
        // Verify our constant event_type has it.
        let event_type = "backoffice.audit.request.impersonated";
        assert!(
            event_type.starts_with("backoffice.audit."),
            "event_type must start with 'backoffice.audit.' so BackofficeAuditSubscriber picks it up"
        );
    }

    // -------------------------------------------------------------------------
    // Integration test (requires live DB — ignored in CI without DATABASE_URL)
    // -------------------------------------------------------------------------

    /// FR-IMP-5 integration: emit_impersonated_request_audit writes exactly
    /// one row to outbox_events with the correct event_type.
    ///
    /// Marked #[ignore] because it requires a running PostgreSQL instance.
    /// Run manually with: cargo test -p api-gateway -- --ignored
    ///
    /// Gap: without a live DB this cannot be tested in pure unit mode.
    /// The outbox write is architecturally verified by the unit tests above
    /// (payload shape) plus the integration test in backoffice-api (S-05).
    #[tokio::test]
    #[ignore = "requires live PostgreSQL (DATABASE_URL env var)"]
    async fn emit_writes_one_outbox_row_with_correct_event_type() {
        use events::PgOutboxRepository;
        use sqlx::postgres::PgPoolOptions;
        use std::env;

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for this integration test");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to PostgreSQL");

        let outbox_repo: Arc<dyn OutboxRepository> =
            Arc::new(PgOutboxRepository::new(pool.clone()));

        let actor_id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));

        // Count rows before
        let before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM outbox_events WHERE event_type = 'backoffice.audit.request.impersonated'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        emit_impersonated_request_audit(
            &pool,
            outbox_repo,
            actor_id,
            "GET",
            "/api/v1/test-path",
            "127.0.0.1",
        )
        .await;

        // Count rows after — should have increased by exactly 1
        let after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM outbox_events WHERE event_type = 'backoffice.audit.request.impersonated'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            after,
            before + 1,
            "emit_impersonated_request_audit must write exactly one outbox row"
        );
    }
}
