use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{
    AbcClassification, DemandForecast, ReorderPolicy, ReplenishmentSuggestion,
};
use crate::domain::value_objects::{AbcClass, ForecastMethod, ForecastPeriod, SuggestionStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandForecastResponse {
    pub id: Uuid,
    pub product_variant_id: Uuid,
    pub store_id: Uuid,
    pub period: ForecastPeriod,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub method: ForecastMethod,
    pub forecasted_qty: Decimal,
    pub confidence_low: Decimal,
    pub confidence_high: Decimal,
    pub computed_at: DateTime<Utc>,
}

impl From<&DemandForecast> for DemandForecastResponse {
    fn from(f: &DemandForecast) -> Self {
        Self {
            id: f.id().into_uuid(),
            product_variant_id: f.product_variant_id(),
            store_id: f.store_id(),
            period: f.period(),
            period_start: f.period_start(),
            period_end: f.period_end(),
            method: f.method(),
            forecasted_qty: f.forecasted_qty(),
            confidence_low: f.confidence_low(),
            confidence_high: f.confidence_high(),
            computed_at: f.computed_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderPolicyResponse {
    pub id: Uuid,
    pub product_variant_id: Uuid,
    pub store_id: Uuid,
    pub min_qty: Decimal,
    pub max_qty: Decimal,
    pub lead_time_days: i32,
    pub safety_stock_qty: Decimal,
    pub review_cycle_days: i32,
    pub preferred_vendor_id: Option<Uuid>,
    pub is_active: bool,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&ReorderPolicy> for ReorderPolicyResponse {
    fn from(p: &ReorderPolicy) -> Self {
        Self {
            id: p.id().into_uuid(),
            product_variant_id: p.product_variant_id(),
            store_id: p.store_id(),
            min_qty: p.min_qty(),
            max_qty: p.max_qty(),
            lead_time_days: p.lead_time_days(),
            safety_stock_qty: p.safety_stock_qty(),
            review_cycle_days: p.review_cycle_days(),
            preferred_vendor_id: p.preferred_vendor_id(),
            is_active: p.is_active(),
            version: p.version(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplenishmentSuggestionResponse {
    pub id: Uuid,
    pub product_variant_id: Uuid,
    pub store_id: Uuid,
    pub current_stock: Decimal,
    pub forecast_qty: Decimal,
    pub recommended_qty: Decimal,
    pub suggested_vendor_id: Option<Uuid>,
    pub status: SuggestionStatus,
    pub generated_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decided_by: Option<Uuid>,
    pub generated_purchase_order_id: Option<Uuid>,
    pub dismiss_reason: Option<String>,
}

impl From<&ReplenishmentSuggestion> for ReplenishmentSuggestionResponse {
    fn from(s: &ReplenishmentSuggestion) -> Self {
        Self {
            id: s.id().into_uuid(),
            product_variant_id: s.product_variant_id(),
            store_id: s.store_id(),
            current_stock: s.current_stock(),
            forecast_qty: s.forecast_qty(),
            recommended_qty: s.recommended_qty(),
            suggested_vendor_id: s.suggested_vendor_id(),
            status: s.status(),
            generated_at: s.generated_at(),
            decided_at: s.decided_at(),
            decided_by: s.decided_by(),
            generated_purchase_order_id: s.generated_purchase_order_id(),
            dismiss_reason: s.dismiss_reason().map(|x| x.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcClassificationResponse {
    pub id: Uuid,
    pub product_variant_id: Uuid,
    pub store_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub revenue_share: Decimal,
    pub abc_class: AbcClass,
    pub classified_at: DateTime<Utc>,
}

impl From<&AbcClassification> for AbcClassificationResponse {
    fn from(a: &AbcClassification) -> Self {
        Self {
            id: a.id().into_uuid(),
            product_variant_id: a.product_variant_id(),
            store_id: a.store_id(),
            period_start: a.period_start(),
            period_end: a.period_end(),
            revenue_share: a.revenue_share(),
            abc_class: a.abc_class(),
            classified_at: a.classified_at(),
        }
    }
}
