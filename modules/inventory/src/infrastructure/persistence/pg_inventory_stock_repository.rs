// PostgreSQL InventoryStockRepository implementation with optimistic locking

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::InventoryStock;
use crate::domain::repositories::InventoryStockRepository;
use crate::domain::value_objects::{ProductId, StockId, VariantId};
use crate::InventoryError;
use identity::StoreId;

/// PostgreSQL implementation of InventoryStockRepository
pub struct PgInventoryStockRepository {
    pool: PgPool,
}

impl PgInventoryStockRepository {
    /// Creates a new PgInventoryStockRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryStockRepository for PgInventoryStockRepository {
    async fn save(&self, stock: &InventoryStock) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO inventory_stock (
                id, store_id, product_id, variant_id, quantity, reserved_quantity,
                version, min_stock_level, max_stock_level, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(stock.id().into_uuid())
        .bind(stock.store_id().as_uuid())
        .bind(stock.product_id().map(|id| id.into_uuid()))
        .bind(stock.variant_id().map(|id| id.into_uuid()))
        .bind(stock.quantity())
        .bind(stock.reserved_quantity())
        .bind(stock.version())
        .bind(stock.min_stock_level())
        .bind(stock.max_stock_level())
        .bind(stock.created_at())
        .bind(stock.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: StockId) -> Result<Option<InventoryStock>, InventoryError> {
        let row = sqlx::query_as::<_, StockRow>(
            r#"
            SELECT id, store_id, product_id, variant_id, quantity, reserved_quantity,
                   version, min_stock_level, max_stock_level, created_at, updated_at
            FROM inventory_stock
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }


    async fn find_by_store_and_product(
        &self,
        store_id: StoreId,
        product_id: ProductId,
    ) -> Result<Option<InventoryStock>, InventoryError> {
        let row = sqlx::query_as::<_, StockRow>(
            r#"
            SELECT id, store_id, product_id, variant_id, quantity, reserved_quantity,
                   version, min_stock_level, max_stock_level, created_at, updated_at
            FROM inventory_stock
            WHERE store_id = $1 AND product_id = $2 AND variant_id IS NULL
            "#,
        )
        .bind(store_id.as_uuid())
        .bind(product_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store_and_variant(
        &self,
        store_id: StoreId,
        variant_id: VariantId,
    ) -> Result<Option<InventoryStock>, InventoryError> {
        let row = sqlx::query_as::<_, StockRow>(
            r#"
            SELECT id, store_id, product_id, variant_id, quantity, reserved_quantity,
                   version, min_stock_level, max_stock_level, created_at, updated_at
            FROM inventory_stock
            WHERE store_id = $1 AND variant_id = $2 AND product_id IS NULL
            "#,
        )
        .bind(store_id.as_uuid())
        .bind(variant_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn update_with_version(
        &self,
        stock: &InventoryStock,
        expected_version: i32,
    ) -> Result<(), InventoryError> {
        // Use optimistic locking: only update if version matches
        let result = sqlx::query(
            r#"
            UPDATE inventory_stock
            SET quantity = $2,
                reserved_quantity = $3,
                version = $4,
                min_stock_level = $5,
                max_stock_level = $6,
                updated_at = $7
            WHERE id = $1 AND version = $8
            "#,
        )
        .bind(stock.id().into_uuid())
        .bind(stock.quantity())
        .bind(stock.reserved_quantity())
        .bind(stock.version())
        .bind(stock.min_stock_level())
        .bind(stock.max_stock_level())
        .bind(stock.updated_at())
        .bind(expected_version)
        .execute(&self.pool)
        .await?;

        // If no rows were affected, the version didn't match (concurrent modification)
        if result.rows_affected() == 0 {
            return Err(InventoryError::OptimisticLockError);
        }

        Ok(())
    }

    async fn find_low_stock(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
        let rows = sqlx::query_as::<_, StockRow>(
            r#"
            SELECT id, store_id, product_id, variant_id, quantity, reserved_quantity,
                   version, min_stock_level, max_stock_level, created_at, updated_at
            FROM inventory_stock
            WHERE store_id = $1 AND (quantity - reserved_quantity) <= min_stock_level
            ORDER BY (quantity - reserved_quantity) ASC
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
        let rows = sqlx::query_as::<_, StockRow>(
            r#"
            SELECT id, store_id, product_id, variant_id, quantity, reserved_quantity,
                   version, min_stock_level, max_stock_level, created_at, updated_at
            FROM inventory_stock
            WHERE store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

/// Internal row type for mapping stock database results
#[derive(sqlx::FromRow)]
struct StockRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    product_id: Option<uuid::Uuid>,
    variant_id: Option<uuid::Uuid>,
    quantity: Decimal,
    reserved_quantity: Decimal,
    version: i32,
    min_stock_level: Decimal,
    max_stock_level: Option<Decimal>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<StockRow> for InventoryStock {
    type Error = InventoryError;

    fn try_from(row: StockRow) -> Result<Self, Self::Error> {
        InventoryStock::reconstitute(
            StockId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.product_id.map(ProductId::from_uuid),
            row.variant_id.map(VariantId::from_uuid),
            row.quantity,
            row.reserved_quantity,
            row.version,
            row.min_stock_level,
            row.max_stock_level,
            row.created_at,
            row.updated_at,
        )
    }
}
