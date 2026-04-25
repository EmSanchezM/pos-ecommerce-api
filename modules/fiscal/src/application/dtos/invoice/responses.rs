//! Invoice response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{Invoice, InvoiceLine};

/// Full response for an invoice with all details
#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub store_id: Uuid,
    pub terminal_id: Uuid,
    pub sale_id: Uuid,
    pub cai_range_id: Uuid,
    pub invoice_type: String,
    pub status: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_rtn: Option<String>,
    pub customer_address: Option<String>,
    pub currency: String,
    pub subtotal: Decimal,
    pub exempt_amount: Decimal,
    pub taxable_amount_15: Decimal,
    pub taxable_amount_18: Decimal,
    pub tax_15: Decimal,
    pub tax_18: Decimal,
    pub total_tax: Decimal,
    pub discount_amount: Decimal,
    pub total: Decimal,
    pub amount_in_words: String,
    pub payment_method: String,
    pub cai_number: String,
    pub cai_expiry_date: DateTime<Utc>,
    pub range_start: String,
    pub range_end: String,
    pub original_invoice_id: Option<Uuid>,
    pub void_invoice_id: Option<Uuid>,
    pub void_reason: Option<String>,
    pub voided_at: Option<DateTime<Utc>>,
    pub printed_at: Option<DateTime<Utc>>,
    pub emitted_at: DateTime<Utc>,
    pub items: Vec<InvoiceLineResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Invoice> for InvoiceResponse {
    fn from(inv: Invoice) -> Self {
        let items: Vec<InvoiceLineResponse> =
            inv.items().iter().map(InvoiceLineResponse::from).collect();

        Self {
            id: inv.id().into_uuid(),
            invoice_number: inv.invoice_number().to_string(),
            store_id: inv.store_id().into_uuid(),
            terminal_id: inv.terminal_id().into_uuid(),
            sale_id: inv.sale_id().into_uuid(),
            cai_range_id: inv.cai_range_id(),
            invoice_type: inv.invoice_type().to_string(),
            status: inv.status().to_string(),
            customer_id: inv.customer_id().map(|c| c.into_uuid()),
            customer_name: inv.customer_name().to_string(),
            customer_rtn: inv.customer_rtn().map(String::from),
            customer_address: inv.customer_address().map(String::from),
            currency: inv.currency().as_str().to_string(),
            subtotal: inv.subtotal(),
            exempt_amount: inv.exempt_amount(),
            taxable_amount_15: inv.taxable_amount_15(),
            taxable_amount_18: inv.taxable_amount_18(),
            tax_15: inv.tax_15(),
            tax_18: inv.tax_18(),
            total_tax: inv.total_tax(),
            discount_amount: inv.discount_amount(),
            total: inv.total(),
            amount_in_words: inv.amount_in_words().to_string(),
            payment_method: inv.payment_method().to_string(),
            cai_number: inv.cai_number().to_string(),
            cai_expiry_date: inv.cai_expiry_date(),
            range_start: inv.range_start().to_string(),
            range_end: inv.range_end().to_string(),
            original_invoice_id: inv.original_invoice_id().map(|id| id.into_uuid()),
            void_invoice_id: inv.void_invoice_id().map(|id| id.into_uuid()),
            void_reason: inv.void_reason().map(String::from),
            voided_at: inv.voided_at(),
            printed_at: inv.printed_at(),
            emitted_at: inv.emitted_at(),
            items,
            created_at: inv.created_at(),
            updated_at: inv.updated_at(),
        }
    }
}

/// Response for a single invoice line
#[derive(Debug, Serialize)]
pub struct InvoiceLineResponse {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub line_number: i32,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub sku: String,
    pub description: String,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    pub unit_price: Decimal,
    pub discount_amount: Decimal,
    pub tax_type: String,
    pub tax_rate: Decimal,
    pub tax_amount: Decimal,
    pub subtotal: Decimal,
    pub total: Decimal,
    pub is_exempt: bool,
}

impl From<&InvoiceLine> for InvoiceLineResponse {
    fn from(line: &InvoiceLine) -> Self {
        Self {
            id: line.id().into_uuid(),
            invoice_id: line.invoice_id().into_uuid(),
            line_number: line.line_number(),
            product_id: line.product_id(),
            variant_id: line.variant_id(),
            sku: line.sku().to_string(),
            description: line.description().to_string(),
            quantity: line.quantity(),
            unit_of_measure: line.unit_of_measure().to_string(),
            unit_price: line.unit_price(),
            discount_amount: line.discount_amount(),
            tax_type: line.tax_type().to_string(),
            tax_rate: line.tax_rate(),
            tax_amount: line.tax_amount(),
            subtotal: line.subtotal(),
            total: line.total(),
            is_exempt: line.is_exempt(),
        }
    }
}

/// Summary response for invoice list views
#[derive(Debug, Serialize)]
pub struct InvoiceSummaryResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub invoice_type: String,
    pub status: String,
    pub customer_name: String,
    pub customer_rtn: Option<String>,
    pub total: Decimal,
    pub emitted_at: DateTime<Utc>,
}

impl From<&Invoice> for InvoiceSummaryResponse {
    fn from(inv: &Invoice) -> Self {
        Self {
            id: inv.id().into_uuid(),
            invoice_number: inv.invoice_number().to_string(),
            invoice_type: inv.invoice_type().to_string(),
            status: inv.status().to_string(),
            customer_name: inv.customer_name().to_string(),
            customer_rtn: inv.customer_rtn().map(String::from),
            total: inv.total(),
            emitted_at: inv.emitted_at(),
        }
    }
}

/// Response for tax calculation
#[derive(Debug, Serialize)]
pub struct TaxCalculationResponse {
    pub items: Vec<TaxCalculationResultItem>,
    pub subtotal: Decimal,
    pub total_exempt: Decimal,
    pub total_taxable_15: Decimal,
    pub total_taxable_18: Decimal,
    pub total_tax_15: Decimal,
    pub total_tax_18: Decimal,
    pub total_tax: Decimal,
    pub total: Decimal,
}

/// Individual item result from tax calculation
#[derive(Debug, Serialize)]
pub struct TaxCalculationResultItem {
    pub product_id: Uuid,
    pub tax_type: String,
    pub tax_rate: Decimal,
    pub tax_amount: Decimal,
    pub subtotal: Decimal,
    pub total: Decimal,
}

/// Response for a fiscal report
#[derive(Debug, Serialize)]
pub struct FiscalReportResponse {
    pub store_id: Uuid,
    pub date_from: String,
    pub date_to: String,
    pub total_invoices: i64,
    pub total_voided: i64,
    pub total_credit_notes: i64,
    pub total_sales: Decimal,
    pub total_exempt: Decimal,
    pub total_taxable_15: Decimal,
    pub total_taxable_18: Decimal,
    pub total_tax_15: Decimal,
    pub total_tax_18: Decimal,
    pub total_tax: Decimal,
}

/// Paginated response for invoice list
#[derive(Debug, Serialize)]
pub struct InvoiceListResponse {
    pub items: Vec<InvoiceSummaryResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
