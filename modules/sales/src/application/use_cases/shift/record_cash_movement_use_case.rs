//! Record cash movement use case

use std::sync::Arc;

use crate::application::dtos::{CashMovementCommand, ShiftResponse};
use crate::domain::repositories::ShiftRepository;
use crate::domain::value_objects::ShiftId;
use crate::SalesError;
use identity::UserId;

/// Use case for recording cash in/out movements
pub struct RecordCashMovementUseCase {
    shift_repo: Arc<dyn ShiftRepository>,
}

impl RecordCashMovementUseCase {
    pub fn new(shift_repo: Arc<dyn ShiftRepository>) -> Self {
        Self { shift_repo }
    }

    /// Record a cash-in movement
    pub async fn cash_in(
        &self,
        cmd: CashMovementCommand,
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

        shift.record_cash_in(cmd.amount)?;

        self.shift_repo.update(&shift).await?;

        Ok(ShiftResponse::from(shift))
    }

    /// Record a cash-out movement
    pub async fn cash_out(
        &self,
        cmd: CashMovementCommand,
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

        shift.record_cash_out(cmd.amount)?;

        self.shift_repo.update(&shift).await?;

        Ok(ShiftResponse::from(shift))
    }
}
