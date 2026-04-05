// ListStoresUseCase - Lists stores with pagination and filters
//
// - Support filters by is_active, is_ecommerce
// - Implement pagination (page, page_size)

use std::sync::Arc;

use identity::StoreRepository;

use crate::application::dtos::{ListStoresQuery, PaginatedStoresResponse, StoreListItemResponse};
use crate::error::CoreError;

/// Default page size for store listing
const DEFAULT_PAGE_SIZE: u32 = 20;
/// Maximum page size allowed
const MAX_PAGE_SIZE: u32 = 100;

/// Use case for listing stores with pagination and filters
///
/// This use case retrieves stores with optional filtering by active status
/// and e-commerce type, with pagination support.
pub struct ListStoresUseCase<S>
where
    S: StoreRepository,
{
    store_repo: Arc<S>,
}

impl<S> ListStoresUseCase<S>
where
    S: StoreRepository,
{
    /// Creates a new instance of ListStoresUseCase
    pub fn new(store_repo: Arc<S>) -> Self {
        Self { store_repo }
    }

    /// Executes the use case to list stores with pagination and filters
    ///
    /// # Arguments
    /// * `query` - Query parameters for filtering and pagination
    ///
    /// # Returns
    /// * `Ok(PaginatedStoresResponse)` - Paginated list of stores
    /// * `Err(CoreError)` - If there was an error fetching stores
    pub async fn execute(
        &self,
        query: ListStoresQuery,
    ) -> Result<PaginatedStoresResponse, CoreError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);

        // Delegate filtering and pagination to the repository
        let (stores, total) = self
            .store_repo
            .find_paginated(
                query.is_active,
                query.is_ecommerce,
                page as i64,
                page_size as i64,
            )
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;

        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

        let items: Vec<StoreListItemResponse> = stores
            .into_iter()
            .map(|store| StoreListItemResponse {
                id: store.id().into_uuid(),
                name: store.name().to_string(),
                address: store.address().to_string(),
                is_ecommerce: store.is_ecommerce(),
                is_active: store.is_active(),
                created_at: store.created_at(),
                updated_at: store.updated_at(),
            })
            .collect();

        Ok(PaginatedStoresResponse {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
