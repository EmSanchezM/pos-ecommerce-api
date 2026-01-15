// PostgreSQL InventoryMovementRepository implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::InventoryMovement;
use crate::domain::repositories::InventoryMovementRepository;
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
        // Order by created_at DESC for Kardex reporting (most recent first)
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
