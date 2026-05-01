//! ShippingRate - tariff for a (method, zone) pair.

use chrono::{DateTime, Datelike, Timelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::ShippingError;
use crate::domain::value_objects::{
    ShippingMethodId, ShippingRateId, ShippingRateType, ShippingZoneId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingRate {
    id: ShippingRateId,
    shipping_method_id: ShippingMethodId,
    shipping_zone_id: ShippingZoneId,
    rate_type: ShippingRateType,
    base_rate: Decimal,
    per_kg_rate: Decimal,
    free_shipping_threshold: Option<Decimal>,
    min_order_amount: Option<Decimal>,
    max_weight_kg: Option<Decimal>,
    currency: String,
    available_days: Option<Vec<i16>>,
    available_hour_start: Option<i16>,
    available_hour_end: Option<i16>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ShippingRate {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        shipping_method_id: ShippingMethodId,
        shipping_zone_id: ShippingZoneId,
        rate_type: ShippingRateType,
        base_rate: Decimal,
        per_kg_rate: Decimal,
        free_shipping_threshold: Option<Decimal>,
        min_order_amount: Option<Decimal>,
        max_weight_kg: Option<Decimal>,
        currency: String,
        available_days: Option<Vec<i16>>,
        available_hour_start: Option<i16>,
        available_hour_end: Option<i16>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ShippingRateId::new(),
            shipping_method_id,
            shipping_zone_id,
            rate_type,
            base_rate,
            per_kg_rate,
            free_shipping_threshold,
            min_order_amount,
            max_weight_kg,
            currency,
            available_days,
            available_hour_start,
            available_hour_end,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ShippingRateId,
        shipping_method_id: ShippingMethodId,
        shipping_zone_id: ShippingZoneId,
        rate_type: ShippingRateType,
        base_rate: Decimal,
        per_kg_rate: Decimal,
        free_shipping_threshold: Option<Decimal>,
        min_order_amount: Option<Decimal>,
        max_weight_kg: Option<Decimal>,
        currency: String,
        available_days: Option<Vec<i16>>,
        available_hour_start: Option<i16>,
        available_hour_end: Option<i16>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            shipping_method_id,
            shipping_zone_id,
            rate_type,
            base_rate,
            per_kg_rate,
            free_shipping_threshold,
            min_order_amount,
            max_weight_kg,
            currency,
            available_days,
            available_hour_start,
            available_hour_end,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Compute the cost for an order. Validates min_order / max_weight limits.
    pub fn calculate(
        &self,
        order_total: Decimal,
        weight_kg: Option<Decimal>,
    ) -> Result<Decimal, ShippingError> {
        if let Some(min) = self.min_order_amount
            && order_total < min
        {
            return Err(ShippingError::BelowMinimumAmount);
        }
        if let (Some(max), Some(w)) = (self.max_weight_kg, weight_kg)
            && w > max
        {
            return Err(ShippingError::ExceedsMaxWeight);
        }

        // Free over threshold short-circuits everything.
        if let Some(threshold) = self.free_shipping_threshold
            && order_total >= threshold
        {
            return Ok(Decimal::ZERO);
        }

        Ok(match self.rate_type {
            ShippingRateType::Flat => self.base_rate,
            ShippingRateType::WeightBased => {
                let w = weight_kg.unwrap_or(Decimal::ZERO);
                self.base_rate + (self.per_kg_rate * w)
            }
            ShippingRateType::OrderBased => self.base_rate,
        })
    }

    /// True when the rate is offered at the supplied instant. Honors
    /// `available_days` (0=Sun .. 6=Sat) and `available_hour_start/end`.
    pub fn is_available_at(&self, when: DateTime<Utc>) -> bool {
        if let Some(days) = &self.available_days {
            let dow = when.weekday().num_days_from_sunday() as i16;
            if !days.contains(&dow) {
                return false;
            }
        }
        match (self.available_hour_start, self.available_hour_end) {
            (Some(start), Some(end)) => {
                let h = when.hour() as i16;
                if start <= end {
                    h >= start && h < end
                } else {
                    // wraps midnight (e.g. 22..6)
                    h >= start || h < end
                }
            }
            _ => true,
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }
    pub fn set_base_rate(&mut self, rate: Decimal) {
        self.base_rate = rate;
        self.touch();
    }
    pub fn set_per_kg_rate(&mut self, rate: Decimal) {
        self.per_kg_rate = rate;
        self.touch();
    }
    pub fn set_free_threshold(&mut self, threshold: Option<Decimal>) {
        self.free_shipping_threshold = threshold;
        self.touch();
    }
    pub fn set_availability(
        &mut self,
        days: Option<Vec<i16>>,
        hour_start: Option<i16>,
        hour_end: Option<i16>,
    ) {
        self.available_days = days;
        self.available_hour_start = hour_start;
        self.available_hour_end = hour_end;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ShippingRateId {
        self.id
    }
    pub fn shipping_method_id(&self) -> ShippingMethodId {
        self.shipping_method_id
    }
    pub fn shipping_zone_id(&self) -> ShippingZoneId {
        self.shipping_zone_id
    }
    pub fn rate_type(&self) -> ShippingRateType {
        self.rate_type
    }
    pub fn base_rate(&self) -> Decimal {
        self.base_rate
    }
    pub fn per_kg_rate(&self) -> Decimal {
        self.per_kg_rate
    }
    pub fn free_shipping_threshold(&self) -> Option<Decimal> {
        self.free_shipping_threshold
    }
    pub fn min_order_amount(&self) -> Option<Decimal> {
        self.min_order_amount
    }
    pub fn max_weight_kg(&self) -> Option<Decimal> {
        self.max_weight_kg
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn available_days(&self) -> Option<&[i16]> {
        self.available_days.as_deref()
    }
    pub fn available_hour_start(&self) -> Option<i16> {
        self.available_hour_start
    }
    pub fn available_hour_end(&self) -> Option<i16> {
        self.available_hour_end
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
