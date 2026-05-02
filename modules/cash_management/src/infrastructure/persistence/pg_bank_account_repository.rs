use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::CashManagementError;
use crate::domain::entities::BankAccount;
use crate::domain::repositories::BankAccountRepository;
use crate::domain::value_objects::{BankAccountId, BankAccountType};

pub struct PgBankAccountRepository {
    pool: PgPool,
}

impl PgBankAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BankAccountRepository for PgBankAccountRepository {
    async fn save(&self, a: &BankAccount) -> Result<(), CashManagementError> {
        sqlx::query(
            r#"
            INSERT INTO bank_accounts (
                id, store_id, bank_name, account_number, account_type,
                currency, current_balance, is_active, version,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.store_id())
        .bind(a.bank_name())
        .bind(a.account_number())
        .bind(a.account_type().to_string())
        .bind(a.currency())
        .bind(a.current_balance())
        .bind(a.is_active())
        .bind(a.version())
        .bind(a.created_at())
        .bind(a.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, a: &BankAccount) -> Result<(), CashManagementError> {
        let result = sqlx::query(
            r#"
            UPDATE bank_accounts
            SET bank_name        = $2,
                account_type     = $3,
                current_balance  = $4,
                is_active        = $5,
                version          = version + 1,
                updated_at       = $6
            WHERE id = $1 AND version = $7
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.bank_name())
        .bind(a.account_type().to_string())
        .bind(a.current_balance())
        .bind(a.is_active())
        .bind(a.updated_at())
        .bind(a.version())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let exists: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM bank_accounts WHERE id = $1")
                    .bind(a.id().into_uuid())
                    .fetch_optional(&self.pool)
                    .await?;
            if exists.is_none() {
                return Err(CashManagementError::BankAccountNotFound(a.id().into_uuid()));
            }
            return Err(CashManagementError::AccountVersionConflict(
                a.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: BankAccountId,
    ) -> Result<Option<BankAccount>, CashManagementError> {
        let row = sqlx::query_as::<_, AccountRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(BankAccount::try_from).transpose()
    }

    async fn find_by_account_number(
        &self,
        account_number: &str,
    ) -> Result<Option<BankAccount>, CashManagementError> {
        let row = sqlx::query_as::<_, AccountRow>(SELECT_BY_ACCOUNT_NUMBER)
            .bind(account_number)
            .fetch_optional(&self.pool)
            .await?;
        row.map(BankAccount::try_from).transpose()
    }

    async fn list(&self, store_id: Option<Uuid>) -> Result<Vec<BankAccount>, CashManagementError> {
        let rows = match store_id {
            Some(s) => {
                sqlx::query_as::<_, AccountRow>(LIST_BY_STORE)
                    .bind(s)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as::<_, AccountRow>(LIST_ALL)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
        rows.into_iter().map(BankAccount::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, bank_name, account_number, account_type,
       currency, current_balance, is_active, version, created_at, updated_at
FROM bank_accounts
WHERE id = $1
"#;

const SELECT_BY_ACCOUNT_NUMBER: &str = r#"
SELECT id, store_id, bank_name, account_number, account_type,
       currency, current_balance, is_active, version, created_at, updated_at
FROM bank_accounts
WHERE account_number = $1
"#;

const LIST_BY_STORE: &str = r#"
SELECT id, store_id, bank_name, account_number, account_type,
       currency, current_balance, is_active, version, created_at, updated_at
FROM bank_accounts
WHERE store_id = $1
ORDER BY bank_name ASC, account_number ASC
"#;

const LIST_ALL: &str = r#"
SELECT id, store_id, bank_name, account_number, account_type,
       currency, current_balance, is_active, version, created_at, updated_at
FROM bank_accounts
ORDER BY bank_name ASC, account_number ASC
"#;

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    store_id: Uuid,
    bank_name: String,
    account_number: String,
    account_type: String,
    currency: String,
    current_balance: Decimal,
    is_active: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AccountRow> for BankAccount {
    type Error = CashManagementError;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        let t = BankAccountType::from_str(&row.account_type)?;
        Ok(BankAccount::reconstitute(
            BankAccountId::from_uuid(row.id),
            row.store_id,
            row.bank_name,
            row.account_number,
            t,
            row.currency,
            row.current_balance,
            row.is_active,
            row.version,
            row.created_at,
            row.updated_at,
        ))
    }
}
