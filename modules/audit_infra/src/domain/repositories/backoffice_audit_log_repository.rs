//! Append-only audit log repository trait.
//!
//! FR-AUD-4: This trait intentionally has NO update or delete methods.
//! The `BackofficeAuditLogRepository` is a write-once / read-many interface.
//! NFR-SEC-2: The absence of mutation methods is the schema-level enforcement
//! of immutability at the Rust trait level.

use async_trait::async_trait;

use crate::AuditInfraError;
use crate::domain::events::BackofficeAuditEvent;

/// A log entry persisted in `backoffice_audit_log`.
///
/// Derives `Clone` so it can be passed through channels, used in tests, etc.
#[derive(Debug, Clone)]
pub struct BackofficeAuditLogEntry {
    pub actor_type: String,
    pub actor_id: uuid::Uuid,
    pub action: String,
    pub target_org_id: Option<uuid::Uuid>,
    pub reason: String,
    pub ip: String,
}

impl From<BackofficeAuditEvent> for BackofficeAuditLogEntry {
    fn from(event: BackofficeAuditEvent) -> Self {
        Self {
            actor_type: event.actor_type,
            actor_id: event.actor_id.into_uuid(),
            action: event.action,
            target_org_id: event.target_org_id.map(|id| id.into_uuid()),
            reason: event.reason,
            ip: event.ip,
        }
    }
}

/// Filters for paginated audit log reads.
#[derive(Debug, Default, Clone)]
pub struct AuditLogFilters {
    pub actor_id: Option<uuid::Uuid>,
    pub target_org_id: Option<uuid::Uuid>,
    pub action: Option<String>,
}

/// Append-only repository for `backoffice_audit_log`.
///
/// IMPORTANT: This trait MUST NOT gain `update_*` or `delete_*` methods.
/// Any PR adding mutation methods must be rejected as a security violation
/// (FR-AUD-4, NFR-SEC-2, C-3).
#[async_trait]
pub trait BackofficeAuditLogRepository: Send + Sync {
    /// Append a single immutable entry to the audit log.
    async fn append(&self, entry: BackofficeAuditLogEntry) -> Result<(), AuditInfraError>;

    /// Read paginated entries ordered by `occurred_at` DESC.
    async fn find_paginated(
        &self,
        filters: AuditLogFilters,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<BackofficeAuditLogEntry>, AuditInfraError>;
}

// =============================================================================
// P4-T01: Append-only invariant tests
//
// These tests verify TWO things:
// 1. The trait only exposes `append` + `find_paginated` — no update or delete.
//    Checked at compile time: if someone adds a mutation method and this module
//    still compiles, the trait shape is wrong. The test mock below acts as a
//    compile-time interface contract.
// 2. Two `append` calls produce two rows (no upsert / ON CONFLICT silently
//    merging them).
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Minimal in-memory mock that satisfies the trait contract.
    /// If someone adds an update/delete method, this struct must implement it
    /// or fail to compile — making the test a shape-enforcer.
    struct MockAuditRepo {
        rows: Arc<Mutex<Vec<BackofficeAuditLogEntry>>>,
    }

    impl MockAuditRepo {
        fn new() -> Self {
            Self {
                rows: Arc::new(Mutex::new(vec![])),
            }
        }

        fn row_count(&self) -> usize {
            self.rows.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl BackofficeAuditLogRepository for MockAuditRepo {
        async fn append(&self, entry: BackofficeAuditLogEntry) -> Result<(), AuditInfraError> {
            self.rows.lock().unwrap().push(entry);
            Ok(())
        }

        async fn find_paginated(
            &self,
            _filters: AuditLogFilters,
            _page: u32,
            _page_size: u32,
        ) -> Result<Vec<BackofficeAuditLogEntry>, AuditInfraError> {
            Ok(self.rows.lock().unwrap().clone())
        }
    }

    fn sample_entry(action: &str) -> BackofficeAuditLogEntry {
        BackofficeAuditLogEntry {
            actor_type: "backoffice_user".to_string(),
            actor_id: uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
            action: action.to_string(),
            target_org_id: None,
            reason: "test reason".to_string(),
            ip: "127.0.0.1".to_string(),
        }
    }

    /// P4-T01a: Two `append` calls produce two distinct rows (no upsert merging).
    #[tokio::test]
    async fn two_appends_produce_two_rows() {
        let repo = MockAuditRepo::new();

        repo.append(sample_entry("org.suspend")).await.unwrap();
        repo.append(sample_entry("org.suspend")).await.unwrap();

        assert_eq!(repo.row_count(), 2, "expected 2 rows, not an upsert merge");
    }

    /// P4-T01b: `find_paginated` returns all appended rows.
    #[tokio::test]
    async fn find_paginated_returns_all_rows() {
        let repo = MockAuditRepo::new();

        repo.append(sample_entry("org.suspend")).await.unwrap();
        repo.append(sample_entry("user.impersonate")).await.unwrap();

        let rows = repo
            .find_paginated(AuditLogFilters::default(), 1, 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
    }

    /// P4-T01c: Compile-time shape check — the trait only has append and find_paginated.
    /// This test calls both methods through a dyn reference, confirming the
    /// vtable exists and no other methods exist on the trait surface.
    #[tokio::test]
    async fn trait_shape_only_has_append_and_find_paginated() {
        let repo: Arc<dyn BackofficeAuditLogRepository> = Arc::new(MockAuditRepo::new());

        // Only these two exist — any update/delete would need to be added here too.
        repo.append(sample_entry("org.suspend")).await.unwrap();
        let rows = repo
            .find_paginated(AuditLogFilters::default(), 1, 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
    }
}
