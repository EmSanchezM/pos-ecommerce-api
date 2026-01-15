// PostgreSQL AdjustmentRepository implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::{AdjustmentItem, StockAdjustment};
use crate::domain::repositories::AdjustmentRepository;
use crate::domain::value_objects::{
    AdjustmentId, AdjustmentReason, AdjustmentStatus, AdjustmentType, StockId,
};
use crate::InventoryError;
use identity::{StoreId, UserId};

/// PostgreSQL implementation of AdjustmentRepository
pub struct PgAdjustmentRepository {
    pool: PgPool,
}

impl PgAdjustmentRepository {
    /// Creates a new PgAdjustmentRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdjustmentRepository for PgAdjustmentRepository {
    async fn save(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError> {
        // Start a transaction to save adjustment and items together
        let mut tx = self.pool.begin().await?;

        // Save the adjustment header
        sqlx::query(
            r#"
            INSERT INTO stock_adjustments (
                id, store_id, adjustment_number, adjustment_type, adjustment_reason, status,
                created_by_id, approved_by_id, approved_at, applied_at, notes, attachments,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(adjustment.id().into_uuid())
        .bind(adjustment.store_id().as_uuid())
        .bind(adjustment.adjustment_number())
        .bind(adjustment.adjustment_type().to_string())
        .bind(adjustment.adjustment_reason().to_string())
        .bind(adjustment.status().to_string())
        .bind(adjustment.created_by_id().into_uuid())
        .bind(adjustment.approved_by_id().map(|id| id.into_uuid()))
        .bind(adjustment.approved_at())
        .bind(adjustment.applied_at())
        .bind(adjustment.notes())
        .bind(adjustment.attachments())
        .bind(adjustment.created_at())
        .bind(adjustment.updated_at())
        .execute(&mut *tx)
        .await?;

        // Save all items
        for item in adjustment.items() {
            sqlx::query(
                r#"
                INSERT INTO stock_adjustment_items (
                    id, adjustment_id, stock_id, quantity, unit_cost, balance_before, balance_after, notes, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(item.id())
            .bind(adjustment.id().into_uuid())
            .bind(item.stock_id().into_uuid())
            .bind(item.quantity())
            .bind(item.unit_cost())
            .bind(item.balance_before())
            .bind(item.balance_after())
            .bind(item.notes())
            .bind(item.created_at())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }


    async fn find_by_id(&self, id: AdjustmentId) -> Result<Option<StockAdjustment>, InventoryError> {
        let row = sqlx::query_as::<_, AdjustmentRow>(
            r#"
            SELECT id, store_id, adjustment_number, adjustment_type, adjustment_reason, status,
                   created_by_id, approved_by_id, approved_at, applied_at, notes, attachments,
                   created_at, updated_at
            FROM stock_adjustments
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        // Return without items
        row.map(|r| r.try_into_without_items()).transpose()
    }

    async fn find_by_id_with_items(&self, id: AdjustmentId) -> Result<Option<StockAdjustment>, InventoryError> {
        let row = sqlx::query_as::<_, AdjustmentRow>(
            r#"
            SELECT id, store_id, adjustment_number, adjustment_type, adjustment_reason, status,
                   created_by_id, approved_by_id, approved_at, applied_at, notes, attachments,
                   created_at, updated_at
            FROM stock_adjustments
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(adjustment_row) => {
                // Load items
                let item_rows = sqlx::query_as::<_, AdjustmentItemRow>(
                    r#"
                    SELECT id, adjustment_id, stock_id, quantity, unit_cost, balance_before, balance_after, notes, created_at
                    FROM stock_adjustment_items
                    WHERE adjustment_id = $1
                    ORDER BY created_at
                    "#,
                )
                .bind(id.into_uuid())
                .fetch_all(&self.pool)
                .await?;

                let items: Vec<AdjustmentItem> = item_rows.into_iter().map(|r| r.into()).collect();
                Ok(Some(adjustment_row.try_into_with_items(items)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<StockAdjustment>, InventoryError> {
        let rows = sqlx::query_as::<_, AdjustmentRow>(
            r#"
            SELECT id, store_id, adjustment_number, adjustment_type, adjustment_reason, status,
                   created_by_id, approved_by_id, approved_at, applied_at, notes, attachments,
                   created_at, updated_at
            FROM stock_adjustments
            WHERE store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into_without_items()).collect()
    }

    async fn update(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError> {
        let mut tx = self.pool.begin().await?;

        // Update the adjustment header
        let result = sqlx::query(
            r#"
            UPDATE stock_adjustments
            SET adjustment_type = $2, adjustment_reason = $3, status = $4,
                approved_by_id = $5, approved_at = $6, applied_at = $7,
                notes = $8, attachments = $9, updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(adjustment.id().into_uuid())
        .bind(adjustment.adjustment_type().to_string())
        .bind(adjustment.adjustment_reason().to_string())
        .bind(adjustment.status().to_string())
        .bind(adjustment.approved_by_id().map(|id| id.into_uuid()))
        .bind(adjustment.approved_at())
        .bind(adjustment.applied_at())
        .bind(adjustment.notes())
        .bind(adjustment.attachments())
        .bind(adjustment.updated_at())
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::AdjustmentNotFound(adjustment.id().into_uuid()));
        }

        // Update items (delete and re-insert for simplicity)
        sqlx::query(
            r#"
            DELETE FROM stock_adjustment_items WHERE adjustment_id = $1
            "#,
        )
        .bind(adjustment.id().into_uuid())
        .execute(&mut *tx)
        .await?;

        for item in adjustment.items() {
            sqlx::query(
                r#"
                INSERT INTO stock_adjustment_items (
                    id, adjustment_id, stock_id, quantity, unit_cost, balance_before, balance_after, notes, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(item.id())
            .bind(adjustment.id().into_uuid())
            .bind(item.stock_id().into_uuid())
            .bind(item.quantity())
            .bind(item.unit_cost())
            .bind(item.balance_before())
            .bind(item.balance_after())
            .bind(item.notes())
            .bind(item.created_at())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn generate_adjustment_number(&self, store_id: StoreId) -> Result<String, InventoryError> {
        // Generate unique adjustment number per store: ADJ-{STORE_SHORT_ID}-{SEQUENCE}
        // Use a sequence based on count of adjustments for this store
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM stock_adjustments WHERE store_id = $1
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let sequence = count.0 + 1;
        let store_short = &store_id.as_uuid().to_string()[..8];
        Ok(format!("ADJ-{}-{:06}", store_short.to_uppercase(), sequence))
    }
}


// =============================================================================
// Row types for database mapping
// =============================================================================

/// Internal row type for mapping adjustment database results
#[derive(sqlx::FromRow)]
struct AdjustmentRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    adjustment_number: String,
    adjustment_type: String,
    adjustment_reason: String,
    status: String,
    created_by_id: uuid::Uuid,
    approved_by_id: Option<uuid::Uuid>,
    approved_at: Option<chrono::DateTime<chrono::Utc>>,
    applied_at: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    attachments: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl AdjustmentRow {
    fn try_into_without_items(self) -> Result<StockAdjustment, InventoryError> {
        self.try_into_with_items(Vec::new())
    }

    fn try_into_with_items(self, items: Vec<AdjustmentItem>) -> Result<StockAdjustment, InventoryError> {
        let adjustment_type: AdjustmentType = self.adjustment_type.parse()?;
        let adjustment_reason: AdjustmentReason = self.adjustment_reason.parse()?;
        let status: AdjustmentStatus = self.status.parse()?;

        Ok(StockAdjustment::reconstitute(
            AdjustmentId::from_uuid(self.id),
            StoreId::from_uuid(self.store_id),
            self.adjustment_number,
            adjustment_type,
            adjustment_reason,
            status,
            UserId::from_uuid(self.created_by_id),
            self.approved_by_id.map(UserId::from_uuid),
            self.approved_at,
            self.applied_at,
            self.notes,
            self.attachments,
            items,
            self.created_at,
            self.updated_at,
        ))
    }
}

/// Internal row type for mapping adjustment item database results
#[derive(sqlx::FromRow)]
struct AdjustmentItemRow {
    id: uuid::Uuid,
    adjustment_id: uuid::Uuid,
    stock_id: uuid::Uuid,
    quantity: Decimal,
    unit_cost: Option<Decimal>,
    balance_before: Option<Decimal>,
    balance_after: Option<Decimal>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AdjustmentItemRow> for AdjustmentItem {
    fn from(row: AdjustmentItemRow) -> Self {
        AdjustmentItem::reconstitute(
            row.id,
            AdjustmentId::from_uuid(row.adjustment_id),
            StockId::from_uuid(row.stock_id),
            row.quantity,
            row.unit_cost,
            row.balance_before,
            row.balance_after,
            row.notes,
            row.created_at,
        )
    }
}
