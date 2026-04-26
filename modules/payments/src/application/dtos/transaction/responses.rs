//! Transaction response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::Transaction;

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub gateway_id: Uuid,
    pub sale_id: Uuid,
    pub payment_id: Option<Uuid>,
    pub transaction_type: String,
    pub status: String,
    pub amount: Decimal,
    pub currency: String,
    pub gateway_transaction_id: Option<String>,
    pub authorization_code: Option<String>,
    pub card_last_four: Option<String>,
    pub card_brand: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub refund_reason: Option<String>,
    pub original_transaction_id: Option<Uuid>,
    pub idempotency_key: String,
    pub reference_number: Option<String>,
    pub confirmed_by_id: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub rejected_by_id: Option<Uuid>,
    pub rejected_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        Self {
            id: t.id().into_uuid(),
            store_id: t.store_id().into_uuid(),
            gateway_id: t.gateway_id().into_uuid(),
            sale_id: t.sale_id().into_uuid(),
            payment_id: t.payment_id().map(|p| p.into_uuid()),
            transaction_type: t.transaction_type().to_string(),
            status: t.status().to_string(),
            amount: t.amount(),
            currency: t.currency().to_string(),
            gateway_transaction_id: t.gateway_transaction_id().map(str::to_string),
            authorization_code: t.authorization_code().map(str::to_string),
            card_last_four: t.card_last_four().map(str::to_string),
            card_brand: t.card_brand().map(str::to_string),
            failure_code: t.failure_code().map(str::to_string),
            failure_message: t.failure_message().map(str::to_string),
            refund_reason: t.refund_reason().map(str::to_string),
            original_transaction_id: t.original_transaction_id().map(|id| id.into_uuid()),
            idempotency_key: t.idempotency_key().to_string(),
            reference_number: t.reference_number().map(str::to_string),
            confirmed_by_id: t.confirmed_by_id().map(|u| u.into_uuid()),
            confirmed_at: t.confirmed_at(),
            rejected_by_id: t.rejected_by_id().map(|u| u.into_uuid()),
            rejected_at: t.rejected_at(),
            rejection_reason: t.rejection_reason().map(str::to_string),
            processed_at: t.processed_at(),
            created_at: t.created_at(),
            updated_at: t.updated_at(),
        }
    }
}

impl From<&Transaction> for TransactionResponse {
    fn from(t: &Transaction) -> Self {
        TransactionResponse::from(t.clone())
    }
}

#[derive(Debug, Serialize)]
pub struct TransactionListResponse {
    pub items: Vec<TransactionResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Result of a manual reconciliation run.
#[derive(Debug, Serialize)]
pub struct ReconciliationResponse {
    pub matched_count: i64,
    pub unmatched_count: i64,
    pub auto_confirmed: Vec<TransactionResponse>,
    pub unmatched_references: Vec<String>,
}
