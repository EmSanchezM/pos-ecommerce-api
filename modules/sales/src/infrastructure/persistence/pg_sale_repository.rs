//! PostgreSQL SaleRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;
use std::str::FromStr;

use crate::domain::entities::{Payment, Sale, SaleItem};
use crate::domain::repositories::{SaleFilter, SaleRepository};
use crate::domain::value_objects::{
    CustomerId, DiscountType, OrderStatus, PaymentId, PaymentMethod, PaymentStatus, SaleId,
    SaleItemId, SaleStatus, SaleType, ShiftId,
};
use crate::SalesError;
use identity::{StoreId, UserId};
use inventory::{Currency, ProductId, ReservationId, UnitOfMeasure, VariantId};
use pos_core::TerminalId;

/// PostgreSQL implementation of SaleRepository
pub struct PgSaleRepository {
    pool: PgPool,
}

impl PgSaleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn load_items(&self, sale_id: SaleId) -> Result<Vec<SaleItem>, SalesError> {
        let rows = sqlx::query_as::<_, SaleItemRow>(
            r#"
            SELECT id, sale_id, line_number, product_id, variant_id, sku, description,
                   quantity, unit_of_measure, unit_price, unit_cost, discount_type,
                   discount_value, discount_amount, tax_rate, tax_amount, subtotal,
                   total, reservation_id, notes, created_at, updated_at
            FROM sale_items
            WHERE sale_id = $1
            ORDER BY line_number
            "#,
        )
        .bind(sale_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn load_payments(&self, sale_id: SaleId) -> Result<Vec<Payment>, SalesError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT id, sale_id, payment_method, status, amount, currency, amount_tendered,
                   change_given, reference_number, authorization_code, card_last_four,
                   card_brand, refunded_amount, refunded_at, notes, processed_at,
                   created_at, updated_at
            FROM payments
            WHERE sale_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(sale_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[async_trait]
impl SaleRepository for PgSaleRepository {
    async fn save(&self, sale: &Sale) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO sales (
                id, sale_number, store_id, sale_type, status, order_status, terminal_id,
                shift_id, cashier_id, customer_id, currency, subtotal, discount_type,
                discount_value, discount_amount, tax_amount, total, amount_paid, amount_due,
                change_given, invoice_number, invoice_date, notes, internal_notes,
                voided_by_id, voided_at, void_reason, completed_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30)
            "#,
        )
        .bind(sale.id().into_uuid())
        .bind(sale.sale_number())
        .bind(sale.store_id().into_uuid())
        .bind(sale.sale_type().to_string())
        .bind(sale.status().to_string())
        .bind(sale.order_status().map(|s| s.to_string()))
        .bind(sale.terminal_id().map(|t| t.into_uuid()))
        .bind(sale.shift_id().map(|s| s.into_uuid()))
        .bind(sale.cashier_id().map(|c| c.into_uuid()))
        .bind(sale.customer_id().map(|c| c.into_uuid()))
        .bind(sale.currency().as_str())
        .bind(sale.subtotal())
        .bind(sale.discount_type().map(|d| d.to_string()))
        .bind(sale.discount_value())
        .bind(sale.discount_amount())
        .bind(sale.tax_amount())
        .bind(sale.total())
        .bind(sale.amount_paid())
        .bind(sale.amount_due())
        .bind(sale.change_given())
        .bind(sale.invoice_number())
        .bind(sale.invoice_date())
        .bind(sale.notes())
        .bind(sale.internal_notes())
        .bind(sale.voided_by_id().map(|u| u.into_uuid()))
        .bind(sale.voided_at())
        .bind(sale.void_reason())
        .bind(sale.completed_at())
        .bind(sale.created_at())
        .bind(sale.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: SaleId) -> Result<Option<Sale>, SalesError> {
        let row = sqlx::query_as::<_, SaleRow>(
            r#"
            SELECT id, sale_number, store_id, sale_type, status, order_status, terminal_id,
                   shift_id, cashier_id, customer_id, currency, subtotal, discount_type,
                   discount_value, discount_amount, tax_amount, total, amount_paid, amount_due,
                   change_given, invoice_number, invoice_date, notes, internal_notes,
                   voided_by_id, voided_at, void_reason, completed_at, created_at, updated_at
            FROM sales
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_sale(Vec::new(), Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_id_with_details(&self, id: SaleId) -> Result<Option<Sale>, SalesError> {
        let row = sqlx::query_as::<_, SaleRow>(
            r#"
            SELECT id, sale_number, store_id, sale_type, status, order_status, terminal_id,
                   shift_id, cashier_id, customer_id, currency, subtotal, discount_type,
                   discount_value, discount_amount, tax_amount, total, amount_paid, amount_due,
                   change_given, invoice_number, invoice_date, notes, internal_notes,
                   voided_by_id, voided_at, void_reason, completed_at, created_at, updated_at
            FROM sales
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let items = self.load_items(id).await?;
                let payments = self.load_payments(id).await?;
                Ok(Some(r.into_sale(items, payments)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_sale_number(
        &self,
        store_id: StoreId,
        sale_number: &str,
    ) -> Result<Option<Sale>, SalesError> {
        let row = sqlx::query_as::<_, SaleRow>(
            r#"
            SELECT id, sale_number, store_id, sale_type, status, order_status, terminal_id,
                   shift_id, cashier_id, customer_id, currency, subtotal, discount_type,
                   discount_value, discount_amount, tax_amount, total, amount_paid, amount_due,
                   change_given, invoice_number, invoice_date, notes, internal_notes,
                   voided_by_id, voided_at, void_reason, completed_at, created_at, updated_at
            FROM sales
            WHERE store_id = $1 AND sale_number = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(sale_number)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_sale(Vec::new(), Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_invoice_number(
        &self,
        store_id: StoreId,
        invoice_number: &str,
    ) -> Result<Option<Sale>, SalesError> {
        let row = sqlx::query_as::<_, SaleRow>(
            r#"
            SELECT id, sale_number, store_id, sale_type, status, order_status, terminal_id,
                   shift_id, cashier_id, customer_id, currency, subtotal, discount_type,
                   discount_value, discount_amount, tax_amount, total, amount_paid, amount_due,
                   change_given, invoice_number, invoice_date, notes, internal_notes,
                   voided_by_id, voided_at, void_reason, completed_at, created_at, updated_at
            FROM sales
            WHERE store_id = $1 AND invoice_number = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(invoice_number)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_sale(Vec::new(), Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn update(&self, sale: &Sale) -> Result<(), SalesError> {
        let result = sqlx::query(
            r#"
            UPDATE sales
            SET status = $2, order_status = $3, customer_id = $4, subtotal = $5,
                discount_type = $6, discount_value = $7, discount_amount = $8, tax_amount = $9,
                total = $10, amount_paid = $11, amount_due = $12, change_given = $13,
                invoice_number = $14, invoice_date = $15, notes = $16, internal_notes = $17,
                voided_by_id = $18, voided_at = $19, void_reason = $20, completed_at = $21,
                updated_at = $22
            WHERE id = $1
            "#,
        )
        .bind(sale.id().into_uuid())
        .bind(sale.status().to_string())
        .bind(sale.order_status().map(|s| s.to_string()))
        .bind(sale.customer_id().map(|c| c.into_uuid()))
        .bind(sale.subtotal())
        .bind(sale.discount_type().map(|d| d.to_string()))
        .bind(sale.discount_value())
        .bind(sale.discount_amount())
        .bind(sale.tax_amount())
        .bind(sale.total())
        .bind(sale.amount_paid())
        .bind(sale.amount_due())
        .bind(sale.change_given())
        .bind(sale.invoice_number())
        .bind(sale.invoice_date())
        .bind(sale.notes())
        .bind(sale.internal_notes())
        .bind(sale.voided_by_id().map(|u| u.into_uuid()))
        .bind(sale.voided_at())
        .bind(sale.void_reason())
        .bind(sale.completed_at())
        .bind(sale.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SalesError::SaleNotFound(sale.id().into_uuid()));
        }

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: SaleFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Sale>, i64), SalesError> {
        let offset = (page - 1) * page_size;

        let mut count_query = String::from("SELECT COUNT(*) FROM sales WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            count_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.sale_type.is_some() {
            count_query.push_str(&format!(" AND sale_type = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            count_query.push_str(&format!(
                " AND (sale_number ILIKE ${} OR invoice_number ILIKE ${})",
                param_idx, param_idx
            ));
        }

        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(store_id) = filter.store_id {
            count_builder = count_builder.bind(store_id.into_uuid());
        }
        if let Some(status) = filter.status {
            count_builder = count_builder.bind(status.to_string());
        }
        if let Some(sale_type) = filter.sale_type {
            count_builder = count_builder.bind(sale_type.to_string());
        }
        if let Some(ref search) = filter.search {
            count_builder = count_builder.bind(format!("%{}%", search));
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        let mut data_query = String::from(
            r#"SELECT id, sale_number, store_id, sale_type, status, order_status, terminal_id,
                   shift_id, cashier_id, customer_id, currency, subtotal, discount_type,
                   discount_value, discount_amount, tax_amount, total, amount_paid, amount_due,
                   change_given, invoice_number, invoice_date, notes, internal_notes,
                   voided_by_id, voided_at, void_reason, completed_at, created_at, updated_at
            FROM sales WHERE 1=1"#,
        );

        param_idx = 1;
        if filter.store_id.is_some() {
            data_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            data_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.sale_type.is_some() {
            data_query.push_str(&format!(" AND sale_type = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            data_query.push_str(&format!(
                " AND (sale_number ILIKE ${} OR invoice_number ILIKE ${})",
                param_idx, param_idx
            ));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        let mut data_builder = sqlx::query_as::<_, SaleRow>(&data_query);
        if let Some(store_id) = filter.store_id {
            data_builder = data_builder.bind(store_id.into_uuid());
        }
        if let Some(status) = filter.status {
            data_builder = data_builder.bind(status.to_string());
        }
        if let Some(sale_type) = filter.sale_type {
            data_builder = data_builder.bind(sale_type.to_string());
        }
        if let Some(ref search) = filter.search {
            data_builder = data_builder.bind(format!("%{}%", search));
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;
        let sales: Result<Vec<Sale>, SalesError> = rows
            .into_iter()
            .map(|r| r.into_sale(Vec::new(), Vec::new()))
            .collect();

        Ok((sales?, total_count))
    }

    async fn generate_sale_number(&self, store_id: StoreId) -> Result<String, SalesError> {
        let today = chrono::Utc::now().format("%Y%m%d");
        let prefix = format!("SALE-{}", today);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sales WHERE store_id = $1 AND sale_number LIKE $2")
                .bind(store_id.into_uuid())
                .bind(format!("{}%", prefix))
                .fetch_one(&self.pool)
                .await?;

        Ok(format!("{}-{:04}", prefix, count.0 + 1))
    }

    async fn save_item(&self, item: &SaleItem) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO sale_items (
                id, sale_id, line_number, product_id, variant_id, sku, description,
                quantity, unit_of_measure, unit_price, unit_cost, discount_type,
                discount_value, discount_amount, tax_rate, tax_amount, subtotal,
                total, reservation_id, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.sale_id().into_uuid())
        .bind(item.line_number())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.sku())
        .bind(item.description())
        .bind(item.quantity())
        .bind(item.unit_of_measure().to_string())
        .bind(item.unit_price())
        .bind(item.unit_cost())
        .bind(item.discount_type().map(|d| d.to_string()))
        .bind(item.discount_value())
        .bind(item.discount_amount())
        .bind(item.tax_rate())
        .bind(item.tax_amount())
        .bind(item.subtotal())
        .bind(item.total())
        .bind(item.reservation_id().map(|r| r.into_uuid()))
        .bind(item.notes())
        .bind(item.created_at())
        .bind(item.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_item(&self, item: &SaleItem) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            UPDATE sale_items
            SET quantity = $2, unit_price = $3, discount_type = $4, discount_value = $5,
                discount_amount = $6, tax_amount = $7, subtotal = $8, total = $9,
                reservation_id = $10, notes = $11, updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.quantity())
        .bind(item.unit_price())
        .bind(item.discount_type().map(|d| d.to_string()))
        .bind(item.discount_value())
        .bind(item.discount_amount())
        .bind(item.tax_amount())
        .bind(item.subtotal())
        .bind(item.total())
        .bind(item.reservation_id().map(|r| r.into_uuid()))
        .bind(item.notes())
        .bind(item.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_item(&self, item_id: SaleItemId) -> Result<(), SalesError> {
        sqlx::query("DELETE FROM sale_items WHERE id = $1")
            .bind(item_id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_items_by_sale(&self, sale_id: SaleId) -> Result<Vec<SaleItem>, SalesError> {
        self.load_items(sale_id).await
    }

    async fn find_item_by_id(&self, item_id: SaleItemId) -> Result<Option<SaleItem>, SalesError> {
        let row = sqlx::query_as::<_, SaleItemRow>(
            r#"
            SELECT id, sale_id, line_number, product_id, variant_id, sku, description,
                   quantity, unit_of_measure, unit_price, unit_cost, discount_type,
                   discount_value, discount_amount, tax_rate, tax_amount, subtotal,
                   total, reservation_id, notes, created_at, updated_at
            FROM sale_items
            WHERE id = $1
            "#,
        )
        .bind(item_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn save_payment(&self, payment: &Payment) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO payments (
                id, sale_id, payment_method, status, amount, currency, amount_tendered,
                change_given, reference_number, authorization_code, card_last_four,
                card_brand, refunded_amount, refunded_at, notes, processed_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(payment.id().into_uuid())
        .bind(payment.sale_id().into_uuid())
        .bind(payment.payment_method().to_string())
        .bind(payment.status().to_string())
        .bind(payment.amount())
        .bind(payment.currency().as_str())
        .bind(payment.amount_tendered())
        .bind(payment.change_given())
        .bind(payment.reference_number())
        .bind(payment.authorization_code())
        .bind(payment.card_last_four())
        .bind(payment.card_brand())
        .bind(payment.refunded_amount())
        .bind(payment.refunded_at())
        .bind(payment.notes())
        .bind(payment.processed_at())
        .bind(payment.created_at())
        .bind(payment.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_payment(&self, payment: &Payment) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            UPDATE payments
            SET status = $2, refunded_amount = $3, refunded_at = $4, notes = $5, updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(payment.id().into_uuid())
        .bind(payment.status().to_string())
        .bind(payment.refunded_amount())
        .bind(payment.refunded_at())
        .bind(payment.notes())
        .bind(payment.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_payments_by_sale(&self, sale_id: SaleId) -> Result<Vec<Payment>, SalesError> {
        self.load_payments(sale_id).await
    }

    async fn find_payment_by_id(
        &self,
        payment_id: PaymentId,
    ) -> Result<Option<Payment>, SalesError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"
            SELECT id, sale_id, payment_method, status, amount, currency, amount_tendered,
                   change_given, reference_number, authorization_code, card_last_four,
                   card_brand, refunded_amount, refunded_at, notes, processed_at,
                   created_at, updated_at
            FROM payments
            WHERE id = $1
            "#,
        )
        .bind(payment_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }
}

// =============================================================================
// Row types
// =============================================================================

#[derive(sqlx::FromRow)]
struct SaleRow {
    id: uuid::Uuid,
    sale_number: String,
    store_id: uuid::Uuid,
    sale_type: String,
    status: String,
    order_status: Option<String>,
    terminal_id: Option<uuid::Uuid>,
    shift_id: Option<uuid::Uuid>,
    cashier_id: Option<uuid::Uuid>,
    customer_id: Option<uuid::Uuid>,
    currency: String,
    subtotal: rust_decimal::Decimal,
    discount_type: Option<String>,
    discount_value: rust_decimal::Decimal,
    discount_amount: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    amount_paid: rust_decimal::Decimal,
    amount_due: rust_decimal::Decimal,
    change_given: rust_decimal::Decimal,
    invoice_number: Option<String>,
    invoice_date: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    internal_notes: Option<String>,
    voided_by_id: Option<uuid::Uuid>,
    voided_at: Option<chrono::DateTime<chrono::Utc>>,
    void_reason: Option<String>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl SaleRow {
    fn into_sale(self, items: Vec<SaleItem>, payments: Vec<Payment>) -> Result<Sale, SalesError> {
        let sale_type: SaleType = self.sale_type.parse().unwrap_or(SaleType::Pos);
        let status: SaleStatus = self.status.parse().unwrap_or(SaleStatus::Draft);
        let order_status: Option<OrderStatus> =
            self.order_status.and_then(|s| s.parse().ok());
        let discount_type: Option<DiscountType> =
            self.discount_type.and_then(|d| d.parse().ok());

        Ok(Sale::reconstitute(
            SaleId::from_uuid(self.id),
            self.sale_number,
            StoreId::from_uuid(self.store_id),
            sale_type,
            status,
            order_status,
            self.terminal_id.map(TerminalId::from_uuid),
            self.shift_id.map(ShiftId::from_uuid),
            self.cashier_id.map(UserId::from_uuid),
            self.customer_id.map(CustomerId::from_uuid),
            Currency::from_string(self.currency),
            self.subtotal,
            discount_type,
            self.discount_value,
            self.discount_amount,
            self.tax_amount,
            self.total,
            self.amount_paid,
            self.amount_due,
            self.change_given,
            self.invoice_number,
            self.invoice_date,
            self.notes,
            self.internal_notes,
            self.voided_by_id.map(UserId::from_uuid),
            self.voided_at,
            self.void_reason,
            self.completed_at,
            items,
            payments,
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct SaleItemRow {
    id: uuid::Uuid,
    sale_id: uuid::Uuid,
    line_number: i32,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    sku: String,
    description: String,
    quantity: rust_decimal::Decimal,
    unit_of_measure: String,
    unit_price: rust_decimal::Decimal,
    unit_cost: rust_decimal::Decimal,
    discount_type: Option<String>,
    discount_value: rust_decimal::Decimal,
    discount_amount: rust_decimal::Decimal,
    tax_rate: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    subtotal: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    reservation_id: Option<uuid::Uuid>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<SaleItemRow> for SaleItem {
    type Error = SalesError;

    fn try_from(row: SaleItemRow) -> Result<Self, Self::Error> {
        let discount_type: Option<DiscountType> =
            row.discount_type.and_then(|d| d.parse().ok());
        let uom = UnitOfMeasure::from_str(&row.unit_of_measure)
            .map_err(|_| SalesError::InvalidUnitOfMeasure)?;

        Ok(SaleItem::reconstitute(
            SaleItemId::from_uuid(row.id),
            SaleId::from_uuid(row.sale_id),
            row.line_number,
            ProductId::from_uuid(row.product_id),
            row.variant_id.map(VariantId::from_uuid),
            row.sku,
            row.description,
            row.quantity,
            uom,
            row.unit_price,
            row.unit_cost,
            discount_type,
            row.discount_value,
            row.discount_amount,
            row.tax_rate,
            row.tax_amount,
            row.subtotal,
            row.total,
            row.reservation_id.map(ReservationId::from_uuid),
            row.notes,
            row.created_at,
            row.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct PaymentRow {
    id: uuid::Uuid,
    sale_id: uuid::Uuid,
    payment_method: String,
    status: String,
    amount: rust_decimal::Decimal,
    currency: String,
    amount_tendered: Option<rust_decimal::Decimal>,
    change_given: Option<rust_decimal::Decimal>,
    reference_number: Option<String>,
    authorization_code: Option<String>,
    card_last_four: Option<String>,
    card_brand: Option<String>,
    refunded_amount: rust_decimal::Decimal,
    refunded_at: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    processed_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<PaymentRow> for Payment {
    type Error = SalesError;

    fn try_from(row: PaymentRow) -> Result<Self, Self::Error> {
        let method: PaymentMethod = row.payment_method.parse().unwrap_or(PaymentMethod::Cash);
        let status: PaymentStatus = row.status.parse().unwrap_or(PaymentStatus::Pending);

        Ok(Payment::reconstitute(
            PaymentId::from_uuid(row.id),
            SaleId::from_uuid(row.sale_id),
            method,
            status,
            row.amount,
            Currency::from_string(row.currency),
            row.amount_tendered,
            row.change_given,
            row.reference_number,
            row.authorization_code,
            row.card_last_four,
            row.card_brand,
            row.refunded_amount,
            row.refunded_at,
            row.notes,
            row.processed_at,
            row.created_at,
            row.updated_at,
        ))
    }
}
