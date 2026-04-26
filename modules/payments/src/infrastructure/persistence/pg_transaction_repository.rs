//! PostgreSQL implementation of TransactionRepository.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::PaymentsError;
use crate::domain::entities::Transaction;
use crate::domain::repositories::{TransactionFilter, TransactionRepository};
use crate::domain::value_objects::{
    PaymentGatewayId, TransactionId, TransactionStatus, TransactionType,
};
use identity::{StoreId, UserId};
use sales::{PaymentId, SaleId};

pub struct PgTransactionRepository {
    pool: PgPool,
}

impl PgTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str = r#"
    id, store_id, gateway_id, sale_id, payment_id,
    transaction_type, status, amount, currency,
    gateway_transaction_id, gateway_response, authorization_code,
    card_last_four, card_brand, failure_code, failure_message,
    refund_reason, original_transaction_id, idempotency_key,
    metadata, reference_number,
    confirmed_by_id, confirmed_at,
    rejected_by_id, rejected_at, rejection_reason,
    processed_at, created_at, updated_at
"#;

#[async_trait]
impl TransactionRepository for PgTransactionRepository {
    async fn save(&self, tx: &Transaction) -> Result<(), PaymentsError> {
        sqlx::query(
            r#"
            INSERT INTO payment_transactions (
                id, store_id, gateway_id, sale_id, payment_id,
                transaction_type, status, amount, currency,
                gateway_transaction_id, gateway_response, authorization_code,
                card_last_four, card_brand, failure_code, failure_message,
                refund_reason, original_transaction_id, idempotency_key,
                metadata, reference_number,
                confirmed_by_id, confirmed_at,
                rejected_by_id, rejected_at, rejection_reason,
                processed_at, created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18, $19,
                $20, $21,
                $22, $23,
                $24, $25, $26,
                $27, $28, $29
            )
            "#,
        )
        .bind(tx.id().into_uuid())
        .bind(tx.store_id().into_uuid())
        .bind(tx.gateway_id().into_uuid())
        .bind(tx.sale_id().into_uuid())
        .bind(tx.payment_id().map(|p| p.into_uuid()))
        .bind(tx.transaction_type().to_string())
        .bind(tx.status().to_string())
        .bind(tx.amount())
        .bind(tx.currency())
        .bind(tx.gateway_transaction_id())
        .bind(tx.gateway_response())
        .bind(tx.authorization_code())
        .bind(tx.card_last_four())
        .bind(tx.card_brand())
        .bind(tx.failure_code())
        .bind(tx.failure_message())
        .bind(tx.refund_reason())
        .bind(tx.original_transaction_id().map(|id| id.into_uuid()))
        .bind(tx.idempotency_key())
        .bind(tx.metadata())
        .bind(tx.reference_number())
        .bind(tx.confirmed_by_id().map(|u| u.into_uuid()))
        .bind(tx.confirmed_at())
        .bind(tx.rejected_by_id().map(|u| u.into_uuid()))
        .bind(tx.rejected_at())
        .bind(tx.rejection_reason())
        .bind(tx.processed_at())
        .bind(tx.created_at())
        .bind(tx.updated_at())
        .execute(&self.pool)
        .await
        .map_err(map_idempotency_error(tx.idempotency_key()))?;
        Ok(())
    }

