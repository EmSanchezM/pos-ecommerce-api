//! Credit note responses (output DTOs)

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{CreditNote, CreditNoteItem};

/// Response DTO for a credit note item
#[derive(Debug, Serialize, Clone)]
pub struct CreditNoteItemResponse {
    pub id: Uuid,
    pub credit_note_id: Uuid,
    pub original_sale_item_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: String,
    pub description: String,
    pub return_quantity: Decimal,
    pub unit_of_measure: String,
    pub unit_price: Decimal,
    pub tax_rate: Decimal,
    pub tax_amount: Decimal,
    pub subtotal: Decimal,
    pub total: Decimal,
    pub restock: bool,
    pub condition: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&CreditNoteItem> for CreditNoteItemResponse {
    fn from(item: &CreditNoteItem) -> Self {
        Self {
            id: item.id().into_uuid(),
            credit_note_id: item.credit_note_id().into_uuid(),
            original_sale_item_id: item.original_sale_item_id().into_uuid(),
            product_id: item.product_id().into_uuid(),
            variant_id: item.variant_id().map(|id| id.into_uuid()),
            sku: item.sku().to_string(),
            description: item.description().to_string(),
            return_quantity: item.return_quantity(),
            unit_of_measure: item.unit_of_measure().to_string(),
            unit_price: item.unit_price(),
            tax_rate: item.tax_rate(),
            tax_amount: item.tax_amount(),
            subtotal: item.subtotal(),
            total: item.total(),
            restock: item.restock(),
            condition: item.condition().map(|s| s.to_string()),
            notes: item.notes().map(|s| s.to_string()),
            created_at: item.created_at(),
            updated_at: item.updated_at(),
        }
    }
}

/// Response DTO for a credit note
#[derive(Debug, Serialize, Clone)]
pub struct CreditNoteResponse {
    pub id: Uuid,
    pub credit_note_number: String,
    pub store_id: Uuid,
    pub original_sale_id: Uuid,
    pub original_invoice_number: String,
    pub status: String,
    pub return_type: String,
    pub return_reason: String,
    pub reason_details: Option<String>,
    pub currency: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub refund_method: Option<String>,
    pub refunded_amount: Decimal,
    pub created_by_id: Uuid,
    pub submitted_by_id: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub applied_by_id: Option<Uuid>,
    pub applied_at: Option<DateTime<Utc>>,
    pub cancelled_by_id: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<CreditNoteItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CreditNote> for CreditNoteResponse {
    fn from(cn: CreditNote) -> Self {
        let items = cn.items().iter().map(CreditNoteItemResponse::from).collect();
        Self {
            id: cn.id().into_uuid(),
            credit_note_number: cn.credit_note_number().to_string(),
            store_id: cn.store_id().into_uuid(),
            original_sale_id: cn.original_sale_id().into_uuid(),
            original_invoice_number: cn.original_invoice_number().to_string(),
            status: cn.status().to_string(),
            return_type: cn.return_type().to_string(),
            return_reason: cn.return_reason().to_string(),
            reason_details: cn.reason_details().map(|s| s.to_string()),
            currency: cn.currency().as_str().to_string(),
            subtotal: cn.subtotal(),
            tax_amount: cn.tax_amount(),
            total: cn.total(),
            refund_method: cn.refund_method().map(|s| s.to_string()),
            refunded_amount: cn.refunded_amount(),
            created_by_id: cn.created_by_id().into_uuid(),
            submitted_by_id: cn.submitted_by_id().map(|id| id.into_uuid()),
            submitted_at: cn.submitted_at(),
            approved_by_id: cn.approved_by_id().map(|id| id.into_uuid()),
            approved_at: cn.approved_at(),
            applied_by_id: cn.applied_by_id().map(|id| id.into_uuid()),
            applied_at: cn.applied_at(),
            cancelled_by_id: cn.cancelled_by_id().map(|id| id.into_uuid()),
            cancelled_at: cn.cancelled_at(),
            cancellation_reason: cn.cancellation_reason().map(|s| s.to_string()),
            notes: cn.notes().map(|s| s.to_string()),
            items,
            created_at: cn.created_at(),
            updated_at: cn.updated_at(),
        }
    }
}

/// Response DTO for credit note list
#[derive(Debug, Serialize)]
pub struct CreditNoteListResponse {
    pub data: Vec<CreditNoteResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
