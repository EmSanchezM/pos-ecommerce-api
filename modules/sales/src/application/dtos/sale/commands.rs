//! Sale command DTOs

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Command to create a new POS sale
#[derive(Debug, Deserialize)]
pub struct CreatePosSaleCommand {
    pub store_id: Uuid,
    pub terminal_id: Uuid,
    pub shift_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub notes: Option<String>,
}

/// Command to add an item to a sale
#[derive(Debug, Deserialize)]
pub struct AddSaleItemCommand {
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity: Decimal,
    pub unit_price: Option<Decimal>,
    pub notes: Option<String>,
}

/// Command to update a sale item
#[derive(Debug, Deserialize)]
pub struct UpdateSaleItemCommand {
    pub item_id: Uuid,
    pub quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub notes: Option<String>,
}

/// Command to apply a discount to a sale or item
#[derive(Debug, Deserialize)]
pub struct ApplyDiscountCommand {
    pub sale_id: Uuid,
    pub item_id: Option<Uuid>,
    pub discount_type: String,
    pub discount_value: Decimal,
}

/// Command to process a payment
#[derive(Debug, Deserialize)]
pub struct ProcessPaymentCommand {
    pub sale_id: Uuid,
    pub payment_method: String,
    pub amount: Decimal,
    pub reference: Option<String>,
    pub card_last_four: Option<String>,
    pub card_brand: Option<String>,
    pub amount_tendered: Option<Decimal>,
    pub notes: Option<String>,
}

/// Command to void a sale
#[derive(Debug, Deserialize)]
pub struct VoidSaleCommand {
    pub sale_id: Uuid,
    pub reason: String,
}

/// Filter for listing sales
#[derive(Debug, Default, Deserialize)]
pub struct ListSalesQuery {
    pub store_id: Option<Uuid>,
    pub terminal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub cashier_id: Option<Uuid>,
    pub shift_id: Option<Uuid>,
    pub sale_type: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub min_total: Option<Decimal>,
    pub max_total: Option<Decimal>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
