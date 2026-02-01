//! Void sale use case

use std::sync::Arc;

use crate::application::dtos::{SaleDetailResponse, VoidSaleCommand};
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleId;
use crate::SalesError;
use identity::UserId;

/// Use case for voiding a sale
pub struct VoidSaleUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl VoidSaleUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(
        &self,
        cmd: VoidSaleCommand,
        voided_by: UserId,
    ) -> Result<SaleDetailResponse, SalesError> {
        let sale_id = SaleId::from_uuid(cmd.sale_id);

        let mut sale = self
            .sale_repo
            .find_by_id_with_details(sale_id)
            .await?
            .ok_or(SalesError::SaleNotFound(cmd.sale_id))?;

        // Void the sale
        sale.void(voided_by, cmd.reason)?;

        // Update sale
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}
