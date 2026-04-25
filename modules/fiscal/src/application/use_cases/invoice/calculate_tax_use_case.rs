//! Calculate tax use case

use std::sync::Arc;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::FiscalError;
use crate::application::dtos::{
    CalculateTaxCommand, TaxCalculationResponse, TaxCalculationResultItem,
};
use crate::domain::repositories::TaxRateRepository;
use crate::domain::value_objects::TaxType;
use identity::StoreId;

/// Use case for calculating taxes on a set of items
pub struct CalculateTaxUseCase {
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl CalculateTaxUseCase {
    pub fn new(tax_rate_repo: Arc<dyn TaxRateRepository>) -> Self {
        Self { tax_rate_repo }
    }

    pub async fn execute(
        &self,
        cmd: CalculateTaxCommand,
    ) -> Result<TaxCalculationResponse, FiscalError> {
        let store_id = StoreId::from_uuid(cmd.store_id);

        // Load tax rates for the store
        let tax_rates = self.tax_rate_repo.find_by_store(store_id).await?;

        // Find the default rates
        let rate_15 = tax_rates
            .iter()
            .find(|tr| tr.tax_type() == TaxType::Isv15 && tr.is_default())
            .map(|tr| tr.rate())
            .unwrap_or(dec!(15));

        let rate_18 = tax_rates
            .iter()
            .find(|tr| tr.tax_type() == TaxType::Isv18 && tr.is_default())
            .map(|tr| tr.rate())
            .unwrap_or(dec!(18));

        let mut result_items = Vec::new();
        let mut subtotal = Decimal::ZERO;
        let mut total_exempt = Decimal::ZERO;
        let mut total_taxable_15 = Decimal::ZERO;
        let mut total_taxable_18 = Decimal::ZERO;
        let mut total_tax_15 = Decimal::ZERO;
        let mut total_tax_18 = Decimal::ZERO;

        let hundred = dec!(100);

        for item in &cmd.items {
            let line_subtotal = item.unit_price * item.quantity;
            subtotal += line_subtotal;

            if item.is_exempt {
                total_exempt += line_subtotal;
                result_items.push(TaxCalculationResultItem {
                    product_id: item.product_id,
                    tax_type: TaxType::Exempt.to_string(),
                    tax_rate: Decimal::ZERO,
                    tax_amount: Decimal::ZERO,
                    subtotal: line_subtotal,
                    total: line_subtotal,
                });
            } else {
                // Determine tax type based on category or default to ISV 15%
                // For now, check if there's a specific 18% rate for the category
                let (tax_type, rate) = if let Some(_category_id) = item.category_id {
                    // Check if this category has a special 18% rate
                    let has_18_rate = tax_rates
                        .iter()
                        .any(|tr| tr.tax_type() == TaxType::Isv18 && tr.is_active());
                    if has_18_rate {
                        (TaxType::Isv18, rate_18)
                    } else {
                        (TaxType::Isv15, rate_15)
                    }
                } else {
                    (TaxType::Isv15, rate_15)
                };

                let tax_amount = line_subtotal * rate / hundred;

                match tax_type {
                    TaxType::Isv15 => {
                        total_taxable_15 += line_subtotal;
                        total_tax_15 += tax_amount;
                    }
                    TaxType::Isv18 => {
                        total_taxable_18 += line_subtotal;
                        total_tax_18 += tax_amount;
                    }
                    TaxType::Exempt => {}
                }

                result_items.push(TaxCalculationResultItem {
                    product_id: item.product_id,
                    tax_type: tax_type.to_string(),
                    tax_rate: rate,
                    tax_amount,
                    subtotal: line_subtotal,
                    total: line_subtotal + tax_amount,
                });
            }
        }

        let total_tax = total_tax_15 + total_tax_18;
        let total = subtotal + total_tax;

        Ok(TaxCalculationResponse {
            items: result_items,
            subtotal,
            total_exempt,
            total_taxable_15,
            total_taxable_18,
            total_tax_15,
            total_tax_18,
            total_tax,
            total,
        })
    }
}
