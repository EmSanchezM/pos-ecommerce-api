use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ShippingRate;

#[derive(Debug, Deserialize)]
pub struct CreateShippingRateCommand {
    pub shipping_method_id: Uuid,
    pub shipping_zone_id: Uuid,
    pub rate_type: String,
    pub base_rate: Decimal,
    #[serde(default)]
    pub per_kg_rate: Decimal,
    pub free_shipping_threshold: Option<Decimal>,
    pub min_order_amount: Option<Decimal>,
    pub max_weight_kg: Option<Decimal>,
    pub currency: String,
    pub available_days: Option<Vec<i16>>,
    pub available_hour_start: Option<i16>,
    pub available_hour_end: Option<i16>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateShippingRateCommand {
    #[serde(default)]
    pub rate_id: Uuid,
    pub base_rate: Option<Decimal>,
    pub per_kg_rate: Option<Decimal>,
    pub free_shipping_threshold: Option<Option<Decimal>>,
    pub is_active: Option<bool>,
    pub available_days: Option<Option<Vec<i16>>>,
    pub available_hour_start: Option<Option<i16>>,
    pub available_hour_end: Option<Option<i16>>,
}

#[derive(Debug, Serialize)]
pub struct ShippingRateResponse {
    pub id: Uuid,
    pub shipping_method_id: Uuid,
    pub shipping_zone_id: Uuid,
    pub rate_type: String,
    pub base_rate: Decimal,
    pub per_kg_rate: Decimal,
    pub free_shipping_threshold: Option<Decimal>,
    pub min_order_amount: Option<Decimal>,
    pub max_weight_kg: Option<Decimal>,
    pub currency: String,
    pub available_days: Option<Vec<i16>>,
    pub available_hour_start: Option<i16>,
    pub available_hour_end: Option<i16>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShippingRate> for ShippingRateResponse {
    fn from(r: ShippingRate) -> Self {
        Self {
            id: r.id().into_uuid(),
            shipping_method_id: r.shipping_method_id().into_uuid(),
            shipping_zone_id: r.shipping_zone_id().into_uuid(),
            rate_type: r.rate_type().to_string(),
            base_rate: r.base_rate(),
            per_kg_rate: r.per_kg_rate(),
            free_shipping_threshold: r.free_shipping_threshold(),
            min_order_amount: r.min_order_amount(),
            max_weight_kg: r.max_weight_kg(),
            currency: r.currency().to_string(),
            available_days: r.available_days().map(<[i16]>::to_vec),
            available_hour_start: r.available_hour_start(),
            available_hour_end: r.available_hour_end(),
            is_active: r.is_active(),
            created_at: r.created_at(),
            updated_at: r.updated_at(),
        }
    }
}

// -----------------------------------------------------------------------------
// CalculateShipping
// -----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CalculateShippingCommand {
    pub store_id: Uuid,
    pub country: String,
    pub state: String,
    pub postal_code: Option<String>,
    pub total_weight_kg: Option<Decimal>,
    pub order_total: Decimal,
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct ShippingOptionResponse {
    pub method_id: Uuid,
    pub method_code: String,
    pub method_name: String,
    pub method_type: String,
    pub zone_id: Uuid,
    pub zone_name: String,
    pub rate_id: Uuid,
    pub rate: Decimal,
    pub currency: String,
    pub estimated_days_min: Option<i32>,
    pub estimated_days_max: Option<i32>,
    pub is_free: bool,
}

#[derive(Debug, Serialize)]
pub struct ShippingOptionsResponse {
    pub options: Vec<ShippingOptionResponse>,
}
