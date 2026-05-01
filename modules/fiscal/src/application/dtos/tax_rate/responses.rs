//! Tax rate response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::TaxRate;

/// Response for a single tax rate
#[derive(Debug, Serialize)]
pub struct TaxRateResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub tax_type: String,
    pub rate: Decimal,
    pub is_default: bool,
    pub is_active: bool,
    pub applies_to: String,
    pub category_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TaxRate> for TaxRateResponse {
    fn from(tr: TaxRate) -> Self {
        Self {
            id: tr.id().into_uuid(),
            store_id: tr.store_id().into_uuid(),
            name: tr.name().to_string(),
            tax_type: tr.tax_type().to_string(),
            rate: tr.rate(),
            is_default: tr.is_default(),
            is_active: tr.is_active(),
            applies_to: tr.applies_to().to_string(),
            category_ids: tr.category_ids().to_vec(),
            created_at: tr.created_at(),
            updated_at: tr.updated_at(),
        }
    }
}

impl From<&TaxRate> for TaxRateResponse {
    fn from(tr: &TaxRate) -> Self {
        Self {
            id: tr.id().into_uuid(),
            store_id: tr.store_id().into_uuid(),
            name: tr.name().to_string(),
            tax_type: tr.tax_type().to_string(),
            rate: tr.rate(),
            is_default: tr.is_default(),
            is_active: tr.is_active(),
            applies_to: tr.applies_to().to_string(),
            category_ids: tr.category_ids().to_vec(),
            created_at: tr.created_at(),
            updated_at: tr.updated_at(),
        }
    }
}
