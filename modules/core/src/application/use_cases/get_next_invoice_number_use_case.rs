// GetNextInvoiceNumberUseCase - Gets the next invoice number for a terminal
//
// Requirements: 3.1, 3.2, 3.3
// - Verify terminal is active with CAI assigned
// - Validate CAI is not expired and not exhausted
// - Atomically increment and return number

use std::sync::Arc;

use crate::domain::repositories::TerminalRepository;
use crate::domain::value_objects::TerminalId;
use crate::error::{CaiError, CoreError};
use crate::NextInvoiceNumberResponse;

/// Use case for getting the next invoice number from a terminal
///
/// This use case orchestrates the retrieval of the next invoice number,
/// ensuring all business rules are satisfied:
/// - Terminal must exist and be active
/// - Terminal must have a CAI assigned
/// - CAI must not be expired
/// - CAI range must not be exhausted
/// - Number increment is atomic to prevent duplicates
pub struct GetNextInvoiceNumberUseCase<T>
where
    T: TerminalRepository,
{
    terminal_repo: Arc<T>,
}

impl<T> GetNextInvoiceNumberUseCase<T>
where
    T: TerminalRepository,
{
    /// Creates a new instance of GetNextInvoiceNumberUseCase
    pub fn new(terminal_repo: Arc<T>) -> Self {
        Self { terminal_repo }
    }

    /// Executes the use case to get the next invoice number
    ///
    /// # Arguments
    /// * `terminal_id` - The ID of the terminal to get the next number from
    ///
    /// # Returns
    /// * `Ok(NextInvoiceNumberResponse)` - The next invoice number and related info
    /// * `Err(CoreError::TerminalNotFound)` - If the terminal doesn't exist
    /// * `Err(CoreError::TerminalInactive)` - If the terminal is inactive
    /// * `Err(CoreError::NoCaiAssigned)` - If no CAI is assigned to the terminal
    /// * `Err(CoreError::CaiExpired)` - If the CAI has expired
    /// * `Err(CoreError::CaiRangeExhausted)` - If the CAI range is exhausted
    pub async fn execute(
        &self,
        terminal_id: TerminalId,
    ) -> Result<NextInvoiceNumberResponse, CoreError> {
        // 1. Get terminal
        let terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await?
            .ok_or(CoreError::TerminalNotFound(terminal_id.into_uuid()))?;

        // 2. Check terminal is active
        if !terminal.is_active() {
            return Err(CoreError::TerminalInactive(terminal_id.into_uuid()));
        }

        // 3. Get current CAI
        let cai = terminal
            .current_cai()
            .ok_or(CoreError::NoCaiAssigned(terminal_id.into_uuid()))?;

        // 4. Validate CAI can emit
        cai.can_emit().map_err(|e| match e {
            CaiError::Expired => CoreError::CaiExpired(terminal_id.into_uuid()),
            CaiError::RangeExhausted => CoreError::CaiRangeExhausted(terminal_id.into_uuid()),
        })?;

        // 5. Atomically increment and get number
        // This is done at the repository level to ensure atomicity
        let invoice_number = self
            .terminal_repo
            .increment_and_get_invoice_number(terminal_id)
            .await?;

        // Calculate remaining after this invoice
        let remaining = cai.range_end() - invoice_number;

        Ok(NextInvoiceNumberResponse {
            terminal_id: terminal_id.into_uuid(),
            cai_number: cai.cai_number().as_str().to_string(),
            invoice_number,
            remaining,
        })
    }
}
