//! Get tax rate use case

use std::sync::Arc;
use uuid::Uuid;

use crate::FiscalError;
use crate::application::dtos::TaxRateResponse;
use crate::domain::repositories::TaxRateRepository;
use crate::domain::value_objects::TaxRateId;

/// Use case for retrieving a tax rate by ID
pub struct GetTaxRateUseCase {
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl GetTaxRateUseCase {
    pub fn new(tax_rate_repo: Arc<dyn TaxRateRepository>) -> Self {
        Self { tax_rate_repo }
    }

    pub async fn execute(&self, tax_rate_id: Uuid) -> Result<TaxRateResponse, FiscalError> {
        let id = TaxRateId::from_uuid(tax_rate_id);

        let tax_rate = self
            .tax_rate_repo
            .find_by_id(id)
            .await?
            .ok_or(FiscalError::TaxRateNotFound(tax_rate_id))?;

        Ok(TaxRateResponse::from(tax_rate))
    }
}
