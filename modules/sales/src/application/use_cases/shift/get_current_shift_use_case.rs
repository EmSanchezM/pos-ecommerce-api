//! Get current shift use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::ShiftResponse;
use crate::domain::repositories::ShiftRepository;
use crate::SalesError;
use pos_core::TerminalId;

/// Use case for getting the current open shift for a terminal
pub struct GetCurrentShiftUseCase {
    shift_repo: Arc<dyn ShiftRepository>,
}

impl GetCurrentShiftUseCase {
    pub fn new(shift_repo: Arc<dyn ShiftRepository>) -> Self {
        Self { shift_repo }
    }

    pub async fn execute(&self, terminal_id: Uuid) -> Result<ShiftResponse, SalesError> {
        let terminal_id = TerminalId::from_uuid(terminal_id);

        let shift = self
            .shift_repo
            .find_open_by_terminal(terminal_id)
            .await?
            .ok_or(SalesError::NoOpenShift)?;

        Ok(ShiftResponse::from(shift))
    }
}
