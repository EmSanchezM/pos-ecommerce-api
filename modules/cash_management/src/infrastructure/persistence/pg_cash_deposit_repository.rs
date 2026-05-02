use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::CashDeposit;
use crate::domain::repositories::CashDepositRepository;
use crate::domain::value_objects::{
    BankAccountId, BankTransactionId, CashDepositId, CashDepositStatus,
};

pub struct PgCashDepositRepository {
    pool: PgPool,
}

impl PgCashDepositRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CashDepositRepository for PgCashDepositRepository {
    async fn save(&self, d: &CashDeposit) -> Result<(), CashManagementError> {
        sqlx::query(
            r#"
            INSERT INTO cash_deposits (
                id, cashier_shift_id, bank_account_id, amount, deposit_date,
                deposit_slip_number, deposited_by_user_id, bank_transaction_id,
                status, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.cashier_shift_id())
        .bind(d.bank_account_id().into_uuid())
        .bind(d.amount())
        .bind(d.deposit_date())
        .bind(d.deposit_slip_number())
        .bind(d.deposited_by_user_id())
        .bind(d.bank_transaction_id().map(|t| t.into_uuid()))
        .bind(d.status().to_string())
        .bind(d.notes())
        .bind(d.created_at())
        .bind(d.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, d: &CashDeposit) -> Result<(), CashManagementError> {
        let result = sqlx::query(
            r#"
            UPDATE cash_deposits
            SET deposit_slip_number   = $2,
                deposited_by_user_id  = $3,
                bank_transaction_id   = $4,
                status                = $5,
                notes                 = $6,
                updated_at            = $7
            WHERE id = $1
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.deposit_slip_number())
        .bind(d.deposited_by_user_id())
        .bind(d.bank_transaction_id().map(|t| t.into_uuid()))
        .bind(d.status().to_string())
        .bind(d.notes())
        .bind(d.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CashManagementError::CashDepositNotFound(d.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: CashDepositId,
    ) -> Result<Option<CashDeposit>, CashManagementError> {
        let row = sqlx::query_as::<_, DepositRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(CashDeposit::try_from).transpose()
    }

    async fn find_by_shift(
        &self,
        cashier_shift_id: Uuid,
    ) -> Result<Option<CashDeposit>, CashManagementError> {
        let row = sqlx::query_as::<_, DepositRow>(SELECT_BY_SHIFT)
            .bind(cashier_shift_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(CashDeposit::try_from).transpose()
    }

    async fn list(
        &self,
        store_id: Option<Uuid>,
        status: Option<CashDepositStatus>,
    ) -> Result<Vec<CashDeposit>, CashManagementError> {
        // Joins through cashier_shifts when filtering by store, since the
        // store_id lives over there. Without a store filter this is a plain
        // table scan ordered by created_at desc.
        let rows = match (store_id, status) {
            (Some(s), Some(st)) => {
                sqlx::query_as::<_, DepositRow>(
                    r#"
                    SELECT cd.id, cd.cashier_shift_id, cd.bank_account_id, cd.amount,
                           cd.deposit_date, cd.deposit_slip_number, cd.deposited_by_user_id,
                           cd.bank_transaction_id, cd.status, cd.notes,
                           cd.created_at, cd.updated_at
                    FROM cash_deposits cd
                    JOIN cashier_shifts cs ON cs.id = cd.cashier_shift_id
                    WHERE cs.store_id = $1 AND cd.status = $2
                    ORDER BY cd.created_at DESC
                    "#,
                )
                .bind(s)
                .bind(st.to_string())
                .fetch_all(&self.pool)
                .await?
            }
            (Some(s), None) => {
                sqlx::query_as::<_, DepositRow>(
                    r#"
                    SELECT cd.id, cd.cashier_shift_id, cd.bank_account_id, cd.amount,
                           cd.deposit_date, cd.deposit_slip_number, cd.deposited_by_user_id,
                           cd.bank_transaction_id, cd.status, cd.notes,
                           cd.created_at, cd.updated_at
                    FROM cash_deposits cd
                    JOIN cashier_shifts cs ON cs.id = cd.cashier_shift_id
                    WHERE cs.store_id = $1
                    ORDER BY cd.created_at DESC
                    "#,
                )
                .bind(s)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(st)) => {
                sqlx::query_as::<_, DepositRow>(
                    r#"
                    SELECT id, cashier_shift_id, bank_account_id, amount,
                           deposit_date, deposit_slip_number, deposited_by_user_id,
                           bank_transaction_id, status, notes, created_at, updated_at
                    FROM cash_deposits
                    WHERE status = $1
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(st.to_string())
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, DepositRow>(
                    r#"
                    SELECT id, cashier_shift_id, bank_account_id, amount,
                           deposit_date, deposit_slip_number, deposited_by_user_id,
                           bank_transaction_id, status, notes, created_at, updated_at
                    FROM cash_deposits
                    ORDER BY created_at DESC
                    "#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        rows.into_iter().map(CashDeposit::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, cashier_shift_id, bank_account_id, amount,
       deposit_date, deposit_slip_number, deposited_by_user_id,
       bank_transaction_id, status, notes, created_at, updated_at
FROM cash_deposits
WHERE id = $1
"#;

const SELECT_BY_SHIFT: &str = r#"
SELECT id, cashier_shift_id, bank_account_id, amount,
       deposit_date, deposit_slip_number, deposited_by_user_id,
       bank_transaction_id, status, notes, created_at, updated_at
FROM cash_deposits
WHERE cashier_shift_id = $1
"#;

#[derive(sqlx::FromRow)]
struct DepositRow {
    id: Uuid,
    cashier_shift_id: Uuid,
    bank_account_id: Uuid,
    amount: Decimal,
    deposit_date: NaiveDate,
    deposit_slip_number: Option<String>,
    deposited_by_user_id: Option<Uuid>,
    bank_transaction_id: Option<Uuid>,
    status: String,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<DepositRow> for CashDeposit {
    type Error = CashManagementError;

    fn try_from(row: DepositRow) -> Result<Self, Self::Error> {
        let s = CashDepositStatus::from_str(&row.status)?;
        Ok(CashDeposit::reconstitute(
            CashDepositId::from_uuid(row.id),
            row.cashier_shift_id,
            BankAccountId::from_uuid(row.bank_account_id),
            row.amount,
            row.deposit_date,
            row.deposit_slip_number,
            row.deposited_by_user_id,
            row.bank_transaction_id.map(BankTransactionId::from_uuid),
            s,
            row.notes,
            row.created_at,
            row.updated_at,
        ))
    }
}
