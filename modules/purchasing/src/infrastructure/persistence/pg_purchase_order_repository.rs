// PostgreSQL PurchaseOrderRepository implementation

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::{PurchaseOrder, PurchaseOrderItem};
use crate::domain::repositories::{PurchaseOrderFilter, PurchaseOrderRepository};
use crate::domain::value_objects::{
    PurchaseOrderId, PurchaseOrderItemId, PurchaseOrderStatus, VendorId,
};
use crate::PurchasingError;
use identity::{StoreId, UserId};
use inventory::{Currency, ProductId, UnitOfMeasure, VariantId};

/// PostgreSQL implementation of PurchaseOrderRepository
pub struct PgPurchaseOrderRepository {
    pool: PgPool,
}

impl PgPurchaseOrderRepository {
    /// Creates a new PgPurchaseOrderRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseOrderRepository for PgPurchaseOrderRepository {
    async fn save(&self, order: &PurchaseOrder) -> Result<(), PurchasingError> {
        let mut tx = self.pool.begin().await?;

        // Save the order header
        sqlx::query(
            r#"
            INSERT INTO purchase_orders (
                id, order_number, store_id, vendor_id, status, order_date,
                expected_delivery_date, received_date, subtotal, tax_amount,
                discount_amount, total, currency, payment_terms_days, notes,
                internal_notes, created_by_id, submitted_by_id, submitted_at,
                approved_by_id, approved_at, received_by_id, cancelled_by_id,
                cancelled_at, cancellation_reason, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                    $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)
            "#,
        )
        .bind(order.id().into_uuid())
        .bind(order.order_number())
        .bind(order.store_id().as_uuid())
        .bind(order.vendor_id().into_uuid())
        .bind(order.status().to_string())
        .bind(order.order_date())
        .bind(order.expected_delivery_date())
        .bind(order.received_date())
        .bind(order.subtotal())
        .bind(order.tax_amount())
        .bind(order.discount_amount())
        .bind(order.total())
        .bind(order.currency().as_str())
        .bind(order.payment_terms_days())
        .bind(order.notes())
        .bind(order.internal_notes())
        .bind(order.created_by_id().into_uuid())
        .bind(order.submitted_by_id().map(|id| id.into_uuid()))
        .bind(order.submitted_at())
        .bind(order.approved_by_id().map(|id| id.into_uuid()))
        .bind(order.approved_at())
        .bind(order.received_by_id().map(|id| id.into_uuid()))
        .bind(order.cancelled_by_id().map(|id| id.into_uuid()))
        .bind(order.cancelled_at())
        .bind(order.cancellation_reason())
        .bind(order.created_at())
        .bind(order.updated_at())
        .execute(&mut *tx)
        .await?;

        // Save all items
        for item in order.items() {
            self.save_item_internal(&mut tx, item).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: PurchaseOrderId,
    ) -> Result<Option<PurchaseOrder>, PurchasingError> {
        let row = sqlx::query_as::<_, PurchaseOrderRow>(
            r#"
            SELECT id, order_number, store_id, vendor_id, status, order_date,
                   expected_delivery_date, received_date, subtotal, tax_amount,
                   discount_amount, total, currency, payment_terms_days, notes,
                   internal_notes, created_by_id, submitted_by_id, submitted_at,
                   approved_by_id, approved_at, received_by_id, cancelled_by_id,
                   cancelled_at, cancellation_reason, created_at, updated_at
            FROM purchase_orders
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
        id: PurchaseOrderId,
    ) -> Result<Option<PurchaseOrder>, PurchasingError> {
        let row = sqlx::query_as::<_, PurchaseOrderRow>(
            r#"
            SELECT id, order_number, store_id, vendor_id, status, order_date,
                   expected_delivery_date, received_date, subtotal, tax_amount,
                   discount_amount, total, currency, payment_terms_days, notes,
                   internal_notes, created_by_id, submitted_by_id, submitted_at,
                   approved_by_id, approved_at, received_by_id, cancelled_by_id,
                   cancelled_at, cancellation_reason, created_at, updated_at
            FROM purchase_orders
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(order_row) => {
                let item_rows = sqlx::query_as::<_, PurchaseOrderItemRow>(
                    r#"
                    SELECT id, purchase_order_id, line_number, product_id, variant_id,
                           description, quantity_ordered, quantity_received, unit_of_measure,
                           unit_cost, discount_percent, tax_percent, line_total, notes
                    FROM purchase_order_items
                    WHERE purchase_order_id = $1
                    ORDER BY line_number
                    "#,
                )
                .bind(id.into_uuid())
                .fetch_all(&self.pool)
                .await?;

                let items: Result<Vec<PurchaseOrderItem>, PurchasingError> =
                    item_rows.into_iter().map(|r| r.try_into()).collect();

                Ok(Some(order_row.try_into_with_items(items?)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_order_number(
        &self,
        store_id: StoreId,
        order_number: &str,
    ) -> Result<Option<PurchaseOrder>, PurchasingError> {
        let row = sqlx::query_as::<_, PurchaseOrderRow>(
            r#"
            SELECT id, order_number, store_id, vendor_id, status, order_date,
                   expected_delivery_date, received_date, subtotal, tax_amount,
                   discount_amount, total, currency, payment_terms_days, notes,
                   internal_notes, created_by_id, submitted_by_id, submitted_at,
                   approved_by_id, approved_at, received_by_id, cancelled_by_id,
                   cancelled_at, cancellation_reason, created_at, updated_at
            FROM purchase_orders
            WHERE store_id = $1 AND order_number = $2
            "#,
        )
        .bind(store_id.as_uuid())
        .bind(order_number)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into_without_items()).transpose()
    }

    async fn update(&self, order: &PurchaseOrder) -> Result<(), PurchasingError> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            r#"
            UPDATE purchase_orders
            SET vendor_id = $2, status = $3, order_date = $4, expected_delivery_date = $5,
                received_date = $6, subtotal = $7, tax_amount = $8, discount_amount = $9,
                total = $10, currency = $11, payment_terms_days = $12, notes = $13,
                internal_notes = $14, submitted_by_id = $15, submitted_at = $16,
                approved_by_id = $17, approved_at = $18, received_by_id = $19,
                cancelled_by_id = $20, cancelled_at = $21, cancellation_reason = $22,
                updated_at = $23
            WHERE id = $1
            "#,
        )
        .bind(order.id().into_uuid())
        .bind(order.vendor_id().into_uuid())
        .bind(order.status().to_string())
        .bind(order.order_date())
        .bind(order.expected_delivery_date())
        .bind(order.received_date())
        .bind(order.subtotal())
        .bind(order.tax_amount())
        .bind(order.discount_amount())
        .bind(order.total())
        .bind(order.currency().as_str())
        .bind(order.payment_terms_days())
        .bind(order.notes())
        .bind(order.internal_notes())
        .bind(order.submitted_by_id().map(|id| id.into_uuid()))
        .bind(order.submitted_at())
        .bind(order.approved_by_id().map(|id| id.into_uuid()))
        .bind(order.approved_at())
        .bind(order.received_by_id().map(|id| id.into_uuid()))
        .bind(order.cancelled_by_id().map(|id| id.into_uuid()))
        .bind(order.cancelled_at())
        .bind(order.cancellation_reason())
        .bind(order.updated_at())
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PurchasingError::PurchaseOrderNotFound(
                order.id().into_uuid(),
            ));
        }

        // Update items (delete and re-insert)
        sqlx::query("DELETE FROM purchase_order_items WHERE purchase_order_id = $1")
            .bind(order.id().into_uuid())
            .execute(&mut *tx)
            .await?;

        for item in order.items() {
            self.save_item_internal(&mut tx, item).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: PurchaseOrderFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<PurchaseOrder>, i64), PurchasingError> {
        let offset = (page - 1) * page_size;

        // Build count query
        let mut count_query = String::from("SELECT COUNT(*) FROM purchase_orders WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.vendor_id.is_some() {
            count_query.push_str(&format!(" AND vendor_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            count_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            count_query.push_str(&format!(" AND order_number ILIKE ${}", param_idx));
        }

        // Execute count
        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(store_id) = &filter.store_id {
            count_builder = count_builder.bind(store_id.as_uuid());
        }
        if let Some(vendor_id) = &filter.vendor_id {
            count_builder = count_builder.bind(vendor_id.into_uuid());
        }
        if let Some(status) = &filter.status {
            count_builder = count_builder.bind(status.to_string());
        }
        if let Some(ref search) = filter.search {
            count_builder = count_builder.bind(format!("%{}%", search));
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        // Build data query
        let mut data_query = String::from(
            r#"SELECT id, order_number, store_id, vendor_id, status, order_date,
                   expected_delivery_date, received_date, subtotal, tax_amount,
                   discount_amount, total, currency, payment_terms_days, notes,
                   internal_notes, created_by_id, submitted_by_id, submitted_at,
                   approved_by_id, approved_at, received_by_id, cancelled_by_id,
                   cancelled_at, cancellation_reason, created_at, updated_at
            FROM purchase_orders
            WHERE 1=1"#,
        );

        param_idx = 1;
        if filter.store_id.is_some() {
            data_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.vendor_id.is_some() {
            data_query.push_str(&format!(" AND vendor_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            data_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            data_query.push_str(&format!(" AND order_number ILIKE ${}", param_idx));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        // Execute data query
        let mut data_builder = sqlx::query_as::<_, PurchaseOrderRow>(&data_query);
        if let Some(store_id) = &filter.store_id {
            data_builder = data_builder.bind(store_id.as_uuid());
        }
        if let Some(vendor_id) = &filter.vendor_id {
            data_builder = data_builder.bind(vendor_id.into_uuid());
        }
        if let Some(status) = &filter.status {
            data_builder = data_builder.bind(status.to_string());
        }
        if let Some(ref search) = filter.search {
            data_builder = data_builder.bind(format!("%{}%", search));
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;

        let orders: Result<Vec<PurchaseOrder>, PurchasingError> = rows
            .into_iter()
            .map(|r| r.try_into_without_items())
            .collect();

        Ok((orders?, total_count))
    }

    async fn generate_order_number(&self, store_id: StoreId) -> Result<String, PurchasingError> {
        let year = chrono::Utc::now().format("%Y");
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM purchase_orders
            WHERE store_id = $1 AND EXTRACT(YEAR FROM created_at) = EXTRACT(YEAR FROM NOW())
            "#,
        )
        .bind(store_id.as_uuid())
        .fetch_one(&self.pool)
        .await?;

        let sequence = count.0 + 1;
        Ok(format!("PO-{}-{:05}", year, sequence))
    }

    // -------------------------------------------------------------------------
    // Item operations
    // -------------------------------------------------------------------------

    async fn save_item(&self, item: &PurchaseOrderItem) -> Result<(), PurchasingError> {
        sqlx::query(
            r#"
            INSERT INTO purchase_order_items (
                id, purchase_order_id, line_number, product_id, variant_id,
                description, quantity_ordered, quantity_received, unit_of_measure,
                unit_cost, discount_percent, tax_percent, line_total, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.purchase_order_id().into_uuid())
        .bind(item.line_number())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.description())
        .bind(item.quantity_ordered())
        .bind(item.quantity_received())
        .bind(item.unit_of_measure().to_string())
        .bind(item.unit_cost())
        .bind(item.discount_percent())
        .bind(item.tax_percent())
        .bind(item.line_total())
        .bind(item.notes())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_item(&self, item: &PurchaseOrderItem) -> Result<(), PurchasingError> {
        let result = sqlx::query(
            r#"
            UPDATE purchase_order_items
            SET line_number = $2, product_id = $3, variant_id = $4, description = $5,
                quantity_ordered = $6, quantity_received = $7, unit_of_measure = $8,
                unit_cost = $9, discount_percent = $10, tax_percent = $11,
                line_total = $12, notes = $13
            WHERE id = $1
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.line_number())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.description())
        .bind(item.quantity_ordered())
        .bind(item.quantity_received())
        .bind(item.unit_of_measure().to_string())
        .bind(item.unit_cost())
        .bind(item.discount_percent())
        .bind(item.tax_percent())
        .bind(item.line_total())
        .bind(item.notes())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(PurchasingError::PurchaseOrderItemNotFound(
                item.id().into_uuid(),
            ));
        }

        Ok(())
    }

    async fn delete_item(&self, item_id: PurchaseOrderItemId) -> Result<(), PurchasingError> {
        let result = sqlx::query("DELETE FROM purchase_order_items WHERE id = $1")
            .bind(item_id.into_uuid())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(PurchasingError::PurchaseOrderItemNotFound(
                item_id.into_uuid(),
            ));
        }

        Ok(())
    }

    async fn find_items_by_order(
        &self,
        order_id: PurchaseOrderId,
    ) -> Result<Vec<PurchaseOrderItem>, PurchasingError> {
        let rows = sqlx::query_as::<_, PurchaseOrderItemRow>(
            r#"
            SELECT id, purchase_order_id, line_number, product_id, variant_id,
                   description, quantity_ordered, quantity_received, unit_of_measure,
                   unit_cost, discount_percent, tax_percent, line_total, notes
            FROM purchase_order_items
            WHERE purchase_order_id = $1
            ORDER BY line_number
            "#,
        )
        .bind(order_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_item_by_id(
        &self,
        item_id: PurchaseOrderItemId,
    ) -> Result<Option<PurchaseOrderItem>, PurchasingError> {
        let row = sqlx::query_as::<_, PurchaseOrderItemRow>(
            r#"
            SELECT id, purchase_order_id, line_number, product_id, variant_id,
                   description, quantity_ordered, quantity_received, unit_of_measure,
                   unit_cost, discount_percent, tax_percent, line_total, notes
            FROM purchase_order_items
            WHERE id = $1
            "#,
        )
        .bind(item_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }
}

impl PgPurchaseOrderRepository {
    async fn save_item_internal(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item: &PurchaseOrderItem,
    ) -> Result<(), PurchasingError> {
        sqlx::query(
            r#"
            INSERT INTO purchase_order_items (
                id, purchase_order_id, line_number, product_id, variant_id,
                description, quantity_ordered, quantity_received, unit_of_measure,
                unit_cost, discount_percent, tax_percent, line_total, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.purchase_order_id().into_uuid())
        .bind(item.line_number())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.description())
        .bind(item.quantity_ordered())
        .bind(item.quantity_received())
        .bind(item.unit_of_measure().to_string())
        .bind(item.unit_cost())
        .bind(item.discount_percent())
        .bind(item.tax_percent())
        .bind(item.line_total())
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
struct PurchaseOrderRow {
    id: uuid::Uuid,
    order_number: String,
    store_id: uuid::Uuid,
    vendor_id: uuid::Uuid,
    status: String,
    order_date: NaiveDate,
    expected_delivery_date: Option<NaiveDate>,
    received_date: Option<NaiveDate>,
    subtotal: Decimal,
    tax_amount: Decimal,
    discount_amount: Decimal,
    total: Decimal,
    currency: String,
    payment_terms_days: i32,
    notes: Option<String>,
    internal_notes: Option<String>,
    created_by_id: uuid::Uuid,
    submitted_by_id: Option<uuid::Uuid>,
    submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    approved_by_id: Option<uuid::Uuid>,
    approved_at: Option<chrono::DateTime<chrono::Utc>>,
    received_by_id: Option<uuid::Uuid>,
    cancelled_by_id: Option<uuid::Uuid>,
    cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    cancellation_reason: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl PurchaseOrderRow {
    fn try_into_without_items(self) -> Result<PurchaseOrder, PurchasingError> {
        self.try_into_with_items(Vec::new())
    }

    fn try_into_with_items(
        self,
        items: Vec<PurchaseOrderItem>,
    ) -> Result<PurchaseOrder, PurchasingError> {
        let status: PurchaseOrderStatus = self.status.parse()?;
        let currency = Currency::from_string(self.currency);

        Ok(PurchaseOrder::reconstitute(
            PurchaseOrderId::from_uuid(self.id),
            self.order_number,
            StoreId::from_uuid(self.store_id),
            VendorId::from_uuid(self.vendor_id),
            status,
            self.order_date,
            self.expected_delivery_date,
            self.received_date,
            self.subtotal,
            self.tax_amount,
            self.discount_amount,
            self.total,
            currency,
            self.payment_terms_days,
            self.notes,
            self.internal_notes,
            UserId::from_uuid(self.created_by_id),
            self.submitted_by_id.map(UserId::from_uuid),
            self.submitted_at,
            self.approved_by_id.map(UserId::from_uuid),
            self.approved_at,
            self.received_by_id.map(UserId::from_uuid),
            self.cancelled_by_id.map(UserId::from_uuid),
            self.cancelled_at,
            self.cancellation_reason,
            items,
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct PurchaseOrderItemRow {
    id: uuid::Uuid,
    purchase_order_id: uuid::Uuid,
    line_number: i32,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    description: String,
    quantity_ordered: Decimal,
    quantity_received: Decimal,
    unit_of_measure: String,
    unit_cost: Decimal,
    discount_percent: Decimal,
    tax_percent: Decimal,
    line_total: Decimal,
    notes: Option<String>,
}

impl TryFrom<PurchaseOrderItemRow> for PurchaseOrderItem {
    type Error = PurchasingError;

    fn try_from(row: PurchaseOrderItemRow) -> Result<Self, Self::Error> {
        let unit_of_measure: UnitOfMeasure = row
            .unit_of_measure
            .parse()
            .map_err(|_| PurchasingError::InvalidUnitOfMeasure)?;

        Ok(PurchaseOrderItem::reconstitute(
            PurchaseOrderItemId::from_uuid(row.id),
            PurchaseOrderId::from_uuid(row.purchase_order_id),
            row.line_number,
            ProductId::from_uuid(row.product_id),
            row.variant_id.map(VariantId::from_uuid),
            row.description,
            row.quantity_ordered,
            row.quantity_received,
            unit_of_measure,
            row.unit_cost,
            row.discount_percent,
            row.tax_percent,
            row.line_total,
            row.notes,
        ))
    }
}
