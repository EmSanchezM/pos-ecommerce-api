//! PostgreSQL CreditNoteRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;
use std::str::FromStr;

use crate::domain::entities::{CreditNote, CreditNoteItem};
use crate::domain::repositories::{CreditNoteFilter, CreditNoteRepository};
use crate::domain::value_objects::{
    CreditNoteId, CreditNoteItemId, CreditNoteStatus, ReturnReason, ReturnType, SaleId, SaleItemId,
};
use crate::SalesError;
use identity::{StoreId, UserId};
use inventory::{Currency, ProductId, UnitOfMeasure, VariantId};

/// PostgreSQL implementation of CreditNoteRepository
pub struct PgCreditNoteRepository {
    pool: PgPool,
}

impl PgCreditNoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn load_items(&self, credit_note_id: CreditNoteId) -> Result<Vec<CreditNoteItem>, SalesError> {
        let rows = sqlx::query_as::<_, CreditNoteItemRow>(
            r#"
            SELECT id, credit_note_id, original_sale_item_id, product_id, variant_id, sku,
                   description, return_quantity, unit_of_measure, unit_price, tax_rate,
                   tax_amount, subtotal, total, restock, condition, notes, created_at, updated_at
            FROM credit_note_items
            WHERE credit_note_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(credit_note_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[async_trait]
impl CreditNoteRepository for PgCreditNoteRepository {
    async fn save(&self, credit_note: &CreditNote) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO credit_notes (
                id, credit_note_number, store_id, original_sale_id, original_invoice_number,
                status, return_type, return_reason, reason_details, currency, subtotal,
                tax_amount, total, refund_method, refunded_amount, created_by_id,
                submitted_by_id, submitted_at, approved_by_id, approved_at, applied_by_id,
                applied_at, cancelled_by_id, cancelled_at, cancellation_reason, notes,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28)
            "#,
        )
        .bind(credit_note.id().into_uuid())
        .bind(credit_note.credit_note_number())
        .bind(credit_note.store_id().into_uuid())
        .bind(credit_note.original_sale_id().into_uuid())
        .bind(credit_note.original_invoice_number())
        .bind(credit_note.status().to_string())
        .bind(credit_note.return_type().to_string())
        .bind(credit_note.return_reason().to_string())
        .bind(credit_note.reason_details())
        .bind(credit_note.currency().as_str())
        .bind(credit_note.subtotal())
        .bind(credit_note.tax_amount())
        .bind(credit_note.total())
        .bind(credit_note.refund_method())
        .bind(credit_note.refunded_amount())
        .bind(credit_note.created_by_id().into_uuid())
        .bind(credit_note.submitted_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.submitted_at())
        .bind(credit_note.approved_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.approved_at())
        .bind(credit_note.applied_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.applied_at())
        .bind(credit_note.cancelled_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.cancelled_at())
        .bind(credit_note.cancellation_reason())
        .bind(credit_note.notes())
        .bind(credit_note.created_at())
        .bind(credit_note.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: CreditNoteId) -> Result<Option<CreditNote>, SalesError> {
        let row = sqlx::query_as::<_, CreditNoteRow>(
            r#"
            SELECT id, credit_note_number, store_id, original_sale_id, original_invoice_number,
                   status, return_type, return_reason, reason_details, currency, subtotal,
                   tax_amount, total, refund_method, refunded_amount, created_by_id,
                   submitted_by_id, submitted_at, approved_by_id, approved_at, applied_by_id,
                   applied_at, cancelled_by_id, cancelled_at, cancellation_reason, notes,
                   created_at, updated_at
            FROM credit_notes
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_credit_note(Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_id_with_items(
        &self,
        id: CreditNoteId,
    ) -> Result<Option<CreditNote>, SalesError> {
        let row = sqlx::query_as::<_, CreditNoteRow>(
            r#"
            SELECT id, credit_note_number, store_id, original_sale_id, original_invoice_number,
                   status, return_type, return_reason, reason_details, currency, subtotal,
                   tax_amount, total, refund_method, refunded_amount, created_by_id,
                   submitted_by_id, submitted_at, approved_by_id, approved_at, applied_by_id,
                   applied_at, cancelled_by_id, cancelled_at, cancellation_reason, notes,
                   created_at, updated_at
            FROM credit_notes
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let items = self.load_items(id).await?;
                Ok(Some(r.into_credit_note(items)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_number(
        &self,
        store_id: StoreId,
        number: &str,
    ) -> Result<Option<CreditNote>, SalesError> {
        let row = sqlx::query_as::<_, CreditNoteRow>(
            r#"
            SELECT id, credit_note_number, store_id, original_sale_id, original_invoice_number,
                   status, return_type, return_reason, reason_details, currency, subtotal,
                   tax_amount, total, refund_method, refunded_amount, created_by_id,
                   submitted_by_id, submitted_at, approved_by_id, approved_at, applied_by_id,
                   applied_at, cancelled_by_id, cancelled_at, cancellation_reason, notes,
                   created_at, updated_at
            FROM credit_notes
            WHERE store_id = $1 AND credit_note_number = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(number)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_credit_note(Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_sale(&self, sale_id: SaleId) -> Result<Vec<CreditNote>, SalesError> {
        let rows = sqlx::query_as::<_, CreditNoteRow>(
            r#"
            SELECT id, credit_note_number, store_id, original_sale_id, original_invoice_number,
                   status, return_type, return_reason, reason_details, currency, subtotal,
                   tax_amount, total, refund_method, refunded_amount, created_by_id,
                   submitted_by_id, submitted_at, approved_by_id, approved_at, applied_by_id,
                   applied_at, cancelled_by_id, cancelled_at, cancellation_reason, notes,
                   created_at, updated_at
            FROM credit_notes
            WHERE original_sale_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(sale_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| r.into_credit_note(Vec::new()))
            .collect()
    }

    async fn update(&self, credit_note: &CreditNote) -> Result<(), SalesError> {
        let result = sqlx::query(
            r#"
            UPDATE credit_notes
            SET status = $2, reason_details = $3, subtotal = $4, tax_amount = $5, total = $6,
                refund_method = $7, refunded_amount = $8, submitted_by_id = $9, submitted_at = $10,
                approved_by_id = $11, approved_at = $12, applied_by_id = $13, applied_at = $14,
                cancelled_by_id = $15, cancelled_at = $16, cancellation_reason = $17, notes = $18,
                updated_at = $19
            WHERE id = $1
            "#,
        )
        .bind(credit_note.id().into_uuid())
        .bind(credit_note.status().to_string())
        .bind(credit_note.reason_details())
        .bind(credit_note.subtotal())
        .bind(credit_note.tax_amount())
        .bind(credit_note.total())
        .bind(credit_note.refund_method())
        .bind(credit_note.refunded_amount())
        .bind(credit_note.submitted_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.submitted_at())
        .bind(credit_note.approved_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.approved_at())
        .bind(credit_note.applied_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.applied_at())
        .bind(credit_note.cancelled_by_id().map(|u| u.into_uuid()))
        .bind(credit_note.cancelled_at())
        .bind(credit_note.cancellation_reason())
        .bind(credit_note.notes())
        .bind(credit_note.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SalesError::CreditNoteNotFound(credit_note.id().into_uuid()));
        }

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: CreditNoteFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CreditNote>, i64), SalesError> {
        let offset = (page - 1) * page_size;

        let mut count_query = String::from("SELECT COUNT(*) FROM credit_notes WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            count_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            count_query.push_str(&format!(
                " AND (credit_note_number ILIKE ${} OR original_invoice_number ILIKE ${})",
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
        if let Some(ref search) = filter.search {
            count_builder = count_builder.bind(format!("%{}%", search));
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        let mut data_query = String::from(
            r#"SELECT id, credit_note_number, store_id, original_sale_id, original_invoice_number,
                   status, return_type, return_reason, reason_details, currency, subtotal,
                   tax_amount, total, refund_method, refunded_amount, created_by_id,
                   submitted_by_id, submitted_at, approved_by_id, approved_at, applied_by_id,
                   applied_at, cancelled_by_id, cancelled_at, cancellation_reason, notes,
                   created_at, updated_at
            FROM credit_notes WHERE 1=1"#,
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
        if filter.search.is_some() {
            data_query.push_str(&format!(
                " AND (credit_note_number ILIKE ${} OR original_invoice_number ILIKE ${})",
                param_idx, param_idx
            ));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        let mut data_builder = sqlx::query_as::<_, CreditNoteRow>(&data_query);
        if let Some(store_id) = filter.store_id {
            data_builder = data_builder.bind(store_id.into_uuid());
        }
        if let Some(status) = filter.status {
            data_builder = data_builder.bind(status.to_string());
        }
        if let Some(ref search) = filter.search {
            data_builder = data_builder.bind(format!("%{}%", search));
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;
        let credit_notes: Result<Vec<CreditNote>, SalesError> = rows
            .into_iter()
            .map(|r| r.into_credit_note(Vec::new()))
            .collect();

        Ok((credit_notes?, total_count))
    }

    async fn generate_credit_note_number(&self, store_id: StoreId) -> Result<String, SalesError> {
        let today = chrono::Utc::now().format("%Y%m%d");
        let prefix = format!("CN-{}", today);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM credit_notes WHERE store_id = $1 AND credit_note_number LIKE $2",
        )
        .bind(store_id.into_uuid())
        .bind(format!("{}%", prefix))
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("{}-{:04}", prefix, count.0 + 1))
    }

    async fn save_item(&self, item: &CreditNoteItem) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO credit_note_items (
                id, credit_note_id, original_sale_item_id, product_id, variant_id, sku,
                description, return_quantity, unit_of_measure, unit_price, tax_rate,
                tax_amount, subtotal, total, restock, condition, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.credit_note_id().into_uuid())
        .bind(item.original_sale_item_id().into_uuid())
        .bind(item.product_id().into_uuid())
        .bind(item.variant_id().map(|v| v.into_uuid()))
        .bind(item.sku())
        .bind(item.description())
        .bind(item.return_quantity())
        .bind(item.unit_of_measure().to_string())
        .bind(item.unit_price())
        .bind(item.tax_rate())
        .bind(item.tax_amount())
        .bind(item.subtotal())
        .bind(item.total())
        .bind(item.restock())
        .bind(item.condition())
        .bind(item.notes())
        .bind(item.created_at())
        .bind(item.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_item(&self, item: &CreditNoteItem) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            UPDATE credit_note_items
            SET return_quantity = $2, tax_amount = $3, subtotal = $4, total = $5,
                restock = $6, condition = $7, notes = $8, updated_at = $9
            WHERE id = $1
            "#,
        )
        .bind(item.id().into_uuid())
        .bind(item.return_quantity())
        .bind(item.tax_amount())
        .bind(item.subtotal())
        .bind(item.total())
        .bind(item.restock())
        .bind(item.condition())
        .bind(item.notes())
        .bind(item.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_item(&self, item_id: CreditNoteItemId) -> Result<(), SalesError> {
        sqlx::query("DELETE FROM credit_note_items WHERE id = $1")
            .bind(item_id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_items_by_credit_note(
        &self,
        credit_note_id: CreditNoteId,
    ) -> Result<Vec<CreditNoteItem>, SalesError> {
        self.load_items(credit_note_id).await
    }

    async fn find_item_by_id(
        &self,
        item_id: CreditNoteItemId,
    ) -> Result<Option<CreditNoteItem>, SalesError> {
        let row = sqlx::query_as::<_, CreditNoteItemRow>(
            r#"
            SELECT id, credit_note_id, original_sale_item_id, product_id, variant_id, sku,
                   description, return_quantity, unit_of_measure, unit_price, tax_rate,
                   tax_amount, subtotal, total, restock, condition, notes, created_at, updated_at
            FROM credit_note_items
            WHERE id = $1
            "#,
        )
        .bind(item_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }
}

// =============================================================================
// Row types
// =============================================================================

#[derive(sqlx::FromRow)]
struct CreditNoteRow {
    id: uuid::Uuid,
    credit_note_number: String,
    store_id: uuid::Uuid,
    original_sale_id: uuid::Uuid,
    original_invoice_number: String,
    status: String,
    return_type: String,
    return_reason: String,
    reason_details: Option<String>,
    currency: String,
    subtotal: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    refund_method: Option<String>,
    refunded_amount: rust_decimal::Decimal,
    created_by_id: uuid::Uuid,
    submitted_by_id: Option<uuid::Uuid>,
    submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    approved_by_id: Option<uuid::Uuid>,
    approved_at: Option<chrono::DateTime<chrono::Utc>>,
    applied_by_id: Option<uuid::Uuid>,
    applied_at: Option<chrono::DateTime<chrono::Utc>>,
    cancelled_by_id: Option<uuid::Uuid>,
    cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    cancellation_reason: Option<String>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl CreditNoteRow {
    fn into_credit_note(self, items: Vec<CreditNoteItem>) -> Result<CreditNote, SalesError> {
        let status: CreditNoteStatus = self.status.parse().unwrap_or(CreditNoteStatus::Draft);
        let return_type: ReturnType = self.return_type.parse().unwrap_or(ReturnType::Partial);
        let return_reason: ReturnReason = self.return_reason.parse().unwrap_or(ReturnReason::Other);

        Ok(CreditNote::reconstitute(
            CreditNoteId::from_uuid(self.id),
            self.credit_note_number,
            StoreId::from_uuid(self.store_id),
            SaleId::from_uuid(self.original_sale_id),
            self.original_invoice_number,
            status,
            return_type,
            return_reason,
            self.reason_details,
            Currency::from_string(self.currency),
            self.subtotal,
            self.tax_amount,
            self.total,
            self.refund_method,
            self.refunded_amount,
            UserId::from_uuid(self.created_by_id),
            self.submitted_by_id.map(UserId::from_uuid),
            self.submitted_at,
            self.approved_by_id.map(UserId::from_uuid),
            self.approved_at,
            self.applied_by_id.map(UserId::from_uuid),
            self.applied_at,
            self.cancelled_by_id.map(UserId::from_uuid),
            self.cancelled_at,
            self.cancellation_reason,
            self.notes,
            items,
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct CreditNoteItemRow {
    id: uuid::Uuid,
    credit_note_id: uuid::Uuid,
    original_sale_item_id: uuid::Uuid,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    sku: String,
    description: String,
    return_quantity: rust_decimal::Decimal,
    unit_of_measure: String,
    unit_price: rust_decimal::Decimal,
    tax_rate: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    subtotal: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    restock: bool,
    condition: Option<String>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<CreditNoteItemRow> for CreditNoteItem {
    type Error = SalesError;

    fn try_from(row: CreditNoteItemRow) -> Result<Self, Self::Error> {
        let uom = UnitOfMeasure::from_str(&row.unit_of_measure)
            .map_err(|_| SalesError::InvalidUnitOfMeasure)?;

        Ok(CreditNoteItem::reconstitute(
            CreditNoteItemId::from_uuid(row.id),
            CreditNoteId::from_uuid(row.credit_note_id),
            SaleItemId::from_uuid(row.original_sale_item_id),
            ProductId::from_uuid(row.product_id),
            row.variant_id.map(VariantId::from_uuid),
            row.sku,
            row.description,
            row.return_quantity,
            uom,
            row.unit_price,
            row.tax_rate,
            row.tax_amount,
            row.subtotal,
            row.total,
            row.restock,
            row.condition,
            row.notes,
            row.created_at,
            row.updated_at,
        ))
    }
}
