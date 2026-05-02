use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::repositories::{StockSnapshot, StockSnapshotRepository};

pub struct PgStockSnapshotRepository {
    pool: PgPool,
}

impl PgStockSnapshotRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockSnapshotRepository for PgStockSnapshotRepository {
    async fn snapshot(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<StockSnapshot>, DemandPlanningError> {
        // The inventory_stock table follows the convention of storing either
        // (product_id, NULL) for simple products or (NULL, variant_id) for
        // variants — match either by COALESCE.
        let row: Option<(Decimal, Decimal)> = sqlx::query_as(
            r#"
            SELECT quantity, reserved_quantity
            FROM inventory_stock
            WHERE store_id = $1
              AND COALESCE(variant_id, product_id) = $2
            LIMIT 1
            "#,
        )
        .bind(store_id)
        .bind(product_variant_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(q, r)| StockSnapshot {
            current_qty: q,
            reserved_qty: r,
        }))
    }
}
