//! Create tax rate use case

use std::str::FromStr;
use std::sync::Arc;

use crate::FiscalError;
use crate::application::dtos::{CreateTaxRateCommand, TaxRateResponse};
use crate::domain::entities::TaxRate;
use crate::domain::repositories::TaxRateRepository;
use crate::domain::value_objects::{TaxAppliesTo, TaxType};
use identity::StoreId;

/// Use case for creating a new tax rate
pub struct CreateTaxRateUseCase {
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl CreateTaxRateUseCase {
    pub fn new(tax_rate_repo: Arc<dyn TaxRateRepository>) -> Self {
        Self { tax_rate_repo }
    }

    pub async fn execute(&self, cmd: CreateTaxRateCommand) -> Result<TaxRateResponse, FiscalError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let tax_type = TaxType::from_str(&cmd.tax_type)?;
        let applies_to = TaxAppliesTo::from_str(&cmd.applies_to)?;

        // Check for duplicate name in this store
        let existing = self.tax_rate_repo.find_by_store(store_id).await?;
        if existing.iter().any(|tr| tr.name() == cmd.name) {
            return Err(FiscalError::DuplicateTaxRateName(cmd.name));
        }

        let category_ids = cmd.category_ids.unwrap_or_default();

        // Create the tax rate entity
        let tax_rate = TaxRate::create(
            store_id,
            cmd.name,
            tax_type,
            cmd.rate,
            cmd.is_default,
            applies_to,
            category_ids,
        );

        // Save the tax rate
        self.tax_rate_repo.save(&tax_rate).await?;

        Ok(TaxRateResponse::from(tax_rate))
    }
}
