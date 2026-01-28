// PostgreSQL GoodsReceiptRepository implementation

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::{GoodsReceipt, GoodsReceiptItem};
use crate::domain::repositories::{GoodsReceiptFilter, GoodsReceiptRepository};
use crate::domain::value_objects::{
    GoodsReceiptId, GoodsReceiptItemId, GoodsReceiptStatus, PurchaseOrderId, PurchaseOrderItemId,
};
use crate::PurchasingError;
use identity::{StoreId, UserId};
use inventory::{ProductId, VariantId};

/// PostgreSQL implementation of GoodsReceiptRepository
pub struct PgGoodsReceiptRepository {
    pool: PgPool,
}

impl PgGoodsReceiptRepository {
    /// Creates a new PgGoodsReceiptRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GoodsReceiptRepository for PgGoodsReceiptRepository {
    async fn save(&self, receipt: &GoodsReceipt) -> Result<(), PurchasingError> {
        let mut tx = self.pool.begin().await?;

        // Save the receipt header
        sqlx::query(
            r#"
            INSERT INTO goods_receipts (
                id, receipt_number, purchase_order_id, store_id, receipt_date,
                status, notes, received_by_id, confirmed_by_id, confirmed_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(receipt.id().into_uuid())
        .bind(receipt.receipt_number())
        .bind(receipt.purchase_order_id().into_uuid())
        .bind(receipt.store_id().as_uuid())
        .bind(receipt.receipt_date())
        .bind(receipt.status().to_string())
        .bind(receipt.notes())
        .bind(receipt.received_by_id().into_uuid())
        .bind(receipt.confirmed_by_id().map(|id| id.into_uuid()))
        .bind(receipt.confirmed_at())
        .bind(receipt.created_at())
        .bind(receipt.updated_at())
        .execute(&mut *tx)
        .await?;

        // Save all items
        for item in receipt.items() {
            self.save_item_internal(&mut tx, item).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: GoodsReceiptId,
    ) -> Result<Option<GoodsReceipt>, PurchasingError> {
        let row = sqlx::query_as::<_, GoodsReceiptRow>(
            r#"
            SELECT id, receipt_number, purchase_order_id, store_id, receipt_date,
                   status, notes, received_by_id, confirmed_by_id, confirmed_at,
                   created_at, updated_at
            FROM goods_receipts
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into_without_items()).transpose()
    }

    async fn find_by_id_with_items(
        &self,
        id: GoodsReceiptId,
    ) -> Result<Option<GoodsReceipt>, PurchasingError> {
        let row = sqlx::query_as::<_, GoodsReceiptRow>(
            r#"
            SELECT id, receipt_number, purchase_order_id, store_id, receipt_date,
                   status, notes, received_by_id, confirmed_by_id, confirmed_at,
                   created_at, updated_at
            FROM goods_receipts
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(receipt_row) => {
                let item_rows = sqlx::query_as::<_, GoodsReceiptItemRow>(
                    r#"
                    SELECT id, goods_receipt_id, purchase_order_item_id, product_id,
                           variant_id, quantity_received, unit_cost, lot_number,
                           expiry_date, notes
                    FROM goods_receipt_items
                    WHERE goods_receipt_id = $1
                    "#,
                )
                .bind(id.into_uuid())
                .fetch_all(&self.pool)
                .await?;

                let items: Vec<GoodsReceiptItem> = item_rows.into_iter().map(|r| r.into()).collect();

                Ok(Some(receipt_row.try_into_with_items(items)?))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, receipt: &GoodsReceipt) -> Result<(), PurchasingError> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            r#"
            UPDATE goods_receipts
            SET receipt_date = $2, status = $3, notes = $4, confirmed_by_id = $5,
                confirmed_at = $6, updated_at = $7
            WHERE id = $1
            "#,
        )
        .bind(receipt.id().into_uuid())
        .bind(receipt.receipt_date())
        .bind(receipt.status().to_string())
        .bind(receipt.notes())
        .bind(receipt.confirmed_by_id().map(|id| id.into_uuid()))
        .bind(receipt.confirmed_at())
        .bind(receipt.updated_at())
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PurchasingError::GoodsReceiptNotFound(
                receipt.id().into_uuid(),
            ));
        }

        // Update items (delete and re-insert)
        sqlx::query("DELETE FROM goods_receipt_items WHERE goods_receipt_id = $1")
            .bind(receipt.id().into_uuid())
            .execute(&mut *tx)
            .await?;

        for item in receipt.items() {
            self.save_item_internal(&mut tx, item).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_purchase_order(
        &self,
        order_id: PurchaseOrderId,
    ) -> Result<Vec<GoodsReceipt>, PurchasingError> {
        let rows = sqlx::query_as::<_, GoodsReceiptRow>(
            r#"
            SELECT id, receipt_number, purchase_order_id, store_id, receipt_date,
                   status, notes, received_by_id, confirmed_by_id, confirmed_at,
                   created_at, updated_at
            FROM goods_receipts
            WHERE purchase_order_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(order_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.try_into_without_items())
            .collect()
    }

    async fn find_paginated(
        &self,
        filter: GoodsReceiptFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<GoodsReceipt>, i64), PurchasingError> {
        let offset = (page - 1) * page_size;

        // Build count query
        let mut count_query = String::from("SELECT COUNT(*) FROM goods_receipts WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.purchase_order_id.is_some() {
            count_query.push_str(&format!(" AND purchase_order_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            count_query.push_str(&format!(" AND status = ${}", param_idx));
        }

        // Execute count
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(store_id) = &filter.store_id {
            count_builder = count_builder.bind(store_id.as_uuid());
        }
        if let Some(po_id) = &filter.purchase_order_id {
            count_builder = count_builder.bind(po_id.into_uuid());
        }
        if let Some(status) = &filter.status {
            count_builder = count_builder.bind(status.to_string());
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        // Build data query
        let mut data_query = String::from(
            r#"SELECT id, receipt_number, purchase_order_id, store_id, receipt_date,
                   status, notes, received_by_id, confirmed_by_id, confirmed_at,
                   created_at, updated_at
            FROM goods_receipts
            WHERE 1=1"#,
        );

        param_idx = 1;
        if filter.store_id.is_some() {
            data_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.purchase_order_id.is_some() {
            data_query.push_str(&format!(" AND purchase_order_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            data_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        // Execute data query
        let mut data_builder = sqlx::query_as::<_, GoodsReceiptRow>(&data_query);
        if let Some(store_id) = &filter.store_id {
            data_builder = data_builder.bind(store_id.as_uuid());
        }
        if let Some(po_id) = &filter.purchase_order_id {
            data_builder = data_builder.bind(po_id.into_uuid());
        }
        if let Some(status) = &filter.status {
            data_builder = data_builder.bind(status.to_string());
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;

        let receipts: Result<Vec<GoodsReceipt>, PurchasingError> = rows
            .into_iter()
            .map(|r| r.try_into_without_items())
            .collect();

        Ok((receipts?, total_count))
    }

    async fn generate_receipt_number(&self, store_id: StoreId) -> Result<String, PurchasingError> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM goods_receipts
            WHERE store_id = $1 AND EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let sequence = count.0 + 1;
        Ok(format!("GR-{}-{:05}", year, sequence))
    }
}

impl PgGoodsReceiptRepository {
    async fn save_item_internal(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item: &GoodsReceiptItem,
    ) -> Result<(), PurchasingError> {
        sqlx::query(
            r#"
            INSERT INTO goods_receipt_items (
                id, goods_receipt_id, purchase_order_item_id, product_id,
                variant_id, quantity_received, unit_cost, lot_number,
                expiry_date, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.goods_receipt_id().into_uuid())
        .bind(item.purchase_order_item_id().into_uuid())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.quantity_received())
        .bind(item.unit_cost())
        .bind(item.lot_number())
        .bind(item.expiry_date())
        .bind(item.notes())
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

// =============================================================================
// Row types for database mapping
// =============================================================================

#[derive(sqlx::FromRow)]
struct GoodsReceiptRow {
    id: uuid::Uuid,
    receipt_number: String,
    purchase_order_id: uuid::Uuid,
    store_id: uuid::Uuid,
    receipt_date: NaiveDate,
    status: String,
    notes: Option<String>,
    received_by_id: uuid::Uuid,
    confirmed_by_id: Option<uuid::Uuid>,
    confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl GoodsReceiptRow {
    fn try_into_without_items(self) -> Result<GoodsReceipt, PurchasingError> {
        self.try_into_with_items(Vec::new())
    }

    fn try_into_with_items(
        self,
        items: Vec<GoodsReceiptItem>,
    ) -> Result<GoodsReceipt, PurchasingError> {
        let status: GoodsReceiptStatus = self.status.parse()?;

        Ok(GoodsReceipt::reconstitute(
            GoodsReceiptId::from_uuid(self.id),
            self.receipt_number,
            PurchaseOrderId::from_uuid(self.purchase_order_id),
            StoreId::from_uuid(self.store_id),
            self.receipt_date,
            status,
            self.notes,
            UserId::from_uuid(self.received_by_id),
            self.confirmed_by_id.map(UserId::from_uuid),
            self.confirmed_at,
            items,
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct GoodsReceiptItemRow {
    id: uuid::Uuid,
    goods_receipt_id: uuid::Uuid,
    purchase_order_item_id: uuid::Uuid,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    quantity_received: Decimal,
    unit_cost: Decimal,
    lot_number: Option<String>,
    expiry_date: Option<NaiveDate>,
    notes: Option<String>,
}

impl From<GoodsReceiptItemRow> for GoodsReceiptItem {
    fn from(row: GoodsReceiptItemRow) -> Self {
        GoodsReceiptItem::reconstitute(
            GoodsReceiptItemId::from_uuid(row.id),
            GoodsReceiptId::from_uuid(row.goods_receipt_id),
            PurchaseOrderItemId::from_uuid(row.purchase_order_item_id),
            ProductId::from_uuid(row.product_id),
            row.variant_id.map(VariantId::from_uuid),
            row.quantity_received,
            row.unit_cost,
            row.lot_number,
            row.expiry_date,
            row.notes,
        )
    }
}
