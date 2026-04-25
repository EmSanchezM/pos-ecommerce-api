//! Tax rate command DTOs

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Command to create a new tax rate
#[derive(Debug, Deserialize)]
pub struct CreateTaxRateCommand {
    pub store_id: Uuid,
    pub name: String,
    pub tax_type: String,
    pub rate: Decimal,
    pub is_default: bool,
    pub applies_to: String,
    pub category_ids: Option<Vec<Uuid>>,
}

/// Command to update an existing tax rate
#[derive(Debug, Deserialize)]
pub struct UpdateTaxRateCommand {
    #[serde(default)]
    pub tax_rate_id: Uuid,
    pub name: Option<String>,
    pub rate: Option<Decimal>,
    pub is_default: Option<bool>,
}
