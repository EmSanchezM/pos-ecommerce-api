//! Get sale use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::SaleDetailResponse;
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleId;
use crate::SalesError;

/// Use case for retrieving a sale by ID
pub struct GetSaleUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl GetSaleUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(&self, sale_id: Uuid) -> Result<SaleDetailResponse, SalesError> {
        let id = SaleId::from_uuid(sale_id);

        let sale = self
            .sale_repo
            .find_by_id_with_details(id)
            .await?
            .ok_or(SalesError::SaleNotFound(sale_id))?;

        Ok(SaleDetailResponse::from(sale))
    }
}
