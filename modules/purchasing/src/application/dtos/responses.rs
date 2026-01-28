// Response DTOs for purchasing operations
//
// These DTOs represent the output data returned from use cases in the purchasing module.
// They are designed for API responses and include all necessary information for clients.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Vendor Responses
// =============================================================================

/// Response for a single vendor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub legal_name: String,
    pub tax_id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub payment_terms_days: i32,
    pub currency: String,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Purchase Order Responses
// =============================================================================

/// Response for a purchase order summary (list view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderResponse {
    pub id: Uuid,
    pub order_number: String,
    pub store_id: Uuid,
    pub vendor_id: Uuid,
    pub status: String,
    pub order_date: NaiveDate,
    pub expected_delivery_date: Option<NaiveDate>,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub discount_amount: Decimal,
    pub total: Decimal,
    pub currency: String,
    pub payment_terms_days: i32,
    pub notes: Option<String>,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for a purchase order with full details including items and workflow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderDetailResponse {
    pub id: Uuid,
    pub order_number: String,
    pub store_id: Uuid,
    pub vendor_id: Uuid,
    pub status: String,
    pub order_date: NaiveDate,
    pub expected_delivery_date: Option<NaiveDate>,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub discount_amount: Decimal,
    pub total: Decimal,
    pub currency: String,
    pub payment_terms_days: i32,
    pub notes: Option<String>,
    pub internal_notes: Option<String>,
    pub created_by_id: Uuid,
    pub submitted_by_id: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub received_by_id: Option<Uuid>,
    pub received_date: Option<NaiveDate>,
    pub cancelled_by_id: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub items: Vec<PurchaseOrderItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for a purchase order line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItemResponse {
    pub id: Uuid,
    pub purchase_order_id: Uuid,
    pub line_number: i32,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub description: String,
    pub quantity_ordered: Decimal,
    pub quantity_received: Decimal,
    pub unit_of_measure: String,
    pub unit_cost: Decimal,
    pub discount_percent: Decimal,
    pub tax_percent: Decimal,
    pub line_total: Decimal,
    pub notes: Option<String>,
}

// =============================================================================
// Goods Receipt Responses
// =============================================================================

/// Response for a goods receipt summary (list view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptResponse {
    pub id: Uuid,
    pub receipt_number: String,
    pub purchase_order_id: Uuid,
    pub store_id: Uuid,
    pub receipt_date: NaiveDate,
    pub status: String,
    pub notes: Option<String>,
    pub received_by_id: Uuid,
    pub confirmed_by_id: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for a goods receipt with full details including items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptDetailResponse {
    pub id: Uuid,
    pub receipt_number: String,
    pub purchase_order_id: Uuid,
    pub store_id: Uuid,
    pub receipt_date: NaiveDate,
    pub status: String,
    pub notes: Option<String>,
    pub received_by_id: Uuid,
    pub confirmed_by_id: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub items: Vec<GoodsReceiptItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for a goods receipt line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptItemResponse {
    pub id: Uuid,
    pub goods_receipt_id: Uuid,
    pub purchase_order_item_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity_received: Decimal,
    pub unit_cost: Decimal,
    pub lot_number: Option<String>,
    pub expiry_date: Option<NaiveDate>,
    pub notes: Option<String>,
}
