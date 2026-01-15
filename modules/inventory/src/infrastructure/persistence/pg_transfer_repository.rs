// PostgreSQL TransferRepository implementation

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::{StockTransfer, TransferItem};
use crate::domain::repositories::TransferRepository;
use crate::domain::value_objects::{ProductId, TransferId, TransferStatus, VariantId};
use crate::InventoryError;
use identity::{StoreId, UserId};

/// PostgreSQL implementation of TransferRepository
pub struct PgTransferRepository {
    pool: PgPool,
}

impl PgTransferRepository {
    /// Creates a new PgTransferRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransferRepository for PgTransferRepository {
    async fn save(&self, transfer: &StockTransfer) -> Result<(), InventoryError> {
        // Start a transaction to save transfer and items together
        let mut tx = self.pool.begin().await?;

        // Save the transfer header
        sqlx::query(
            r#"
            INSERT INTO stock_transfers (
                id, transfer_number, from_store_id, to_store_id, status, requested_date,
                shipped_date, received_date, requested_by_id, shipped_by_id, received_by_id,
                notes, shipping_method, tracking_number, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(transfer.id().into_uuid())
        .bind(transfer.transfer_number())
        .bind(transfer.from_store_id().as_uuid())
        .bind(transfer.to_store_id().as_uuid())
        .bind(transfer.status().to_string())
        .bind(transfer.requested_date())
        .bind(transfer.shipped_date())
        .bind(transfer.received_date())
        .bind(transfer.requested_by_id().into_uuid())
        .bind(transfer.shipped_by_id().map(|id| id.into_uuid()))
        .bind(transfer.received_by_id().map(|id| id.into_uuid()))
        .bind(transfer.notes())
        .bind(transfer.shipping_method())
        .bind(transfer.tracking_number())
        .bind(transfer.created_at())
        .bind(transfer.updated_at())
        .execute(&mut *tx)
        .await?;

        // Save all items
        for item in transfer.items() {
            sqlx::query(
                r#"
                INSERT INTO stock_transfer_items (
                    id, transfer_id, product_id, variant_id, quantity_requested,
                    quantity_shipped, quantity_received, unit_cost, notes, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(item.id())
            .bind(transfer.id().into_uuid())
            .bind(item.product_id().map(|id| id.into_uuid()))
            .bind(item.variant_id().map(|id| id.into_uuid()))
            .bind(item.quantity_requested())
            .bind(item.quantity_shipped())
            .bind(item.quantity_received())
            .bind(item.unit_cost())
            .bind(item.notes())
            .bind(item.created_at())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }


    async fn find_by_id(&self, id: TransferId) -> Result<Option<StockTransfer>, InventoryError> {
        let row = sqlx::query_as::<_, TransferRow>(
            r#"
            SELECT id, transfer_number, from_store_id, to_store_id, status, requested_date,
                   shipped_date, received_date, requested_by_id, shipped_by_id, received_by_id,
                   notes, shipping_method, tracking_number, created_at, updated_at
            FROM stock_transfers
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        // Return without items
        row.map(|r| r.try_into_without_items()).transpose()
    }

    async fn find_by_id_with_items(&self, id: TransferId) -> Result<Option<StockTransfer>, InventoryError> {
        let row = sqlx::query_as::<_, TransferRow>(
            r#"
            SELECT id, transfer_number, from_store_id, to_store_id, status, requested_date,
                   shipped_date, received_date, requested_by_id, shipped_by_id, received_by_id,
                   notes, shipping_method, tracking_number, created_at, updated_at
            FROM stock_transfers
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(transfer_row) => {
                // Load items
                let item_rows = sqlx::query_as::<_, TransferItemRow>(
                    r#"
                    SELECT id, transfer_id, product_id, variant_id, quantity_requested,
                           quantity_shipped, quantity_received, unit_cost, notes, created_at
                    FROM stock_transfer_items
                    WHERE transfer_id = $1
                    ORDER BY created_at
                    "#,
                )
                .bind(id.into_uuid())
                .fetch_all(&self.pool)
                .await?;

                let items: Result<Vec<TransferItem>, InventoryError> = 
                    item_rows.into_iter().map(|r| r.try_into()).collect();
                Ok(Some(transfer_row.try_into_with_items(items?)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<StockTransfer>, InventoryError> {
        let rows = sqlx::query_as::<_, TransferRow>(
            r#"
            SELECT id, transfer_number, from_store_id, to_store_id, status, requested_date,
                   shipped_date, received_date, requested_by_id, shipped_by_id, received_by_id,
                   notes, shipping_method, tracking_number, created_at, updated_at
            FROM stock_transfers
            WHERE from_store_id = $1 OR to_store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into_without_items()).collect()
    }

    async fn find_outgoing_by_store(&self, store_id: StoreId) -> Result<Vec<StockTransfer>, InventoryError> {
        let rows = sqlx::query_as::<_, TransferRow>(
            r#"
            SELECT id, transfer_number, from_store_id, to_store_id, status, requested_date,
                   shipped_date, received_date, requested_by_id, shipped_by_id, received_by_id,
                   notes, shipping_method, tracking_number, created_at, updated_at
            FROM stock_transfers
            WHERE from_store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into_without_items()).collect()
    }

    async fn find_incoming_by_store(&self, store_id: StoreId) -> Result<Vec<StockTransfer>, InventoryError> {
        let rows = sqlx::query_as::<_, TransferRow>(
            r#"
            SELECT id, transfer_number, from_store_id, to_store_id, status, requested_date,
                   shipped_date, received_date, requested_by_id, shipped_by_id, received_by_id,
                   notes, shipping_method, tracking_number, created_at, updated_at
            FROM stock_transfers
            WHERE to_store_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into_without_items()).collect()
    }


    async fn update(&self, transfer: &StockTransfer) -> Result<(), InventoryError> {
        let mut tx = self.pool.begin().await?;

        // Update the transfer header
        let result = sqlx::query(
            r#"
            UPDATE stock_transfers
            SET status = $2, shipped_date = $3, received_date = $4,
                shipped_by_id = $5, received_by_id = $6, notes = $7,
                shipping_method = $8, tracking_number = $9, updated_at = $10
            WHERE id = $1
            "#,
        )
        .bind(transfer.id().into_uuid())
        .bind(transfer.status().to_string())
        .bind(transfer.shipped_date())
        .bind(transfer.received_date())
        .bind(transfer.shipped_by_id().map(|id| id.into_uuid()))
        .bind(transfer.received_by_id().map(|id| id.into_uuid()))
        .bind(transfer.notes())
        .bind(transfer.shipping_method())
        .bind(transfer.tracking_number())
        .bind(transfer.updated_at())
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::TransferNotFound(transfer.id().into_uuid()));
        }

        // Update items (delete and re-insert for simplicity)
        sqlx::query(
            r#"
            DELETE FROM stock_transfer_items WHERE transfer_id = $1
            "#,
        )
        .bind(transfer.id().into_uuid())
        .execute(&mut *tx)
        .await?;

        for item in transfer.items() {
            sqlx::query(
                r#"
                INSERT INTO stock_transfer_items (
                    id, transfer_id, product_id, variant_id, quantity_requested,
                    quantity_shipped, quantity_received, unit_cost, notes, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(item.id())
            .bind(transfer.id().into_uuid())
            .bind(item.product_id().map(|id| id.into_uuid()))
            .bind(item.variant_id().map(|id| id.into_uuid()))
            .bind(item.quantity_requested())
            .bind(item.quantity_shipped())
            .bind(item.quantity_received())
            .bind(item.unit_cost())
            .bind(item.notes())
            .bind(item.created_at())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn generate_transfer_number(&self) -> Result<String, InventoryError> {
        // Generate globally unique transfer number: TRF-{YYYYMMDD}-{SEQUENCE}
        let today = Utc::now().format("%Y%m%d").to_string();
        
        // Count transfers created today
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM stock_transfers 
            WHERE transfer_number LIKE $1
            "#,
        )
        .bind(format!("TRF-{}-%%", today))
        .fetch_one(&self.pool)
        .await?;

        let sequence = count.0 + 1;
        Ok(format!("TRF-{}-{:06}", today, sequence))
    }
}

// =============================================================================
// Row types for database mapping
// =============================================================================

/// Internal row type for mapping transfer database results
#[derive(sqlx::FromRow)]
struct TransferRow {
    id: uuid::Uuid,
    transfer_number: String,
    from_store_id: uuid::Uuid,
    to_store_id: uuid::Uuid,
    status: String,
    requested_date: chrono::DateTime<chrono::Utc>,
    shipped_date: Option<chrono::DateTime<chrono::Utc>>,
    received_date: Option<chrono::DateTime<chrono::Utc>>,
    requested_by_id: uuid::Uuid,
    shipped_by_id: Option<uuid::Uuid>,
    received_by_id: Option<uuid::Uuid>,
    notes: Option<String>,
    shipping_method: Option<String>,
    tracking_number: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TransferRow {
    fn try_into_without_items(self) -> Result<StockTransfer, InventoryError> {
        self.try_into_with_items(Vec::new())
    }

    fn try_into_with_items(self, items: Vec<TransferItem>) -> Result<StockTransfer, InventoryError> {
        let status: TransferStatus = self.status.parse()?;

        Ok(StockTransfer::reconstitute(
            TransferId::from_uuid(self.id),
            self.transfer_number,
            StoreId::from_uuid(self.from_store_id),
            StoreId::from_uuid(self.to_store_id),
            status,
            self.requested_date,
            self.shipped_date,
            self.received_date,
            UserId::from_uuid(self.requested_by_id),
            self.shipped_by_id.map(UserId::from_uuid),
            self.received_by_id.map(UserId::from_uuid),
            self.notes,
            self.shipping_method,
            self.tracking_number,
            items,
            self.created_at,
            self.updated_at,
        ))
    }
}

/// Internal row type for mapping transfer item database results
#[derive(sqlx::FromRow)]
struct TransferItemRow {
    id: uuid::Uuid,
    transfer_id: uuid::Uuid,
    product_id: Option<uuid::Uuid>,
    variant_id: Option<uuid::Uuid>,
    quantity_requested: Decimal,
    quantity_shipped: Option<Decimal>,
    quantity_received: Option<Decimal>,
    unit_cost: Option<Decimal>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<TransferItemRow> for TransferItem {
    type Error = InventoryError;

    fn try_from(row: TransferItemRow) -> Result<Self, Self::Error> {
        TransferItem::reconstitute(
            row.id,
            TransferId::from_uuid(row.transfer_id),
            row.product_id.map(ProductId::from_uuid),
            row.variant_id.map(VariantId::from_uuid),
            row.quantity_requested,
            row.quantity_shipped,
            row.quantity_received,
            row.unit_cost,
            row.notes,
            row.created_at,
        )
    }
}
