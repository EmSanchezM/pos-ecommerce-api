//! Delete tax rate use case

use std::sync::Arc;
use uuid::Uuid;

use crate::FiscalError;
use crate::domain::repositories::TaxRateRepository;
use crate::domain::value_objects::TaxRateId;

/// Use case for deleting (deactivating) a tax rate
pub struct DeleteTaxRateUseCase {
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl DeleteTaxRateUseCase {
    pub fn new(tax_rate_repo: Arc<dyn TaxRateRepository>) -> Self {
        Self { tax_rate_repo }
    }

    pub async fn execute(&self, tax_rate_id: Uuid) -> Result<(), FiscalError> {
        let id = TaxRateId::from_uuid(tax_rate_id);

        let tax_rate = self
            .tax_rate_repo
            .find_by_id(id)
            .await?
            .ok_or(FiscalError::TaxRateNotFound(tax_rate_id))?;

        // Cannot delete a default tax rate
        if tax_rate.is_default() {
            return Err(FiscalError::CannotDeleteDefaultTaxRate);
        }

        self.tax_rate_repo.delete(id).await?;

        Ok(())
    }
}
