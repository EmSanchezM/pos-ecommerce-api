//! Cart command DTOs

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Command to create a new cart
#[derive(Debug, Deserialize)]
pub struct CreateCartCommand {
    pub store_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub currency: String,
}

/// Command to add an item to a cart
#[derive(Debug, Deserialize)]
pub struct AddCartItemCommand {
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: String,
    pub name: String,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    pub unit_price: Decimal,
    pub tax_rate: Decimal,
}

/// Command to update a cart item quantity
#[derive(Debug, Deserialize)]
pub struct UpdateCartItemCommand {
    pub cart_id: Uuid,
    pub item_id: Uuid,
    pub quantity: Decimal,
}
