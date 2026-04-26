//! Transaction repository trait

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::PaymentsError;
use crate::domain::entities::Transaction;
use crate::domain::value_objects::{
    PaymentGatewayId, TransactionId, TransactionStatus, TransactionType,
};
use identity::StoreId;
use sales::SaleId;

#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    pub store_id: Option<StoreId>,
    pub gateway_id: Option<PaymentGatewayId>,
    pub sale_id: Option<SaleId>,
    pub transaction_type: Option<TransactionType>,
    pub status: Option<TransactionStatus>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn save(&self, tx: &Transaction) -> Result<(), PaymentsError>;

    async fn find_by_id(&self, id: TransactionId) -> Result<Option<Transaction>, PaymentsError>;

    async fn find_by_gateway_transaction_id(
        &self,
        gateway_tx_id: &str,
    ) -> Result<Option<Transaction>, PaymentsError>;

    async fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<Transaction>, PaymentsError>;

    async fn find_by_sale_id(&self, sale_id: SaleId) -> Result<Vec<Transaction>, PaymentsError>;

    async fn update(&self, tx: &Transaction) -> Result<(), PaymentsError>;

    async fn find_paginated(
        &self,
        filter: TransactionFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Transaction>, i64), PaymentsError>;

    /// All pending transactions for a store with a non-null `reference_number`.
    /// Used by the reconciliation use case to match against bank statements.
    async fn find_pending_for_reconciliation(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<Transaction>, PaymentsError>;
}
