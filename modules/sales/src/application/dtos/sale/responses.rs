//! Sale response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{Payment, Sale, SaleItem};

/// Response for a sale (list item)
#[derive(Debug, Serialize)]
pub struct SaleResponse {
    pub id: Uuid,
    pub sale_number: String,
    pub store_id: Uuid,
    pub terminal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub cashier_id: Option<Uuid>,
    pub shift_id: Option<Uuid>,
    pub sale_type: String,
    pub status: String,
    pub subtotal: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub currency: String,
    pub invoice_number: Option<String>,
    pub item_count: usize,
    pub is_fully_paid: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Sale> for SaleResponse {
    fn from(s: &Sale) -> Self {
        Self {
            id: s.id().into_uuid(),
            sale_number: s.sale_number().to_string(),
            store_id: s.store_id().into_uuid(),
            terminal_id: s.terminal_id().map(|t| t.into_uuid()),
            customer_id: s.customer_id().map(|c| c.into_uuid()),
            cashier_id: s.cashier_id().map(|c| c.into_uuid()),
            shift_id: s.shift_id().map(|sh| sh.into_uuid()),
            sale_type: s.sale_type().to_string(),
            status: s.status().to_string(),
            subtotal: s.subtotal(),
            discount_amount: s.discount_amount(),
            tax_amount: s.tax_amount(),
            total: s.total(),
            currency: s.currency().as_str().to_string(),
            invoice_number: s.invoice_number().map(String::from),
            item_count: s.items().len(),
            is_fully_paid: s.is_fully_paid(),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

/// Detailed response for a sale with items and payments
#[derive(Debug, Serialize)]
pub struct SaleDetailResponse {
    pub id: Uuid,
    pub sale_number: String,
    pub store_id: Uuid,
    pub terminal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub cashier_id: Option<Uuid>,
    pub shift_id: Option<Uuid>,
    pub sale_type: String,
    pub status: String,
    pub order_status: Option<String>,
    pub subtotal: Decimal,
    pub discount_type: Option<String>,
    pub discount_value: Decimal,
    pub discount_amount: Decimal,
    pub tax_amount: Decimal,
    pub total: Decimal,
    pub amount_paid: Decimal,
    pub amount_due: Decimal,
    pub change_given: Decimal,
    pub currency: String,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub internal_notes: Option<String>,
    pub voided_by_id: Option<Uuid>,
    pub voided_at: Option<DateTime<Utc>>,
    pub void_reason: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub items: Vec<SaleItemResponse>,
    pub payments: Vec<PaymentResponse>,
    pub is_fully_paid: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Sale> for SaleDetailResponse {
    fn from(s: Sale) -> Self {
        let items: Vec<SaleItemResponse> = s.items().iter().map(SaleItemResponse::from).collect();
        let payments: Vec<PaymentResponse> = s.payments().iter().map(PaymentResponse::from).collect();

        Self {
            id: s.id().into_uuid(),
            sale_number: s.sale_number().to_string(),
            store_id: s.store_id().into_uuid(),
            terminal_id: s.terminal_id().map(|t| t.into_uuid()),
            customer_id: s.customer_id().map(|c| c.into_uuid()),
            cashier_id: s.cashier_id().map(|c| c.into_uuid()),
            shift_id: s.shift_id().map(|sh| sh.into_uuid()),
            sale_type: s.sale_type().to_string(),
            status: s.status().to_string(),
            order_status: s.order_status().map(|o| o.to_string()),
            subtotal: s.subtotal(),
            discount_type: s.discount_type().map(|d| d.to_string()),
            discount_value: s.discount_value(),
            discount_amount: s.discount_amount(),
            tax_amount: s.tax_amount(),
            total: s.total(),
            amount_paid: s.amount_paid(),
            amount_due: s.amount_due(),
            change_given: s.change_given(),
            currency: s.currency().as_str().to_string(),
            invoice_number: s.invoice_number().map(String::from),
            invoice_date: s.invoice_date(),
            notes: s.notes().map(String::from),
            internal_notes: s.internal_notes().map(String::from),
            voided_by_id: s.voided_by_id().map(|u| u.into_uuid()),
            voided_at: s.voided_at(),
            void_reason: s.void_reason().map(String::from),
            completed_at: s.completed_at(),
            is_fully_paid: s.is_fully_paid(),
            items,
            payments,
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

/// Response for a sale item
#[derive(Debug, Serialize)]
pub struct SaleItemResponse {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub line_number: i32,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: String,
    pub description: String,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    pub unit_price: Decimal,
    pub unit_cost: Decimal,
    pub discount_type: Option<String>,
    pub discount_value: Decimal,
    pub discount_amount: Decimal,
    pub tax_rate: Decimal,
    pub tax_amount: Decimal,
    pub subtotal: Decimal,
    pub total: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&SaleItem> for SaleItemResponse {
    fn from(i: &SaleItem) -> Self {
        Self {
            id: i.id().into_uuid(),
            sale_id: i.sale_id().into_uuid(),
            line_number: i.line_number(),
            product_id: i.product_id().into_uuid(),
            variant_id: i.variant_id().map(|v| v.into_uuid()),
            sku: i.sku().to_string(),
            description: i.description().to_string(),
            quantity: i.quantity(),
            unit_of_measure: i.unit_of_measure().to_string(),
            unit_price: i.unit_price(),
            unit_cost: i.unit_cost(),
            discount_type: i.discount_type().map(|d| d.to_string()),
            discount_value: i.discount_value(),
            discount_amount: i.discount_amount(),
            tax_rate: i.tax_rate(),
            tax_amount: i.tax_amount(),
            subtotal: i.subtotal(),
            total: i.total(),
            notes: i.notes().map(String::from),
            created_at: i.created_at(),
            updated_at: i.updated_at(),
        }
    }
}

/// Response for a payment
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub payment_method: String,
    pub status: String,
    pub amount: Decimal,
    pub currency: String,
    pub reference_number: Option<String>,
    pub authorization_code: Option<String>,
    pub card_last_four: Option<String>,
    pub card_brand: Option<String>,
    pub amount_tendered: Option<Decimal>,
    pub change_given: Option<Decimal>,
    pub refunded_amount: Decimal,
    pub refunded_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub processed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Payment> for PaymentResponse {
    fn from(p: &Payment) -> Self {
        Self {
            id: p.id().into_uuid(),
            sale_id: p.sale_id().into_uuid(),
            payment_method: p.payment_method().to_string(),
            status: p.status().to_string(),
            amount: p.amount(),
            currency: p.currency().as_str().to_string(),
            reference_number: p.reference_number().map(String::from),
            authorization_code: p.authorization_code().map(String::from),
            card_last_four: p.card_last_four().map(String::from),
            card_brand: p.card_brand().map(String::from),
            amount_tendered: p.amount_tendered(),
            change_given: p.change_given(),
            refunded_amount: p.refunded_amount(),
            refunded_at: p.refunded_at(),
            notes: p.notes().map(String::from),
            processed_at: p.processed_at(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

/// Paginated response for sale list
#[derive(Debug, Serialize)]
pub struct SaleListResponse {
    pub data: Vec<SaleResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
