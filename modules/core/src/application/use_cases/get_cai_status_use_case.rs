// GetCaiStatusUseCase - Gets the current CAI status for a terminal
//
// Requirements: 3.4, 3.5
// - Return current CAI status
// - Include warning if CAI expires within 30 days

use std::sync::Arc;

use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::TerminalId;
use crate::error::CoreError;
use crate::CaiStatusResponse;

/// Use case for getting the CAI status of a terminal
///
/// This use case retrieves the current CAI status including:
/// - Current invoice number
/// - Remaining invoices in the range
/// - Expiration date
/// - Whether the range is exhausted
/// - Warning if CAI expires within 30 days
pub struct GetCaiStatusUseCase<T>
where
    T: TerminalRepository,
{
    terminal_repo: Arc<T>,
}

impl<T> GetCaiStatusUseCase<T>
where
    T: TerminalRepository,
{
    /// Creates a new instance of GetCaiStatusUseCase
    pub fn new(terminal_repo: Arc<T>) -> Self {
        Self { terminal_repo }
    }

    /// Executes the use case to get the CAI status
    ///
    /// # Arguments
    /// * `terminal_id` - The ID of the terminal to get CAI status for
    ///
    /// # Returns
    /// * `Ok(CaiStatusResponse)` - The current CAI status with optional expiration warning
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    /// * `Err(CoreError::NoCaiAssigned)` - If no CAI is assigned to the terminal
    pub async fn execute(
        &self,
        terminal_id: TerminalId,
    ) -> Result<CaiStatusResponse, CoreError> {
        // 1. Get terminal
        let terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await?
            .ok_or(CoreError::TerminalNotFound(terminal_id.into_uuid()))?;

        // 2. Get current CAI
        let cai = terminal
            .current_cai()
            .ok_or(CoreError::NoCaiAssigned(terminal_id.into_uuid()))?;

        // 3. Build response with expiration warning if applicable
        // The CaiStatusResponse::from already handles the 30-day warning
        Ok(CaiStatusResponse::from(cai))
    }
}
