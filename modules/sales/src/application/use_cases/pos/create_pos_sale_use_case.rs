//! Create POS sale use case

use std::sync::Arc;

use crate::application::dtos::{CreatePosSaleCommand, SaleDetailResponse};
use crate::domain::entities::Sale;
use crate::domain::repositories::{SaleRepository, ShiftRepository};
use crate::domain::value_objects::ShiftId;
use crate::SalesError;
use identity::{StoreId, UserId};
use inventory::Currency;
use pos_core::TerminalId;

/// Use case for creating a new POS sale
pub struct CreatePosSaleUseCase {
    sale_repo: Arc<dyn SaleRepository>,
    shift_repo: Arc<dyn ShiftRepository>,
}

impl CreatePosSaleUseCase {
    pub fn new(
        sale_repo: Arc<dyn SaleRepository>,
        shift_repo: Arc<dyn ShiftRepository>,
    ) -> Self {
        Self {
            sale_repo,
            shift_repo,
        }
    }

    pub async fn execute(
        &self,
        cmd: CreatePosSaleCommand,
        cashier_id: UserId,
    ) -> Result<SaleDetailResponse, SalesError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let terminal_id = TerminalId::from_uuid(cmd.terminal_id);
        let shift_id = ShiftId::from_uuid(cmd.shift_id);

        // Verify the shift exists and is open
        let shift = self
            .shift_repo
            .find_by_id(shift_id)
            .await?
            .ok_or(SalesError::ShiftNotFound(cmd.shift_id))?;

        if !shift.is_open() {
            return Err(SalesError::NoOpenShift);
        }

        // Verify the cashier owns this shift
        if shift.cashier_id() != cashier_id {
            return Err(SalesError::ShiftNotFound(cmd.shift_id)); // User doesn't have access
        }

        // Generate sale number
        let sale_number = self.sale_repo.generate_sale_number(store_id).await?;

        // Create the sale
        let sale = Sale::create_pos(
            sale_number,
            store_id,
            terminal_id,
            shift_id,
            cashier_id,
            Currency::default(),
        );

        // Save the sale
        self.sale_repo.save(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}
