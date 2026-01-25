// PostgreSQL InventoryMovementRepository implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::InventoryMovement;
use crate::domain::repositories::{InventoryMovementRepository, MovementQuery};
use crate::domain::value_objects::{Currency, MovementId, MovementType, StockId};
use crate::InventoryError;
use identity::UserId;

/// PostgreSQL implementation of InventoryMovementRepository
pub struct PgInventoryMovementRepository {
    pool: PgPool,
}

impl PgInventoryMovementRepository {
    /// Creates a new PgInventoryMovementRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryMovementRepository for PgInventoryMovementRepository {
    async fn save(&self, movement: &InventoryMovement) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO inventory_movements (
                id, stock_id, movement_type, movement_reason, quantity, unit_cost, currency,
                balance_after, reference_type, reference_id, actor_id, notes, metadata, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(movement.id().into_uuid())
        .bind(movement.stock_id().into_uuid())
        .bind(movement.movement_type().to_string())
        .bind(movement.movement_reason())
        .bind(movement.quantity())
        .bind(movement.unit_cost())
        .bind(movement.currency().as_str())
        .bind(movement.balance_after())
        .bind(movement.reference_type())
        .bind(movement.reference_id())
        .bind(movement.actor_id().into_uuid())
        .bind(movement.notes())
        .bind(movement.metadata())
        .bind(movement.created_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_stock_id(
        &self,
        stock_id: StockId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<InventoryMovement>, InventoryError> {
        // Order by created_at DESC for stock history reporting (most recent first)
        let rows = sqlx::query_as::<_, MovementRow>(
            r#"
            SELECT id, stock_id, movement_type, movement_reason, quantity, unit_cost, currency,
                   balance_after, reference_type, reference_id, actor_id, notes, metadata, created_at
            FROM inventory_movements
            WHERE stock_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(stock_id.into_uuid())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn count_by_stock_id(&self, stock_id: StockId) -> Result<i64, InventoryError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM inventory_movements
            WHERE stock_id = $1
            "#,
        )
        .bind(stock_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_by_stock_id_and_date_range(
        &self,
        stock_id: StockId,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<InventoryMovement>, InventoryError> {
        let rows = sqlx::query_as::<_, MovementRow>(
            r#"
            SELECT id, stock_id, movement_type, movement_reason, quantity, unit_cost, currency,
                   balance_after, reference_type, reference_id, actor_id, notes, metadata, created_at
            FROM inventory_movements
            WHERE stock_id = $1
              AND ($2::timestamptz IS NULL OR created_at >= $2)
              AND ($3::timestamptz IS NULL OR created_at <= $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(stock_id.into_uuid())
        .bind(from_date)
        .bind(to_date)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn count_by_stock_id_and_date_range(
        &self,
        stock_id: StockId,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
    ) -> Result<i64, InventoryError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM inventory_movements
            WHERE stock_id = $1
              AND ($2::timestamptz IS NULL OR created_at >= $2)
              AND ($3::timestamptz IS NULL OR created_at <= $3)
            "#,
        )
        .bind(stock_id.into_uuid())
        .bind(from_date)
        .bind(to_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_by_reference(
        &self,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<InventoryMovement>, InventoryError> {
        let rows = sqlx::query_as::<_, MovementRow>(
            r#"
            SELECT id, stock_id, movement_type, movement_reason, quantity, unit_cost, currency,
                   balance_after, reference_type, reference_id, actor_id, notes, metadata, created_at
            FROM inventory_movements
            WHERE reference_type = $1 AND reference_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(reference_type)
        .bind(reference_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_with_filters(
        &self,
        query: &MovementQuery,
    ) -> Result<Vec<InventoryMovement>, InventoryError> {
        let offset = (query.page - 1) * query.page_size;

        let rows = sqlx::query_as::<_, MovementRow>(
            r#"
            SELECT m.id, m.stock_id, m.movement_type, m.movement_reason, m.quantity, m.unit_cost, m.currency,
                   m.balance_after, m.reference_type, m.reference_id, m.actor_id, m.notes, m.metadata, m.created_at
            FROM inventory_movements m
            INNER JOIN inventory_stock s ON m.stock_id = s.id
            WHERE ($1::uuid IS NULL OR s.store_id = $1)
              AND ($2::uuid IS NULL OR m.stock_id = $2)
              AND ($3::varchar IS NULL OR m.movement_type = $3)
              AND ($4::timestamptz IS NULL OR m.created_at >= $4)
              AND ($5::timestamptz IS NULL OR m.created_at <= $5)
            ORDER BY m.created_at DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(query.store_id)
        .bind(query.stock_id.map(|s| s.into_uuid()))
        .bind(&query.movement_type)
        .bind(query.from_date)
        .bind(query.to_date)
        .bind(query.page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn count_with_filters(&self, query: &MovementQuery) -> Result<i64, InventoryError> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM inventory_movements m
            INNER JOIN inventory_stock s ON m.stock_id = s.id
            WHERE ($1::uuid IS NULL OR s.store_id = $1)
              AND ($2::uuid IS NULL OR m.stock_id = $2)
              AND ($3::varchar IS NULL OR m.movement_type = $3)
              AND ($4::timestamptz IS NULL OR m.created_at >= $4)
              AND ($5::timestamptz IS NULL OR m.created_at <= $5)
            "#,
        )
        .bind(query.store_id)
        .bind(query.stock_id.map(|s| s.into_uuid()))
        .bind(&query.movement_type)
        .bind(query.from_date)
        .bind(query.to_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn calculate_weighted_average_cost(
        &self,
        stock_id: StockId,
    ) -> Result<Option<Decimal>, InventoryError> {
        // Calculate weighted average cost from incoming movements (movement_type = 'in')
        // Formula: sum(quantity * unit_cost) / sum(quantity) for positive quantities with unit_cost
        let result: Option<(Option<Decimal>,)> = sqlx::query_as(
            r#"
            SELECT 
                CASE 
                    WHEN SUM(ABS(quantity)) > 0 THEN 
                        SUM(ABS(quantity) * COALESCE(unit_cost, 0)) / SUM(ABS(quantity))
                    ELSE NULL
                END as weighted_avg_cost
            FROM inventory_movements
            WHERE stock_id = $1 
              AND movement_type = 'in'
              AND unit_cost IS NOT NULL
              AND quantity > 0
            "#,
        )
        .bind(stock_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.and_then(|r| r.0))
    }
}

/// Internal row type for mapping movement database results
#[derive(sqlx::FromRow)]
struct MovementRow {
    id: uuid::Uuid,
    stock_id: uuid::Uuid,
    movement_type: String,
    movement_reason: Option<String>,
    quantity: Decimal,
    unit_cost: Option<Decimal>,
    currency: String,
    balance_after: Decimal,
    reference_type: Option<String>,
    reference_id: Option<uuid::Uuid>,
    actor_id: uuid::Uuid,
    notes: Option<String>,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<MovementRow> for InventoryMovement {
    type Error = InventoryError;

    fn try_from(row: MovementRow) -> Result<Self, Self::Error> {
        let movement_type: MovementType = row.movement_type.parse()?;
        
        Ok(InventoryMovement::reconstitute(
            MovementId::from_uuid(row.id),
            StockId::from_uuid(row.stock_id),
            movement_type,
            row.movement_reason,
            row.quantity,
            row.unit_cost,
            Currency::from_string(row.currency),
            row.balance_after,
            row.reference_type,
            row.reference_id,
            UserId::from_uuid(row.actor_id),
            row.notes,
            row.metadata,
            row.created_at,
        ))
    }
}
