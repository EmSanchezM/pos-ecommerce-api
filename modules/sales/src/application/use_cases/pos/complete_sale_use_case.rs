//! Complete sale use case

use std::sync::Arc;
use uuid::Uuid;

use chrono::NaiveDate;

use crate::application::dtos::SaleDetailResponse;
use crate::domain::repositories::SaleRepository;
use crate::domain::value_objects::SaleId;
use crate::SalesError;

/// Use case for completing a POS sale
pub struct CompleteSaleUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl CompleteSaleUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(
        &self,
        sale_id: Uuid,
        invoice_number: i64,
        cai_number: String,
        invoice_date: NaiveDate,
    ) -> Result<SaleDetailResponse, SalesError> {
        let sale_id = SaleId::from_uuid(sale_id);

        let mut sale = self
            .sale_repo
            .find_by_id_with_items(sale_id)
            .await?
            .ok_or(SalesError::SaleNotFound)?;

        // Complete the sale (validates status and payment)
        sale.complete(invoice_number, cai_number, invoice_date)?;

        // Update sale
        self.sale_repo.update(&sale).await?;

        Ok(SaleDetailResponse::from(sale))
    }
}
