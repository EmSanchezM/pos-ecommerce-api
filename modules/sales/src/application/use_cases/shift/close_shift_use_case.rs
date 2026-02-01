//! Close shift use case

use std::sync::Arc;

use crate::application::dtos::{CloseShiftCommand, ShiftResponse};
use crate::domain::repositories::ShiftRepository;
use crate::domain::value_objects::ShiftId;
use crate::SalesError;
use identity::UserId;

/// Use case for closing a cashier shift
pub struct CloseShiftUseCase {
    shift_repo: Arc<dyn ShiftRepository>,
}

impl CloseShiftUseCase {
    pub fn new(shift_repo: Arc<dyn ShiftRepository>) -> Self {
        Self { shift_repo }
    }

    pub async fn execute(
        &self,
        cmd: CloseShiftCommand,
        cashier_id: UserId,
    ) -> Result<ShiftResponse, SalesError> {
        let shift_id = ShiftId::from_uuid(cmd.shift_id);

        let mut shift = self
            .shift_repo
            .find_by_id(shift_id)
            .await?
            .ok_or(SalesError::ShiftNotFound(cmd.shift_id))?;

        // Verify the cashier owns this shift
        if shift.cashier_id() != cashier_id {
            return Err(SalesError::ShiftNotFound(cmd.shift_id)); // User doesn't have access
        }

        // Close the shift
        shift.close(cmd.closing_balance, cmd.closing_notes)?;

        // Save changes
        self.shift_repo.update(&shift).await?;

        Ok(ShiftResponse::from(shift))
    }
}
