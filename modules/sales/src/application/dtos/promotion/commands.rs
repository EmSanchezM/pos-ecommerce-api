// Command DTOs for promotion operations

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Command to create a new promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromotionCommand {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "percentage", "fixed_amount", "buy_x_get_y"
    pub promotion_type: String,
    pub discount_value: Decimal,
    pub buy_quantity: Option<i32>,
    pub get_quantity: Option<i32>,
    #[serde(default)]
    pub minimum_purchase: Decimal,
    pub maximum_discount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub per_customer_limit: Option<i32>,
    #[serde(default = "default_applies_to")]
    pub applies_to: String,
    #[serde(default)]
    pub product_ids: Vec<Uuid>,
    #[serde(default)]
    pub category_ids: Vec<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub store_id: Option<Uuid>,
}

fn default_applies_to() -> String {
    "order".to_string()
}

/// Command to update an existing promotion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePromotionCommand {
    pub name: Option<String>,
    pub description: Option<String>,
    pub discount_value: Option<Decimal>,
    pub buy_quantity: Option<i32>,
    pub get_quantity: Option<i32>,
    pub minimum_purchase: Option<Decimal>,
    pub maximum_discount: Option<Decimal>,
    pub usage_limit: Option<i32>,
    pub per_customer_limit: Option<i32>,
    pub applies_to: Option<String>,
    pub product_ids: Option<Vec<Uuid>>,
    pub category_ids: Option<Vec<Uuid>>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub store_id: Option<Uuid>,
}

/// Command to apply a promotion to a sale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyPromotionCommand {
    pub sale_id: Uuid,
    pub promotion_code: String,
}
