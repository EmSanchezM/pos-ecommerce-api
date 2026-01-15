// PostgreSQL ReservationRepository implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::InventoryReservation;
use crate::domain::repositories::ReservationRepository;
use crate::domain::value_objects::{ReservationId, ReservationStatus, StockId};
use crate::InventoryError;

/// PostgreSQL implementation of ReservationRepository
pub struct PgReservationRepository {
    pool: PgPool,
}

impl PgReservationRepository {
    /// Creates a new PgReservationRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReservationRepository for PgReservationRepository {
    async fn save(&self, reservation: &InventoryReservation) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO inventory_reservations (
                id, stock_id, reference_type, reference_id, quantity, status, expires_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(reservation.id().into_uuid())
        .bind(reservation.stock_id().into_uuid())
        .bind(reservation.reference_type())
        .bind(reservation.reference_id())
        .bind(reservation.quantity())
        .bind(reservation.status().to_string())
        .bind(reservation.expires_at())
        .bind(reservation.created_at())
        .bind(reservation.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: ReservationId) -> Result<Option<InventoryReservation>, InventoryError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            r#"
            SELECT id, stock_id, reference_type, reference_id, quantity, status, expires_at, created_at, updated_at
            FROM inventory_reservations
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }


    async fn find_by_stock_id(&self, stock_id: StockId) -> Result<Vec<InventoryReservation>, InventoryError> {
        let rows = sqlx::query_as::<_, ReservationRow>(
            r#"
            SELECT id, stock_id, reference_type, reference_id, quantity, status, expires_at, created_at, updated_at
            FROM inventory_reservations
            WHERE stock_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(stock_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_by_reference(
        &self,
        reference_type: &str,
        reference_id: Uuid,
    ) -> Result<Vec<InventoryReservation>, InventoryError> {
        let rows = sqlx::query_as::<_, ReservationRow>(
            r#"
            SELECT id, stock_id, reference_type, reference_id, quantity, status, expires_at, created_at, updated_at
            FROM inventory_reservations
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

    async fn find_expired(&self) -> Result<Vec<InventoryReservation>, InventoryError> {
        let rows = sqlx::query_as::<_, ReservationRow>(
            r#"
            SELECT id, stock_id, reference_type, reference_id, quantity, status, expires_at, created_at, updated_at
            FROM inventory_reservations
            WHERE status = 'pending' AND expires_at < NOW()
            ORDER BY expires_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, reservation: &InventoryReservation) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            UPDATE inventory_reservations
            SET status = $2, updated_at = $3
            WHERE id = $1
            "#,
        )
        .bind(reservation.id().into_uuid())
        .bind(reservation.status().to_string())
        .bind(reservation.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::ReservationNotFound(reservation.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete(&self, id: ReservationId) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM inventory_reservations
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::ReservationNotFound(id.into_uuid()));
        }

        Ok(())
    }
}

/// Internal row type for mapping reservation database results
#[derive(sqlx::FromRow)]
struct ReservationRow {
    id: uuid::Uuid,
    stock_id: uuid::Uuid,
    reference_type: String,
    reference_id: uuid::Uuid,
    quantity: Decimal,
    status: String,
    expires_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ReservationRow> for InventoryReservation {
    type Error = InventoryError;

    fn try_from(row: ReservationRow) -> Result<Self, Self::Error> {
        let status: ReservationStatus = row.status.parse()?;
        
        Ok(InventoryReservation::reconstitute(
            ReservationId::from_uuid(row.id),
            StockId::from_uuid(row.stock_id),
            row.reference_type,
            row.reference_id,
            row.quantity,
            status,
            row.expires_at,
            row.created_at,
            row.updated_at,
        ))
    }
}
