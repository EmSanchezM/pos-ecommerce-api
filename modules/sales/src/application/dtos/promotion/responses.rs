// Response DTOs for promotion operations

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Promotion;

/// Response for a promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub promotion_type: String,
    pub status: String,
    pub discount_value: Decimal,
    pub buy_quantity: Option<i32>,
    pub get_quantity: Option<i32>,
    pub minimum_purchase: Decimal,
    pub maximum_discount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub usage_count: i32,
    pub per_customer_limit: Option<i32>,
    pub applies_to: String,
    pub product_ids: Vec<Uuid>,
    pub category_ids: Vec<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub store_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Promotion> for PromotionResponse {
    fn from(p: &Promotion) -> Self {
        Self {
            id: p.id().into_uuid(),
            code: p.code().to_string(),
            name: p.name().to_string(),
            description: p.description().map(|s| s.to_string()),
            promotion_type: p.promotion_type().to_string(),
            status: p.status().to_string(),
            discount_value: p.discount_value(),
            buy_quantity: p.buy_quantity(),
            get_quantity: p.get_quantity(),
            minimum_purchase: p.minimum_purchase(),
            maximum_discount: p.maximum_discount(),
            usage_limit: p.usage_limit(),
            usage_count: p.usage_count(),
            per_customer_limit: p.per_customer_limit(),
            applies_to: p.applies_to().to_string(),
            product_ids: p.product_ids().to_vec(),
            category_ids: p.category_ids().to_vec(),
            start_date: p.start_date(),
            end_date: p.end_date(),
            store_id: p.store_id(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

impl From<Promotion> for PromotionResponse {
    fn from(p: Promotion) -> Self {
        Self::from(&p)
    }
}
