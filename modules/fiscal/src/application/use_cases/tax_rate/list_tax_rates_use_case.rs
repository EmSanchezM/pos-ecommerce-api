//! List tax rates use case

use std::sync::Arc;
use uuid::Uuid;

use crate::FiscalError;
use crate::application::dtos::TaxRateResponse;
use crate::domain::repositories::TaxRateRepository;
use identity::StoreId;

/// Use case for listing all tax rates for a store
pub struct ListTaxRatesUseCase {
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl ListTaxRatesUseCase {
    pub fn new(tax_rate_repo: Arc<dyn TaxRateRepository>) -> Self {
        Self { tax_rate_repo }
    }

    pub async fn execute(&self, store_id: Uuid) -> Result<Vec<TaxRateResponse>, FiscalError> {
        let store_id = StoreId::from_uuid(store_id);

        let tax_rates = self.tax_rate_repo.find_by_store(store_id).await?;

        Ok(tax_rates.iter().map(TaxRateResponse::from).collect())
    }
}