    async fn find_by_id(&self, id: TransactionId) -> Result<Option<Transaction>, PaymentsError> {
        let sql =
            format!("SELECT {SELECT_COLUMNS} FROM payment_transactions WHERE id = $1 LIMIT 1");
        let row = sqlx::query_as::<_, TransactionRow>(&sql)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_gateway_transaction_id(
        &self,
        gateway_tx_id: &str,
    ) -> Result<Option<Transaction>, PaymentsError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM payment_transactions WHERE gateway_transaction_id = $1 LIMIT 1"
        );
        let row = sqlx::query_as::<_, TransactionRow>(&sql)
            .bind(gateway_tx_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_idempotency_key(
        &self,
        key: &str,
    ) -> Result<Option<Transaction>, PaymentsError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM payment_transactions WHERE idempotency_key = $1 LIMIT 1"
        );
        let row = sqlx::query_as::<_, TransactionRow>(&sql)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_sale_id(&self, sale_id: SaleId) -> Result<Vec<Transaction>, PaymentsError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM payment_transactions WHERE sale_id = $1 ORDER BY created_at DESC"
        );
        let rows = sqlx::query_as::<_, TransactionRow>(&sql)
            .bind(sale_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, tx: &Transaction) -> Result<(), PaymentsError> {
        let result = sqlx::query(
            r#"
            UPDATE payment_transactions
            SET status = $2,
                gateway_transaction_id = $3,
                gateway_response = $4,
                authorization_code = $5,
                card_last_four = $6,
                card_brand = $7,
                failure_code = $8,
                failure_message = $9,
                refund_reason = $10,
                reference_number = $11,
                confirmed_by_id = $12,
                confirmed_at = $13,
                rejected_by_id = $14,
                rejected_at = $15,
                rejection_reason = $16,
                processed_at = $17,
                updated_at = $18
            WHERE id = $1
            "#,
        )
        .bind(tx.id().into_uuid())
        .bind(tx.status().to_string())
        .bind(tx.gateway_transaction_id())
        .bind(tx.gateway_response())
        .bind(tx.authorization_code())
        .bind(tx.card_last_four())
        .bind(tx.card_brand())
        .bind(tx.failure_code())
        .bind(tx.failure_message())
        .bind(tx.refund_reason())
        .bind(tx.reference_number())
        .bind(tx.confirmed_by_id().map(|u| u.into_uuid()))
        .bind(tx.confirmed_at())
        .bind(tx.rejected_by_id().map(|u| u.into_uuid()))
        .bind(tx.rejected_at())
        .bind(tx.rejection_reason())
        .bind(tx.processed_at())
        .bind(tx.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PaymentsError::TransactionNotFound(tx.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: TransactionFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Transaction>, i64), PaymentsError> {
        let offset = (page - 1) * page_size;

        let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(format!(
            "SELECT {SELECT_COLUMNS} FROM payment_transactions WHERE 1 = 1"
        ));
        let mut count_qb: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM payment_transactions WHERE 1 = 1");

        push_filters(&mut data_qb, &filter);
        push_filters(&mut count_qb, &filter);

        data_qb.push(" ORDER BY created_at DESC LIMIT ");
        data_qb.push_bind(page_size);
        data_qb.push(" OFFSET ");
        data_qb.push_bind(offset);

        let rows: Vec<TransactionRow> = data_qb
            .build_query_as::<TransactionRow>()
            .fetch_all(&self.pool)
            .await?;
        let total: (i64,) = count_qb
            .build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await?;

        let items: Result<Vec<Transaction>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        Ok((items?, total.0))
    }

    async fn find_pending_for_reconciliation(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<Transaction>, PaymentsError> {
        let sql = format!(
            r#"
            SELECT {SELECT_COLUMNS}
            FROM payment_transactions
            WHERE store_id = $1
              AND status = 'pending'
              AND reference_number IS NOT NULL
            ORDER BY created_at ASC
            "#
        );
        let rows = sqlx::query_as::<_, TransactionRow>(&sql)
            .bind(store_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

fn push_filters<'q>(qb: &mut QueryBuilder<'q, Postgres>, f: &'q TransactionFilter) {
    if let Some(store_id) = f.store_id {
        qb.push(" AND store_id = ");
        qb.push_bind(store_id.into_uuid());
    }
    if let Some(gateway_id) = f.gateway_id {
        qb.push(" AND gateway_id = ");
        qb.push_bind(gateway_id.into_uuid());
    }
    if let Some(sale_id) = f.sale_id {
        qb.push(" AND sale_id = ");
        qb.push_bind(sale_id.into_uuid());
    }
    if let Some(tx_type) = f.transaction_type {
        qb.push(" AND transaction_type = ");
        qb.push_bind(tx_type.to_string());
    }
    if let Some(status) = f.status {
        qb.push(" AND status = ");
        qb.push_bind(status.to_string());
    }
    if let Some(date_from) = f.date_from {
        qb.push(" AND created_at >= ");
        qb.push_bind(date_from);
    }
    if let Some(date_to) = f.date_to {
        qb.push(" AND created_at <= ");
        qb.push_bind(date_to);
    }
    if let Some(search) = &f.search {
        let pattern = format!("%{}%", search);
        qb.push(" AND (gateway_transaction_id ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR idempotency_key ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR reference_number ILIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }
}

fn map_idempotency_error(key: &str) -> impl FnOnce(sqlx::Error) -> PaymentsError + '_ {
    move |err: sqlx::Error| match &err {
        sqlx::Error::Database(db_err)
            if db_err
                .constraint()
                .map(|c| c.contains("idempotency"))
                .unwrap_or(false) =>
        {
            PaymentsError::DuplicateIdempotencyKey(key.to_string())
        }
        _ => PaymentsError::Database(err),
    }
}

#[derive(sqlx::FromRow)]
struct TransactionRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    gateway_id: uuid::Uuid,
    sale_id: uuid::Uuid,
    payment_id: Option<uuid::Uuid>,
    transaction_type: String,
    status: String,
    amount: rust_decimal::Decimal,
    currency: String,
    gateway_transaction_id: Option<String>,
    gateway_response: Option<String>,
    authorization_code: Option<String>,
    card_last_four: Option<String>,
    card_brand: Option<String>,
    failure_code: Option<String>,
    failure_message: Option<String>,
    refund_reason: Option<String>,
    original_transaction_id: Option<uuid::Uuid>,
    idempotency_key: String,
    metadata: Option<String>,
    reference_number: Option<String>,
    confirmed_by_id: Option<uuid::Uuid>,
    confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    rejected_by_id: Option<uuid::Uuid>,
    rejected_at: Option<chrono::DateTime<chrono::Utc>>,
    rejection_reason: Option<String>,
    processed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = PaymentsError;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        let tx_type: TransactionType = row
            .transaction_type
            .parse()
            .map_err(|_| PaymentsError::InvalidTransactionType)?;
        let status: TransactionStatus = row
            .status
            .parse()
            .map_err(|_| PaymentsError::InvalidTransactionStatus)?;

        Ok(Transaction::reconstitute(
            TransactionId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            PaymentGatewayId::from_uuid(row.gateway_id),
            SaleId::from_uuid(row.sale_id),
            row.payment_id.map(PaymentId::from_uuid),
            tx_type,
            status,
            row.amount,
            row.currency,
            row.gateway_transaction_id,
            row.gateway_response,
            row.authorization_code,
            row.card_last_four,
            row.card_brand,
            row.failure_code,
            row.failure_message,
            row.refund_reason,
            row.original_transaction_id.map(TransactionId::from_uuid),
            row.idempotency_key,
            row.metadata,
            row.reference_number,
            row.confirmed_by_id.map(UserId::from_uuid),
            row.confirmed_at,
            row.rejected_by_id.map(UserId::from_uuid),
            row.rejected_at,
            row.rejection_reason,
            row.processed_at,
            row.created_at,
            row.updated_at,
        ))
    }
}
