//! Cart response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{Cart, CartItem};

/// Response for a cart with items
#[derive(Debug, Serialize)]
pub struct CartResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub currency: String,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub item_count: i32,
    pub items: Vec<CartItemResponse>,
    pub expires_at: DateTime<Utc>,
    pub is_expired: bool,
    pub is_active: bool,
    pub converted_to_sale: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Cart> for CartResponse {
    fn from(c: Cart) -> Self {
        let items: Vec<CartItemResponse> = c.items().iter().map(CartItemResponse::from).collect();
        Self {
            id: c.id().into_uuid(),
            store_id: c.store_id().into_uuid(),
            customer_id: c.customer_id().map(|id| id.into_uuid()),
            session_id: c.session_id().map(String::from),
            currency: c.currency().as_str().to_string(),
            subtotal: c.subtotal(),
            discount_amount: c.discount_amount(),
            tax_amount: c.tax_amount(),
            total: c.total(),
            item_count: c.item_count(),
            is_expired: c.is_expired(),
            is_active: c.is_active(),
            converted_to_sale: c.converted_to_sale(),
            notes: c.notes().map(String::from),
            expires_at: c.expires_at(),
            created_at: c.created_at(),
            updated_at: c.updated_at(),
            items,
        }
    }
}

/// Response for a cart item
#[derive(Debug, Serialize)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: String,
    pub name: String,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    pub unit_price: Decimal,
    pub discount_amount: Decimal,
    pub tax_rate: Decimal,
    pub tax_amount: Decimal,
    pub subtotal: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&CartItem> for CartItemResponse {
    fn from(i: &CartItem) -> Self {
        Self {
            id: i.id().into_uuid(),
            cart_id: i.cart_id().into_uuid(),
            product_id: i.product_id().into_uuid(),
            variant_id: i.variant_id().map(|v| v.into_uuid()),
            sku: i.sku().to_string(),
            name: i.name().to_string(),
            quantity: i.quantity(),
            unit_of_measure: i.unit_of_measure().to_string(),
            unit_price: i.unit_price(),
            discount_amount: i.discount_amount(),
            tax_rate: i.tax_rate(),
            tax_amount: i.tax_amount(),
            subtotal: i.subtotal(),
            created_at: i.created_at(),
            updated_at: i.updated_at(),
        }
    }
}
