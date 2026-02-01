//! Credit note commands (input DTOs)

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Command to create a new credit note
#[derive(Debug, Deserialize)]
pub struct CreateCreditNoteCommand {
    pub original_sale_id: Uuid,
    pub store_id: Uuid,
    pub original_invoice_number: String,
    pub return_type: String,
    pub return_reason: String,
    pub reason_details: Option<String>,
    pub notes: Option<String>,
}

/// Command to add an item to a credit note
#[derive(Debug, Deserialize)]
pub struct AddCreditNoteItemCommand {
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
}

/// Command to submit a credit note for approval
#[derive(Debug, Deserialize)]
pub struct SubmitCreditNoteCommand {
    pub credit_note_id: Uuid,
}

/// Command to approve a credit note
#[derive(Debug, Deserialize)]
pub struct ApproveCreditNoteCommand {
    pub credit_note_id: Uuid,
}

/// Command to apply (process refund) a credit note
#[derive(Debug, Deserialize)]
pub struct ApplyCreditNoteCommand {
    pub credit_note_id: Uuid,
    pub refund_method: String,
}

/// Command to cancel a credit note
#[derive(Debug, Deserialize)]
pub struct CancelCreditNoteCommand {
    pub credit_note_id: Uuid,
    pub reason: String,
}

/// Query parameters for listing credit notes
#[derive(Debug, Deserialize)]
pub struct ListCreditNotesQuery {
    pub store_id: Option<Uuid>,
    pub original_sale_id: Option<Uuid>,
    pub status: Option<String>,
    pub search: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}
