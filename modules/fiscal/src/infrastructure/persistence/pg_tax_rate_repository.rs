//! PostgreSQL TaxRateRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::FiscalError;
use crate::domain::entities::TaxRate;
use crate::domain::repositories::TaxRateRepository;
use crate::domain::value_objects::{TaxAppliesTo, TaxRateId, TaxType};
use identity::StoreId;

/// PostgreSQL implementation of TaxRateRepository
pub struct PgTaxRateRepository {
    pool: PgPool,
}

impl PgTaxRateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaxRateRepository for PgTaxRateRepository {
    async fn save(&self, tax_rate: &TaxRate) -> Result<(), FiscalError> {
        sqlx::query(
            r#"
            INSERT INTO tax_rates (
                id, store_id, name, tax_type, rate, is_default, is_active,
                applies_to, category_ids, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(tax_rate.id().into_uuid())
        .bind(tax_rate.store_id().into_uuid())
        .bind(tax_rate.name())
        .bind(tax_rate.tax_type().to_string())
        .bind(tax_rate.rate())
        .bind(tax_rate.is_default())
        .bind(tax_rate.is_active())
        .bind(tax_rate.applies_to().to_string())
        .bind(tax_rate.category_ids())
        .bind(tax_rate.created_at())
        .bind(tax_rate.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: TaxRateId) -> Result<Option<TaxRate>, FiscalError> {
        let row = sqlx::query_as::<_, TaxRateRow>(
            r#"
            SELECT id, store_id, name, tax_type, rate, is_default, is_active,
                   applies_to, category_ids, created_at, updated_at
            FROM tax_rates
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<TaxRate>, FiscalError> {
        let rows = sqlx::query_as::<_, TaxRateRow>(
            r#"
            SELECT id, store_id, name, tax_type, rate, is_default, is_active,
                   applies_to, category_ids, created_at, updated_at
            FROM tax_rates
            WHERE store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_default(&self, store_id: StoreId) -> Result<Option<TaxRate>, FiscalError> {
        let row = sqlx::query_as::<_, TaxRateRow>(
            r#"
            SELECT id, store_id, name, tax_type, rate, is_default, is_active,
                   applies_to, category_ids, created_at, updated_at
            FROM tax_rates
            WHERE store_id = $1 AND is_default = true
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_active_by_store(&self, store_id: StoreId) -> Result<Vec<TaxRate>, FiscalError> {
        let rows = sqlx::query_as::<_, TaxRateRow>(
            r#"
            SELECT id, store_id, name, tax_type, rate, is_default, is_active,
                   applies_to, category_ids, created_at, updated_at
            FROM tax_rates
            WHERE store_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, tax_rate: &TaxRate) -> Result<(), FiscalError> {
        let result = sqlx::query(
            r#"
            UPDATE tax_rates
            SET name = $2, tax_type = $3, rate = $4, is_default = $5, is_active = $6,
                applies_to = $7, category_ids = $8, updated_at = $9
            WHERE id = $1
            "#,
        )
        .bind(tax_rate.id().into_uuid())
        .bind(tax_rate.name())
        .bind(tax_rate.tax_type().to_string())
        .bind(tax_rate.rate())
        .bind(tax_rate.is_default())
        .bind(tax_rate.is_active())
        .bind(tax_rate.applies_to().to_string())
        .bind(tax_rate.category_ids())
        .bind(tax_rate.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(FiscalError::TaxRateNotFound(tax_rate.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete(&self, id: TaxRateId) -> Result<(), FiscalError> {
        let result = sqlx::query("DELETE FROM tax_rates WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(FiscalError::TaxRateNotFound(id.into_uuid()));
        }

        Ok(())
    }
}

// =============================================================================
// Row types
// =============================================================================

#[derive(sqlx::FromRow)]
struct TaxRateRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    name: String,
    tax_type: String,
    rate: rust_decimal::Decimal,
    is_default: bool,
    is_active: bool,
    applies_to: String,
    category_ids: Vec<uuid::Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<TaxRateRow> for TaxRate {
    type Error = FiscalError;

    fn try_from(row: TaxRateRow) -> Result<Self, Self::Error> {
        let tax_type: TaxType = row.tax_type.parse().unwrap_or(TaxType::Isv15);
        let applies_to: TaxAppliesTo = row.applies_to.parse().unwrap_or(TaxAppliesTo::All);

        Ok(TaxRate::reconstitute(
            TaxRateId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.name,
            tax_type,
            row.rate,
            row.is_default,
            row.is_active,
            applies_to,
            row.category_ids,
            row.created_at,
            row.updated_at,
        ))
    }
}
