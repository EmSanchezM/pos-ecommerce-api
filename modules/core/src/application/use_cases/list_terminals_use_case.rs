// ListTerminalsUseCase - Lists terminals of a store with CAI status
//
// - List terminals of a store with their current CAI status

use std::sync::Arc;

use identity::StoreId;

use crate::application::dtos::{ListTerminalsQuery, PaginatedTerminalsResponse, TerminalResponse};
use crate::domain::repositories::TerminalRepository;
use crate::error::CoreError;

/// Default page size for terminal listing
const DEFAULT_PAGE_SIZE: u32 = 20;
/// Maximum page size allowed
const MAX_PAGE_SIZE: u32 = 100;

/// Use case for listing terminals of a store with pagination and filters
///
/// This use case retrieves terminals for a specific store with optional
/// filtering by active status and pagination support.
pub struct ListTerminalsUseCase<T>
where
    T: TerminalRepository,
{
    terminal_repo: Arc<T>,
}

impl<T> ListTerminalsUseCase<T>
where
    T: TerminalRepository,
{
    /// Creates a new instance of ListTerminalsUseCase
    pub fn new(terminal_repo: Arc<T>) -> Self {
        Self { terminal_repo }
    }

    /// Executes the use case to list terminals for a store
    ///
    /// # Arguments
    /// * `store_id` - The store ID to list terminals for
    /// * `query` - Query parameters for filtering and pagination
    ///
    /// # Returns
    /// * `Ok(PaginatedTerminalsResponse)` - Paginated list of terminals with CAI status
    /// * `Err(CoreError)` - If there was an error fetching terminals
    pub async fn execute(
        &self,
        store_id: StoreId,
        query: ListTerminalsQuery,
    ) -> Result<PaginatedTerminalsResponse, CoreError> {
        // 1. Fetch all terminals for the store
        let all_terminals = self.terminal_repo.find_by_store(store_id).await?;

        // 2. Apply filters
        let filtered_terminals: Vec<_> = all_terminals
            .into_iter()
            .filter(|terminal| {
                // Filter by is_active if specified
                if let Some(is_active) = query.is_active
                    && terminal.is_active() != is_active {
                        return false;
                    }
                true
            })
            .collect();

        // 3. Calculate pagination
        let total = filtered_terminals.len() as i64;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        let total_pages = if total == 0 {
            0
        } else {
            ((total as f64) / (page_size as f64)).ceil() as u32
        };

        // 4. Apply pagination and convert to response DTOs
        let skip = ((page - 1) * page_size) as usize;
        let items: Vec<TerminalResponse> = filtered_terminals
            .into_iter()
            .skip(skip)
            .take(page_size as usize)
            .map(TerminalResponse::from)
            .collect();

        Ok(PaginatedTerminalsResponse {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
