// SetStoreActiveUseCaseExtended - Activates or deactivates a store with terminal cascade
//
// Requirements: 1.3, 1.4, 6.1
// - When deactivating: cascade deactivation to all terminals
// - When reactivating: do NOT cascade to terminals (they remain in their state)
// - Register audit entry

use std::sync::Arc;

use identity::{AuditEntry, AuditRepository, Store, StoreId, StoreRepository, UserId};

use crate::domain::repositories::TerminalRepository;
use crate::error::CoreError;

/// Use case for activating or deactivating a store with terminal cascade
///
/// This use case extends the basic store activation/deactivation by:
/// - Cascading deactivation to all terminals when a store is deactivated
/// - NOT cascading reactivation (terminals remain in their previous state)
/// - Recording audit trail for all changes
pub struct SetStoreActiveUseCaseExtended<S, T, A>
where
    S: StoreRepository,
    T: TerminalRepository,
    A: AuditRepository,
{
    store_repo: Arc<S>,
    terminal_repo: Arc<T>,
    audit_repo: Arc<A>,
}

impl<S, T, A> SetStoreActiveUseCaseExtended<S, T, A>
where
    S: StoreRepository,
    T: TerminalRepository,
    A: AuditRepository,
{
    /// Creates a new instance of SetStoreActiveUseCaseExtended
    pub fn new(store_repo: Arc<S>, terminal_repo: Arc<T>, audit_repo: Arc<A>) -> Self {
        Self {
            store_repo,
            terminal_repo,
            audit_repo,
        }
    }

    /// Executes the use case to activate a store
    ///
    /// Note: Reactivating a store does NOT automatically reactivate its terminals.
    /// Terminals must be reactivated individually.
    ///
    /// # Arguments
    /// * `store_id` - The ID of the store to activate
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(Store)` - The activated store
    /// * `Err(CoreError::StoreNotFound)` - If the store doesn't exist
    pub async fn activate(
        &self,
        store_id: StoreId,
        actor_id: UserId,
    ) -> Result<Store, CoreError> {
        self.set_active(store_id, true, actor_id).await
    }

    /// Executes the use case to deactivate a store
    ///
    /// When a store is deactivated, all its terminals are also deactivated.
    /// CAI history is preserved for all terminals.
    ///
    /// # Arguments
    /// * `store_id` - The ID of the store to deactivate
    /// * `actor_id` - The ID of the user performing the action (for audit)
    ///
    /// # Returns
    /// * `Ok(Store)` - The deactivated store
    /// * `Err(CoreError::StoreNotFound)` - If the store doesn't exist
    pub async fn deactivate(
        &self,
        store_id: StoreId,
        actor_id: UserId,
    ) -> Result<Store, CoreError> {
        self.set_active(store_id, false, actor_id).await
    }

    /// Internal method to set store active status
    async fn set_active(
        &self,
        store_id: StoreId,
        active: bool,
        actor_id: UserId,
    ) -> Result<Store, CoreError> {
        // 1. Find existing store
        let mut store = self
            .store_repo
            .find_by_id(store_id)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?
            .ok_or(CoreError::StoreNotFound(store_id.into_uuid()))?;

        // 2. Check if state change is needed
        if store.is_active() == active {
            return Ok(store);
        }

        // 3. Clone old state for audit
        let old_store = store.clone();

        // 4. Update active status
        if active {
            store.activate();
            // Note: When reactivating, we do NOT cascade to terminals
            // Terminals must be reactivated individually (Requirement 1.4)
        } else {
            store.deactivate();
            // Cascade deactivation to all terminals (Requirement 1.3)
            self.terminal_repo.deactivate_by_store(store_id).await?;
        }

        // 5. Save store changes
        self.store_repo
            .update(&store)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;

        // 6. Audit
        let audit = AuditEntry::for_update(
            "store",
            store_id.into_uuid(),
            &old_store,
            &store,
            actor_id,
        );
        self.audit_repo
            .save(&audit)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(store)
    }
}
