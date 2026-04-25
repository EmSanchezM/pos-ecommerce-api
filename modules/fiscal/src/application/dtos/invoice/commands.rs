//! Invoice command DTOs

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Command to generate a fiscal invoice from a completed sale
#[derive(Debug, Deserialize)]
pub struct GenerateInvoiceCommand {
    pub sale_id: Uuid,
    pub store_id: Uuid,
    pub terminal_id: Uuid,
    pub customer_name: String,
    pub customer_rtn: Option<String>,
    pub customer_address: Option<String>,
    pub invoice_type: String,
}

/// Command to void an emitted invoice
#[derive(Debug, Deserialize)]
pub struct VoidInvoiceCommand {
    #[serde(default)]
    pub invoice_id: Uuid,
    pub reason: String,
}

/// Command to calculate taxes for a set of items
#[derive(Debug, Deserialize)]
pub struct CalculateTaxCommand {
    pub store_id: Uuid,
    pub items: Vec<TaxCalculationItem>,
}

/// Individual item for tax calculation
#[derive(Debug, Deserialize)]
pub struct TaxCalculationItem {
    pub product_id: Uuid,
    pub category_id: Option<Uuid>,
    pub unit_price: Decimal,
    pub quantity: Decimal,
    pub is_exempt: bool,
}

/// Filter for listing invoices
#[derive(Debug, Default, Deserialize)]
pub struct ListInvoicesQuery {
    pub store_id: Option<Uuid>,
    pub terminal_id: Option<Uuid>,
    pub invoice_type: Option<String>,
    pub status: Option<String>,
    pub customer_rtn: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// Command to generate a fiscal report for a date range
#[derive(Debug, Deserialize)]
pub struct FiscalReportCommand {
    pub store_id: Uuid,
    pub date_from: String,
    pub date_to: String,
}
