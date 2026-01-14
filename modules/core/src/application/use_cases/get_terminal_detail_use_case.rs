// GetTerminalDetailUseCase - Gets terminal details with complete CAI history
//
// - Get terminal with complete CAI history

use std::sync::Arc;

use crate::application::dtos::{
    CaiHistoryItemResponse, CaiStatusResponse, TerminalDetailResponse,
};
use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::TerminalId;
use crate::error::CoreError;

/// Use case for getting terminal details with complete CAI history
///
/// This use case retrieves a terminal by ID along with its complete
/// history of CAI ranges, ordered by creation date.
///
/// Requirements: 4.4
pub struct GetTerminalDetailUseCase<T>
where
    T: TerminalRepository,
{
    terminal_repo: Arc<T>,
}

impl<T> GetTerminalDetailUseCase<T>
where
    T: TerminalRepository,
{
    /// Creates a new instance of GetTerminalDetailUseCase
    pub fn new(terminal_repo: Arc<T>) -> Self {
        Self { terminal_repo }
    }

    /// Executes the use case to get terminal details with CAI history
    ///
    /// # Arguments
    /// * `terminal_id` - The terminal ID to get details for
    ///
    /// # Returns
    /// * `Ok(TerminalDetailResponse)` - Terminal details with CAI history
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    /// * `Err(CoreError)` - If there was an error fetching the terminal
    pub async fn execute(
        &self,
        terminal_id: TerminalId,
    ) -> Result<TerminalDetailResponse, CoreError> {
        // 1. Find the terminal
        let terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await?
            .ok_or(CoreError::TerminalNotFound(terminal_id.into_uuid()))?;

        // 2. Get CAI history for the terminal
        let cai_history = self.terminal_repo.get_cai_history(terminal_id).await?;

        // 3. Convert CAI history to response DTOs
        let cai_history_response: Vec<CaiHistoryItemResponse> =
            cai_history.iter().map(CaiHistoryItemResponse::from).collect();

        // 4. Get current CAI status if assigned
        let cai_status = terminal.current_cai().map(CaiStatusResponse::from);

        // 5. Build and return the response
        Ok(TerminalDetailResponse {
            id: terminal.id().into_uuid(),
            store_id: terminal.store_id().into_uuid(),
            code: terminal.code().as_str().to_string(),
            name: terminal.name().to_string(),
            is_active: terminal.is_active(),
            cai_status,
            cai_history: cai_history_response,
            created_at: terminal.created_at(),
            updated_at: terminal.updated_at(),
        })
    }
}
