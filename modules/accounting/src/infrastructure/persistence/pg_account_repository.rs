use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AccountingError;
use crate::domain::entities::Account;
use crate::domain::repositories::AccountRepository;
use crate::domain::value_objects::{AccountId, AccountType};

pub struct PgAccountRepository {
    pool: PgPool,
}

impl PgAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PgAccountRepository {
    async fn save(&self, a: &Account) -> Result<(), AccountingError> {
        sqlx::query(
            r#"
            INSERT INTO chart_of_accounts (
                id, code, name, account_type, parent_id, is_active,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.code())
        .bind(a.name())
        .bind(a.account_type().to_string())
        .bind(a.parent_id().map(|p| p.into_uuid()))
        .bind(a.is_active())
        .bind(a.created_at())
        .bind(a.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, a: &Account) -> Result<(), AccountingError> {
        let result = sqlx::query(
            r#"
            UPDATE chart_of_accounts
            SET name = $2,
                account_type = $3,
                parent_id = $4,
                is_active = $5,
                updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.name())
        .bind(a.account_type().to_string())
        .bind(a.parent_id().map(|p| p.into_uuid()))
        .bind(a.is_active())
        .bind(a.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AccountingError::AccountNotFound(a.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: AccountId) -> Result<Option<Account>, AccountingError> {
        let row = sqlx::query_as::<_, AccountRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Account::try_from).transpose()
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Account>, AccountingError> {
        let row = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT id, code, name, account_type, parent_id, is_active,
                   created_at, updated_at
            FROM chart_of_accounts
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        row.map(Account::try_from).transpose()
    }

    async fn list(&self) -> Result<Vec<Account>, AccountingError> {
        let rows = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT id, code, name, account_type, parent_id, is_active,
                   created_at, updated_at
            FROM chart_of_accounts
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Account::try_from).collect()
    }

    async fn list_by_type(
        &self,
        account_type: AccountType,
    ) -> Result<Vec<Account>, AccountingError> {
        let rows = sqlx::query_as::<_, AccountRow>(
            r#"
            SELECT id, code, name, account_type, parent_id, is_active,
                   created_at, updated_at
            FROM chart_of_accounts
            WHERE account_type = $1
            ORDER BY code ASC
            "#,
        )
        .bind(account_type.to_string())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Account::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, code, name, account_type, parent_id, is_active,
       created_at, updated_at
FROM chart_of_accounts
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    code: String,
    name: String,
    account_type: String,
    parent_id: Option<Uuid>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AccountRow> for Account {
    type Error = AccountingError;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        let account_type: AccountType = row.account_type.parse()?;
        Ok(Account::reconstitute(
            AccountId::from_uuid(row.id),
            row.code,
            row.name,
            account_type,
            row.parent_id.map(AccountId::from_uuid),
            row.is_active,
            row.created_at,
            row.updated_at,
        ))
    }
}
