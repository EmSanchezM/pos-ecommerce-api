//! Update tax rate use case

use std::sync::Arc;

use crate::FiscalError;
use crate::application::dtos::{TaxRateResponse, UpdateTaxRateCommand};
use crate::domain::repositories::TaxRateRepository;
use crate::domain::value_objects::TaxRateId;

/// Use case for updating an existing tax rate
pub struct UpdateTaxRateUseCase {
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl UpdateTaxRateUseCase {
    pub fn new(tax_rate_repo: Arc<dyn TaxRateRepository>) -> Self {
        Self { tax_rate_repo }
    }

    pub async fn execute(&self, cmd: UpdateTaxRateCommand) -> Result<TaxRateResponse, FiscalError> {
        let tax_rate_id = TaxRateId::from_uuid(cmd.tax_rate_id);

        let mut tax_rate = self
            .tax_rate_repo
            .find_by_id(tax_rate_id)
            .await?
            .ok_or(FiscalError::TaxRateNotFound(cmd.tax_rate_id))?;

        // Check for duplicate name if name is being changed
        if let Some(ref name) = cmd.name.as_ref().filter(|n| n.as_str() != tax_rate.name()) {
            let existing = self
                .tax_rate_repo
                .find_by_store(tax_rate.store_id())
                .await?;
            if existing
                .iter()
                .any(|tr| tr.name() == name.as_str() && tr.id() != tax_rate_id)
            {
                return Err(FiscalError::DuplicateTaxRateName(name.to_string()));
            }
        }

        // Apply updates using entity setter methods
        if let Some(name) = cmd.name {
            tax_rate.set_name(name);
        }
        if let Some(rate) = cmd.rate {
            tax_rate.set_rate(rate);
        }
        if let Some(is_default) = cmd.is_default {
            tax_rate.set_default(is_default);
        }

        // Save
        self.tax_rate_repo.update(&tax_rate).await?;

        Ok(TaxRateResponse::from(tax_rate))
    }
}
