// GetStoreDetailUseCase - Gets store details with active terminal count
//
// Requirements: 4.2
// - Obtain store with count of active terminals

use std::sync::Arc;

use identity::{StoreId, StoreRepository};

use crate::application::dtos::StoreDetailResponse;
use crate::domain::repositories::TerminalRepository;
use crate::error::CoreError;

/// Use case for getting store details with active terminal count
///
/// This use case retrieves a store by ID and includes the count of
/// active terminals belonging to that store.
pub struct GetStoreDetailUseCase<S, T>
where
    S: StoreRepository,
    T: TerminalRepository,
{
    store_repo: Arc<S>,
    terminal_repo: Arc<T>,
}

impl<S, T> GetStoreDetailUseCase<S, T>
where
    S: StoreRepository,
    T: TerminalRepository,
{
    /// Creates a new instance of GetStoreDetailUseCase
    pub fn new(store_repo: Arc<S>, terminal_repo: Arc<T>) -> Self {
        Self {
            store_repo,
            terminal_repo,
        }
    }

    /// Executes the use case to get store details
    ///
    /// # Arguments
    /// * `store_id` - The ID of the store to retrieve
    ///
    /// # Returns
    /// * `Ok(StoreDetailResponse)` - The store details with terminal count
    /// * `Err(CoreError::StoreNotFound)` - If the store doesn't exist
    pub async fn execute(&self, store_id: StoreId) -> Result<StoreDetailResponse, CoreError> {
        // 1. Find the store
        let store = self
            .store_repo
            .find_by_id(store_id)
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?
            .ok_or(CoreError::StoreNotFound(store_id.into_uuid()))?;

        // 2. Count active terminals for this store
        let active_terminals_count = self.terminal_repo.count_active_by_store(store_id).await?;

        // 3. Build response
        Ok(StoreDetailResponse {
            id: store_id.into_uuid(),
            name: store.name().to_string(),
            address: store.address().to_string(),
            is_ecommerce: store.is_ecommerce(),
            is_active: store.is_active(),
            active_terminals_count,
            created_at: store.created_at(),
            updated_at: store.updated_at(),
        })
    }
}
