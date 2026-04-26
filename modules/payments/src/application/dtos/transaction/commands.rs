//! Transaction command DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::value_objects::ManualPaymentDetails;

#[derive(Debug, Deserialize)]
pub struct ProcessOnlinePaymentCommand {
    pub sale_id: Uuid,
    pub store_id: Uuid,
    /// `None` → use the store default gateway
    pub gateway_id: Option<Uuid>,
    pub amount: Decimal,
    pub currency: String,
    pub payment_method: String,
    pub card_token: Option<String>,
    pub return_url: Option<String>,
    pub idempotency_key: String,
    /// Free-form metadata. When `manual_details` is provided it overrides
    /// this field with a JSON-serialized `ManualPaymentDetails`.
    pub metadata: Option<String>,
    /// Customer-provided reference (boleta de depósito, # de transferencia).
    pub reference_number: Option<String>,
    /// Structured details for offline payments (BankTransfer, AgencyDeposit,
    /// CashOnDelivery, …). When present, it is JSON-serialized into the
    /// `metadata` column and `reference_number` is also persisted.
    pub manual_details: Option<ManualPaymentDetails>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessRefundCommand {
    #[serde(default)]
    pub transaction_id: Uuid,
    /// `None` → full refund
    pub amount: Option<Decimal>,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTransactionsQuery {
    pub store_id: Option<Uuid>,
    pub gateway_id: Option<Uuid>,
    pub sale_id: Option<Uuid>,
    pub transaction_type: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// Manually confirms a pending transaction (e.g. after verifying the deposit
/// in the bank statement). `confirmed_by_id` is set by the handler from the
/// authenticated user.
#[derive(Debug, Deserialize, Default)]
pub struct ConfirmTransactionCommand {
    #[serde(default)]
    pub transaction_id: Uuid,
    #[serde(default)]
    pub confirmed_by_id: Uuid,
    /// Optional bank reference captured at confirmation time.
    pub reference_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RejectTransactionCommand {
    #[serde(default)]
    pub transaction_id: Uuid,
    #[serde(default)]
    pub rejected_by_id: Uuid,
    pub reason: String,
}

/// One row from a bank statement upload, used to bulk-confirm pending
/// transactions during reconciliation.
#[derive(Debug, Deserialize, Clone)]
pub struct BankStatementEntry {
    pub reference_number: String,
    pub amount: Decimal,
    pub deposit_date: Option<DateTime<Utc>>,
    pub depositor_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReconcilePaymentsCommand {
    #[serde(default)]
    pub store_id: Uuid,
    #[serde(default)]
    pub confirmed_by_id: Uuid,
    pub entries: Vec<BankStatementEntry>,
}
