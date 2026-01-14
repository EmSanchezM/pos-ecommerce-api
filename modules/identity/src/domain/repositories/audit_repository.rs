// AuditRepository trait - interface for audit log persistence

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::AuditEntry;
use crate::error::IdentityError;

/// Repository trait for AuditEntry persistence
///
/// Defines the contract for storing and querying audit logs.
/// Audit entries are immutable once created.
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Saves a new audit entry to the repository
    ///
    /// # Errors
    /// - `IdentityError::Database` on database errors
    async fn save(&self, entry: &AuditEntry) -> Result<(), IdentityError>;

    /// Finds all audit entries for a specific entity
    ///
    /// Returns entries ordered by creation time (newest first).
    async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<AuditEntry>, IdentityError>;

    /// Finds all audit entries within a date range
    ///
    /// Returns entries where `created_at` is between `from` (inclusive)
    /// and `to` (exclusive), ordered by creation time (newest first).
    async fn find_by_date_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, IdentityError>;
}
