use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::BankTransaction;
use crate::domain::repositories::{BankTransactionFilter, BankTransactionRepository};
use crate::domain::value_objects::{BankAccountId, BankTransactionId, BankTransactionType};

pub struct PgBankTransactionRepository {
    pool: PgPool,
}

impl PgBankTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BankTransactionRepository for PgBankTransactionRepository {
    async fn save(&self, t: &BankTransaction) -> Result<(), CashManagementError> {
        sqlx::query(
            r#"
            INSERT INTO bank_transactions (
                id, bank_account_id, txn_type, amount, reference, description,
                occurred_at, reconciled, reconciliation_id, created_by, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.bank_account_id().into_uuid())
        .bind(t.txn_type().to_string())
        .bind(t.amount())
        .bind(t.reference())
        .bind(t.description())
        .bind(t.occurred_at())
        .bind(t.reconciled())
        .bind(t.reconciliation_id())
        .bind(t.created_by())
        .bind(t.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, t: &BankTransaction) -> Result<(), CashManagementError> {
        let result = sqlx::query(
            r#"
            UPDATE bank_transactions
            SET reconciled        = $2,
                reconciliation_id = $3
            WHERE id = $1
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.reconciled())
        .bind(t.reconciliation_id())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CashManagementError::BankTransactionNotFound(
                t.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BankTransactionId,
    ) -> Result<Option<BankTransaction>, CashManagementError> {
        let row = sqlx::query_as::<_, TxnRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(BankTransaction::try_from).transpose()
    }

    async fn list(
        &self,
        filter: BankTransactionFilter,
    ) -> Result<Vec<BankTransaction>, CashManagementError> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            r#"
            SELECT id, bank_account_id, txn_type, amount, reference, description,
                   occurred_at, reconciled, reconciliation_id, created_by, created_at
            FROM bank_transactions
            WHERE 1 = 1
            "#,
        );
        if let Some(account_id) = filter.bank_account_id {
            qb.push(" AND bank_account_id = ")
                .push_bind(account_id.into_uuid());
        }
        if let Some(from) = filter.from {
            qb.push(" AND occurred_at >= ").push_bind(from);
        }
        if let Some(to) = filter.to {
            qb.push(" AND occurred_at < ").push_bind(to);
        }
        if let Some(reconciled) = filter.reconciled {
            qb.push(" AND reconciled = ").push_bind(reconciled);
        }
        qb.push(" ORDER BY occurred_at DESC, created_at DESC");

        let rows = qb.build_query_as::<TxnRow>().fetch_all(&self.pool).await?;
        rows.into_iter().map(BankTransaction::try_from).collect()
    }

    async fn book_balance(
        &self,
        bank_account_id: BankAccountId,
        opening_balance: Decimal,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Decimal, CashManagementError> {
        let row: (Option<Decimal>,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(amount), 0)::NUMERIC AS net
            FROM bank_transactions
            WHERE bank_account_id = $1
              AND occurred_at >= $2
              AND occurred_at <= $3
            "#,
        )
        .bind(bank_account_id.into_uuid())
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await?;
        Ok(opening_balance + row.0.unwrap_or(Decimal::ZERO))
    }

    async fn has_linked_deposit(
        &self,
        bank_transaction_id: BankTransactionId,
    ) -> Result<bool, CashManagementError> {
        let row: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM cash_deposits WHERE bank_transaction_id = $1 LIMIT 1")
                .bind(bank_transaction_id.into_uuid())
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.is_some())
    }

    async fn mark_range_reconciled(
        &self,
        bank_account_id: BankAccountId,
        reconciliation_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<u64, CashManagementError> {
        let result = sqlx::query(
            r#"
            UPDATE bank_transactions
            SET reconciled = TRUE,
                reconciliation_id = $4
            WHERE bank_account_id = $1
              AND occurred_at >= $2
              AND occurred_at <= $3
              AND reconciled = FALSE
            "#,
        )
        .bind(bank_account_id.into_uuid())
        .bind(from)
        .bind(to)
        .bind(reconciliation_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, bank_account_id, txn_type, amount, reference, description,
       occurred_at, reconciled, reconciliation_id, created_by, created_at
FROM bank_transactions
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct TxnRow {
    id: Uuid,
    bank_account_id: Uuid,
    txn_type: String,
    amount: Decimal,
    reference: Option<String>,
    description: Option<String>,
    occurred_at: DateTime<Utc>,
    reconciled: bool,
    reconciliation_id: Option<Uuid>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl TryFrom<TxnRow> for BankTransaction {
    type Error = CashManagementError;

    fn try_from(row: TxnRow) -> Result<Self, Self::Error> {
        let t = BankTransactionType::from_str(&row.txn_type)?;
        Ok(BankTransaction::reconstitute(
            BankTransactionId::from_uuid(row.id),
            BankAccountId::from_uuid(row.bank_account_id),
            t,
            row.amount,
            row.reference,
            row.description,
            row.occurred_at,
            row.reconciled,
            row.reconciliation_id,
            row.created_by,
            row.created_at,
        ))
    }
}
