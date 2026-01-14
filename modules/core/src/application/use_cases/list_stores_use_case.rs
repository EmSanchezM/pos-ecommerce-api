// ListStoresUseCase - Lists stores with pagination and filters
//
// - Support filters by is_active, is_ecommerce
// - Implement pagination (page, page_size)

use std::sync::Arc;

use identity::{Store, StoreRepository};

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
    pub async fn execute(&self, query: ListStoresQuery) -> Result<PaginatedStoresResponse, CoreError> {
        // 1. Fetch all stores
        let all_stores = self
            .store_repo
            .find_all()
            .await
            .map_err(|e| CoreError::Database(sqlx::Error::Protocol(e.to_string())))?;

        // 2. Apply filters
        let filtered_stores: Vec<Store> = all_stores
            .into_iter()
            .filter(|store| {
                // Filter by is_active if specified
                if let Some(is_active) = query.is_active {
                    if store.is_active() != is_active {
                        return false;
                    }
                }
                // Filter by is_ecommerce if specified
                if let Some(is_ecommerce) = query.is_ecommerce {
                    if store.is_ecommerce() != is_ecommerce {
                        return false;
                    }
                }
                true
            })
            .collect();

        // 3. Calculate pagination
        let total = filtered_stores.len() as i64;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE).max(1);
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;
        
        // 4. Apply pagination
        let skip = ((page - 1) * page_size) as usize;
        let items: Vec<StoreListItemResponse> = filtered_stores
            .into_iter()
            .skip(skip)
            .take(page_size as usize)
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
