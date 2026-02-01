//! Open shift use case

use std::sync::Arc;

use crate::application::dtos::{OpenShiftCommand, ShiftResponse};
use crate::domain::entities::CashierShift;
use crate::domain::repositories::ShiftRepository;
use crate::SalesError;
use identity::{StoreId, UserId};
use pos_core::TerminalId;

/// Use case for opening a new cashier shift
pub struct OpenShiftUseCase {
    shift_repo: Arc<dyn ShiftRepository>,
}

impl OpenShiftUseCase {
    pub fn new(shift_repo: Arc<dyn ShiftRepository>) -> Self {
        Self { shift_repo }
    }

    pub async fn execute(
        &self,
        cmd: OpenShiftCommand,
        cashier_id: UserId,
    ) -> Result<ShiftResponse, SalesError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let terminal_id = TerminalId::from_uuid(cmd.terminal_id);

        // Check if there's already an open shift for this terminal
        if self
            .shift_repo
            .find_open_shift(terminal_id)
            .await?
            .is_some()
        {
            return Err(SalesError::TerminalHasOpenShift);
        }

        // Check if the cashier already has an open shift elsewhere
        if self
            .shift_repo
            .find_open_shift_by_cashier(cashier_id)
            .await?
            .is_some()
        {
            return Err(SalesError::CashierHasOpenShift);
        }

        // Create new shift
        let shift = CashierShift::create(
            store_id,
            terminal_id,
            cashier_id,
            cmd.opening_balance,
        )?;

        // Save the shift
        self.shift_repo.save(&shift).await?;

        Ok(ShiftResponse::from(shift))
    }
}
