use std::sync::Arc;

use crate::SalesError;
use crate::application::dtos::SaleDetailResponse;
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleId;

/// Use case for shipping an order.
/// Transitions: Processing → Shipped
pub struct ShipOrderUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl ShipOrderUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(&self, sale_id: uuid::Uuid) -> Result<SaleDetailResponse, SalesError> {
        let id = SaleId::from_uuid(sale_id);
        let mut sale = self
            .sale_repo
            .find_by_id_with_details(id)
            .await?
            .ok_or(SalesError::SaleNotFound(sale_id))?;

        sale.ship()?;
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}
