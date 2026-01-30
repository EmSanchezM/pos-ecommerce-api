// Command DTOs for purchasing operations
//
// These DTOs represent the input data for various operations in the purchasing module.
// They use primitive types (String, Uuid, Decimal) rather than domain value objects
// to keep the application boundary clean and allow validation in use cases.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Vendor Commands
// =============================================================================

/// Command to create a new vendor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVendorCommand {
    /// Vendor display name
    pub name: String,
    /// Legal/registered name (used to auto-generate vendor code)
    pub legal_name: String,
    /// Tax identification number (must be unique)
    pub tax_id: String,
    /// Optional contact email
    pub email: Option<String>,
    /// Optional contact phone
    pub phone: Option<String>,
    /// Optional physical address
    pub address: Option<String>,
    /// Payment terms in days (default: 30)
    pub payment_terms_days: Option<i32>,
    /// Currency code (ISO 4217, default: "HNL")
    pub currency: Option<String>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Command to update an existing vendor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVendorCommand {
    /// New name (if changing)
    pub name: Option<String>,
    /// New legal name (if changing)
    pub legal_name: Option<String>,
    /// New tax ID (if changing)
    pub tax_id: Option<String>,
    /// New email (if changing)
    pub email: Option<String>,
    /// New phone (if changing)
    pub phone: Option<String>,
    /// New address (if changing)
    pub address: Option<String>,
    /// New payment terms in days (if changing)
    pub payment_terms_days: Option<i32>,
    /// New currency code (if changing)
    pub currency: Option<String>,
    /// New notes (if changing)
    pub notes: Option<String>,
}

// =============================================================================
// Purchase Order Commands
// =============================================================================

/// Command to create a new purchase order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePurchaseOrderCommand {
    /// Store ID where goods will be received
    pub store_id: Uuid,
    /// Vendor ID to order from
    pub vendor_id: Uuid,
    /// Order date (YYYY-MM-DD format)
    pub order_date: String,
    /// Expected delivery date (YYYY-MM-DD format, optional)
    pub expected_delivery_date: Option<String>,
    /// Currency code (ISO 4217, optional - defaults to vendor's currency)
    pub currency: Option<String>,
    /// Payment terms in days (optional - defaults to vendor's terms)
    pub payment_terms_days: Option<i32>,
    /// Optional notes visible to the vendor
    pub notes: Option<String>,
    /// Order line items
    pub items: Vec<CreatePurchaseOrderItemCommand>,
}

/// Line item within a CreatePurchaseOrderCommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePurchaseOrderItemCommand {
    /// Product ID being ordered
    pub product_id: Uuid,
    /// Optional variant ID (if ordering a specific variant)
    pub variant_id: Option<Uuid>,
    /// Line item description
    pub description: String,
    /// Quantity to order
    pub quantity_ordered: Decimal,
    /// Unit of measure: "unit", "kg", "lb", "liter", "oz"
    pub unit_of_measure: String,
    /// Cost per unit
    pub unit_cost: Decimal,
    /// Discount percentage (e.g., 10 for 10%, default: 0)
    #[serde(default)]
    pub discount_percent: Decimal,
    /// Tax percentage (e.g., 15 for 15%, default: 0)
    #[serde(default)]
    pub tax_percent: Decimal,
    /// Optional notes for this line item
    pub notes: Option<String>,
}

/// Command to update an existing purchase order (only in draft status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePurchaseOrderCommand {
    /// New vendor ID (if changing)
    pub vendor_id: Option<Uuid>,
    /// New order date (if changing, YYYY-MM-DD format)
    pub order_date: Option<String>,
    /// New expected delivery date (if changing, YYYY-MM-DD format)
    pub expected_delivery_date: Option<String>,
    /// New payment terms in days (if changing)
    pub payment_terms_days: Option<i32>,
    /// New notes (if changing)
    pub notes: Option<String>,
    /// New internal notes (if changing)
    pub internal_notes: Option<String>,
}

/// Command to add a line item to an existing purchase order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOrderItemCommand {
    /// Product ID being ordered
    pub product_id: Uuid,
    /// Optional variant ID (if ordering a specific variant)
    pub variant_id: Option<Uuid>,
    /// Line item description
    pub description: String,
    /// Quantity to order
    pub quantity_ordered: Decimal,
    /// Unit of measure: "unit", "kg", "lb", "liter", "oz"
    pub unit_of_measure: String,
    /// Cost per unit
    pub unit_cost: Decimal,
    /// Discount percentage (e.g., 10 for 10%)
    pub discount_percent: Option<Decimal>,
    /// Tax percentage (e.g., 15 for 15%)
    pub tax_percent: Option<Decimal>,
    /// Optional notes for this line item
    pub notes: Option<String>,
}

/// Command to update an existing line item on a purchase order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderItemCommand {
    /// New description (if changing)
    pub description: Option<String>,
    /// New quantity ordered (if changing)
    pub quantity_ordered: Option<Decimal>,
    /// New unit cost (if changing)
    pub unit_cost: Option<Decimal>,
    /// New discount percentage (if changing)
    pub discount_percent: Option<Decimal>,
    /// New tax percentage (if changing)
    pub tax_percent: Option<Decimal>,
    /// New notes (if changing)
    pub notes: Option<String>,
}

/// Command to reject a submitted purchase order (returns to draft)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectOrderCommand {
    /// Optional reason for rejection
    pub reason: Option<String>,
}

/// Command to cancel a purchase order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderCommand {
    /// Reason for cancellation (required)
    pub reason: String,
}

// =============================================================================
// Goods Receipt Commands
// =============================================================================

/// Command to create a new goods receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoodsReceiptCommand {
    /// Purchase order ID this receipt is for
    pub purchase_order_id: Uuid,
    /// Store ID where goods are being received
    pub store_id: Uuid,
    /// Receipt date (YYYY-MM-DD format)
    pub receipt_date: String,
    /// Optional notes
    pub notes: Option<String>,
    /// Receipt line items
    pub items: Vec<CreateGoodsReceiptItemCommand>,
}

/// Line item within a CreateGoodsReceiptCommand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoodsReceiptItemCommand {
    /// Purchase order item ID this receipt item corresponds to
    pub purchase_order_item_id: Uuid,
    /// Product ID being received
    pub product_id: Uuid,
    /// Optional variant ID (if receiving a specific variant)
    pub variant_id: Option<Uuid>,
    /// Quantity actually received
    pub quantity_received: Decimal,
    /// Actual unit cost at time of receipt
    pub unit_cost: Decimal,
    /// Optional lot/batch number for traceability
    pub lot_number: Option<String>,
    /// Optional expiry date (YYYY-MM-DD format, for perishable goods)
    pub expiry_date: Option<String>,
    /// Optional notes for this line item
    pub notes: Option<String>,
}

/// Command to update an existing goods receipt (only in draft status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoodsReceiptCommand {
    /// New receipt date (if changing, YYYY-MM-DD format)
    pub receipt_date: Option<String>,
    /// New notes (if changing)
    pub notes: Option<String>,
}
