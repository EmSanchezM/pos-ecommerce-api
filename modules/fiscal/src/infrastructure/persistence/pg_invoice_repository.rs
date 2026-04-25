//! PostgreSQL InvoiceRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::FiscalError;
use crate::domain::entities::{Invoice, InvoiceLine};
use crate::domain::repositories::{InvoiceFilter, InvoiceRepository};
use crate::domain::value_objects::{InvoiceId, InvoiceLineId, InvoiceStatus, InvoiceType, TaxType};
use identity::{StoreId, UserId};
use inventory::Currency;
use pos_core::TerminalId;
use sales::{CustomerId, SaleId};

/// PostgreSQL implementation of InvoiceRepository
pub struct PgInvoiceRepository {
    pool: PgPool,
}

impl PgInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn load_lines(&self, invoice_id: InvoiceId) -> Result<Vec<InvoiceLine>, FiscalError> {
        let rows = sqlx::query_as::<_, InvoiceLineRow>(
            r#"
            SELECT id, invoice_id, line_number, product_id, variant_id, sku, description,
                   quantity, unit_of_measure, unit_price, discount_amount, tax_type,
                   tax_rate, tax_amount, subtotal, total, is_exempt, created_at, updated_at
            FROM invoice_lines
            WHERE invoice_id = $1
            ORDER BY line_number
            "#,
        )
        .bind(invoice_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }
}

#[async_trait]
impl InvoiceRepository for PgInvoiceRepository {
    async fn save(&self, invoice: &Invoice) -> Result<(), FiscalError> {
        sqlx::query(
            r#"
            INSERT INTO invoices (
                id, invoice_number, store_id, terminal_id, sale_id, cai_range_id,
                invoice_type, status, customer_id, customer_name, customer_rtn,
                customer_address, currency, subtotal, exempt_amount, taxable_amount_15,
                taxable_amount_18, tax_15, tax_18, total_tax, discount_amount, total,
                amount_in_words, payment_method, cai_number, cai_expiry_date,
                range_start, range_end, voided_by_id, voided_at, void_reason,
                void_invoice_id, original_invoice_id, printed_at, emitted_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                    $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
                    $31, $32, $33, $34, $35, $36, $37)
            "#,
        )
        .bind(invoice.id().into_uuid())
        .bind(invoice.invoice_number())
        .bind(invoice.store_id().into_uuid())
        .bind(invoice.terminal_id().into_uuid())
        .bind(invoice.sale_id().into_uuid())
        .bind(invoice.cai_range_id())
        .bind(invoice.invoice_type().to_string())
        .bind(invoice.status().to_string())
        .bind(invoice.customer_id().map(|c| c.into_uuid()))
        .bind(invoice.customer_name())
        .bind(invoice.customer_rtn())
        .bind(invoice.customer_address())
        .bind(invoice.currency().as_str())
        .bind(invoice.subtotal())
        .bind(invoice.exempt_amount())
        .bind(invoice.taxable_amount_15())
        .bind(invoice.taxable_amount_18())
        .bind(invoice.tax_15())
        .bind(invoice.tax_18())
        .bind(invoice.total_tax())
        .bind(invoice.discount_amount())
        .bind(invoice.total())
        .bind(invoice.amount_in_words())
        .bind(invoice.payment_method())
        .bind(invoice.cai_number())
        .bind(invoice.cai_expiry_date())
        .bind(invoice.range_start())
        .bind(invoice.range_end())
        .bind(invoice.voided_by_id().map(|u| u.into_uuid()))
        .bind(invoice.voided_at())
        .bind(invoice.void_reason())
        .bind(invoice.void_invoice_id().map(|i| i.into_uuid()))
        .bind(invoice.original_invoice_id().map(|i| i.into_uuid()))
        .bind(invoice.printed_at())
        .bind(invoice.emitted_at())
        .bind(invoice.created_at())
        .bind(invoice.updated_at())
        .execute(&self.pool)
        .await?;

        for line in invoice.items() {
            self.save_line(line).await?;
        }

        Ok(())
    }

    async fn find_by_id(&self, id: InvoiceId) -> Result<Option<Invoice>, FiscalError> {
        let row = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT id, invoice_number, store_id, terminal_id, sale_id, cai_range_id,
                   invoice_type, status, customer_id, customer_name, customer_rtn,
                   customer_address, currency, subtotal, exempt_amount, taxable_amount_15,
                   taxable_amount_18, tax_15, tax_18, total_tax, discount_amount, total,
                   amount_in_words, payment_method, cai_number, cai_expiry_date,
                   range_start, range_end, voided_by_id, voided_at, void_reason,
                   void_invoice_id, original_invoice_id, printed_at, emitted_at,
                   created_at, updated_at
            FROM invoices
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_invoice(Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_id_with_lines(&self, id: InvoiceId) -> Result<Option<Invoice>, FiscalError> {
        let row = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT id, invoice_number, store_id, terminal_id, sale_id, cai_range_id,
                   invoice_type, status, customer_id, customer_name, customer_rtn,
                   customer_address, currency, subtotal, exempt_amount, taxable_amount_15,
                   taxable_amount_18, tax_15, tax_18, total_tax, discount_amount, total,
                   amount_in_words, payment_method, cai_number, cai_expiry_date,
                   range_start, range_end, voided_by_id, voided_at, void_reason,
                   void_invoice_id, original_invoice_id, printed_at, emitted_at,
                   created_at, updated_at
            FROM invoices
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let lines = self.load_lines(id).await?;
                Ok(Some(r.into_invoice(lines)?))
            }
            None => Ok(None),
        }
    }

    async fn find_by_number(
        &self,
        store_id: StoreId,
        number: &str,
    ) -> Result<Option<Invoice>, FiscalError> {
        let row = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT id, invoice_number, store_id, terminal_id, sale_id, cai_range_id,
                   invoice_type, status, customer_id, customer_name, customer_rtn,
                   customer_address, currency, subtotal, exempt_amount, taxable_amount_15,
                   taxable_amount_18, tax_15, tax_18, total_tax, discount_amount, total,
                   amount_in_words, payment_method, cai_number, cai_expiry_date,
                   range_start, range_end, voided_by_id, voided_at, void_reason,
                   void_invoice_id, original_invoice_id, printed_at, emitted_at,
                   created_at, updated_at
            FROM invoices
            WHERE store_id = $1 AND invoice_number = $2
            "#,
        )
        .bind(store_id.into_uuid())
        .bind(number)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_invoice(Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn find_by_sale_id(&self, sale_id: SaleId) -> Result<Option<Invoice>, FiscalError> {
        let row = sqlx::query_as::<_, InvoiceRow>(
            r#"
            SELECT id, invoice_number, store_id, terminal_id, sale_id, cai_range_id,
                   invoice_type, status, customer_id, customer_name, customer_rtn,
                   customer_address, currency, subtotal, exempt_amount, taxable_amount_15,
                   taxable_amount_18, tax_15, tax_18, total_tax, discount_amount, total,
                   amount_in_words, payment_method, cai_number, cai_expiry_date,
                   range_start, range_end, voided_by_id, voided_at, void_reason,
                   void_invoice_id, original_invoice_id, printed_at, emitted_at,
                   created_at, updated_at
            FROM invoices
            WHERE sale_id = $1
            "#,
        )
        .bind(sale_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_invoice(Vec::new())?)),
            None => Ok(None),
        }
    }

    async fn update(&self, invoice: &Invoice) -> Result<(), FiscalError> {
        let result = sqlx::query(
            r#"
            UPDATE invoices
            SET status = $2, customer_id = $3, customer_name = $4, customer_rtn = $5,
                customer_address = $6, voided_by_id = $7, voided_at = $8, void_reason = $9,
                void_invoice_id = $10, printed_at = $11, updated_at = $12
            WHERE id = $1
            "#,
        )
        .bind(invoice.id().into_uuid())
        .bind(invoice.status().to_string())
        .bind(invoice.customer_id().map(|c| c.into_uuid()))
        .bind(invoice.customer_name())
        .bind(invoice.customer_rtn())
        .bind(invoice.customer_address())
        .bind(invoice.voided_by_id().map(|u| u.into_uuid()))
        .bind(invoice.voided_at())
        .bind(invoice.void_reason())
        .bind(invoice.void_invoice_id().map(|i| i.into_uuid()))
        .bind(invoice.printed_at())
        .bind(invoice.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(FiscalError::InvoiceNotFound(invoice.id().into_uuid()));
        }

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: InvoiceFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Invoice>, i64), FiscalError> {
        let offset = (page - 1) * page_size;

        let mut count_query = String::from("SELECT COUNT(*) FROM invoices WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.terminal_id.is_some() {
            count_query.push_str(&format!(" AND terminal_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.invoice_type.is_some() {
            count_query.push_str(&format!(" AND invoice_type = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            count_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.customer_rtn.is_some() {
            count_query.push_str(&format!(" AND customer_rtn = ${}", param_idx));
            param_idx += 1;
        }
        if filter.date_from.is_some() {
            count_query.push_str(&format!(" AND emitted_at >= ${}", param_idx));
            param_idx += 1;
        }
        if filter.date_to.is_some() {
            count_query.push_str(&format!(" AND emitted_at <= ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            count_query.push_str(&format!(
                " AND (invoice_number ILIKE ${} OR customer_name ILIKE ${} OR customer_rtn ILIKE ${})",
                param_idx, param_idx, param_idx
            ));
        }

        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(store_id) = filter.store_id {
            count_builder = count_builder.bind(store_id.into_uuid());
        }
        if let Some(terminal_id) = filter.terminal_id {
            count_builder = count_builder.bind(terminal_id.into_uuid());
        }
        if let Some(invoice_type) = filter.invoice_type {
            count_builder = count_builder.bind(invoice_type.to_string());
        }
        if let Some(status) = filter.status {
            count_builder = count_builder.bind(status.to_string());
        }
        if let Some(ref customer_rtn) = filter.customer_rtn {
            count_builder = count_builder.bind(customer_rtn.clone());
        }
        if let Some(date_from) = filter.date_from {
            count_builder = count_builder.bind(date_from);
        }
        if let Some(date_to) = filter.date_to {
            count_builder = count_builder.bind(date_to);
        }
        if let Some(ref search) = filter.search {
            count_builder = count_builder.bind(format!("%{}%", search));
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        let mut data_query = String::from(
            r#"SELECT id, invoice_number, store_id, terminal_id, sale_id, cai_range_id,
                   invoice_type, status, customer_id, customer_name, customer_rtn,
                   customer_address, currency, subtotal, exempt_amount, taxable_amount_15,
                   taxable_amount_18, tax_15, tax_18, total_tax, discount_amount, total,
                   amount_in_words, payment_method, cai_number, cai_expiry_date,
                   range_start, range_end, voided_by_id, voided_at, void_reason,
                   void_invoice_id, original_invoice_id, printed_at, emitted_at,
                   created_at, updated_at
            FROM invoices WHERE 1=1"#,
        );

        param_idx = 1;
        if filter.store_id.is_some() {
            data_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.terminal_id.is_some() {
            data_query.push_str(&format!(" AND terminal_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.invoice_type.is_some() {
            data_query.push_str(&format!(" AND invoice_type = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            data_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if filter.customer_rtn.is_some() {
            data_query.push_str(&format!(" AND customer_rtn = ${}", param_idx));
            param_idx += 1;
        }
        if filter.date_from.is_some() {
            data_query.push_str(&format!(" AND emitted_at >= ${}", param_idx));
            param_idx += 1;
        }
        if filter.date_to.is_some() {
            data_query.push_str(&format!(" AND emitted_at <= ${}", param_idx));
            param_idx += 1;
        }
        if filter.search.is_some() {
            data_query.push_str(&format!(
                " AND (invoice_number ILIKE ${} OR customer_name ILIKE ${} OR customer_rtn ILIKE ${})",
                param_idx, param_idx, param_idx
            ));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        let mut data_builder = sqlx::query_as::<_, InvoiceRow>(&data_query);
        if let Some(store_id) = filter.store_id {
            data_builder = data_builder.bind(store_id.into_uuid());
        }
        if let Some(terminal_id) = filter.terminal_id {
            data_builder = data_builder.bind(terminal_id.into_uuid());
        }
        if let Some(invoice_type) = filter.invoice_type {
            data_builder = data_builder.bind(invoice_type.to_string());
        }
        if let Some(status) = filter.status {
            data_builder = data_builder.bind(status.to_string());
        }
        if let Some(ref customer_rtn) = filter.customer_rtn {
            data_builder = data_builder.bind(customer_rtn.clone());
        }
        if let Some(date_from) = filter.date_from {
            data_builder = data_builder.bind(date_from);
        }
        if let Some(date_to) = filter.date_to {
            data_builder = data_builder.bind(date_to);
        }
        if let Some(ref search) = filter.search {
            data_builder = data_builder.bind(format!("%{}%", search));
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;
        let invoices: Result<Vec<Invoice>, FiscalError> = rows
            .into_iter()
            .map(|r| r.into_invoice(Vec::new()))
            .collect();

        Ok((invoices?, total_count))
    }

    async fn save_line(&self, line: &InvoiceLine) -> Result<(), FiscalError> {
        sqlx::query(
            r#"
            INSERT INTO invoice_lines (
                id, invoice_id, line_number, product_id, variant_id, sku, description,
                quantity, unit_of_measure, unit_price, discount_amount, tax_type,
                tax_rate, tax_amount, subtotal, total, is_exempt, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            "#,
        )
        .bind(line.id().into_uuid())
        .bind(line.invoice_id().into_uuid())
        .bind(line.line_number())
        .bind(line.product_id())
        .bind(line.variant_id())
        .bind(line.sku())
        .bind(line.description())
        .bind(line.quantity())
        .bind(line.unit_of_measure())
        .bind(line.unit_price())
        .bind(line.discount_amount())
        .bind(line.tax_type().to_string())
        .bind(line.tax_rate())
        .bind(line.tax_amount())
        .bind(line.subtotal())
        .bind(line.total())
        .bind(line.is_exempt())
        .bind(line.created_at())
        .bind(line.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_lines_by_invoice(
        &self,
        invoice_id: InvoiceId,
    ) -> Result<Vec<InvoiceLine>, FiscalError> {
        self.load_lines(invoice_id).await
    }
}

// =============================================================================
// Row types
// =============================================================================

#[derive(sqlx::FromRow)]
struct InvoiceRow {
    id: uuid::Uuid,
    invoice_number: String,
    store_id: uuid::Uuid,
    terminal_id: uuid::Uuid,
    sale_id: uuid::Uuid,
    cai_range_id: uuid::Uuid,
    invoice_type: String,
    status: String,
    customer_id: Option<uuid::Uuid>,
    customer_name: String,
    customer_rtn: Option<String>,
    customer_address: Option<String>,
    currency: String,
    subtotal: rust_decimal::Decimal,
    exempt_amount: rust_decimal::Decimal,
    taxable_amount_15: rust_decimal::Decimal,
    taxable_amount_18: rust_decimal::Decimal,
    tax_15: rust_decimal::Decimal,
    tax_18: rust_decimal::Decimal,
    total_tax: rust_decimal::Decimal,
    discount_amount: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    amount_in_words: String,
    payment_method: String,
    cai_number: String,
    cai_expiry_date: chrono::DateTime<chrono::Utc>,
    range_start: String,
    range_end: String,
    voided_by_id: Option<uuid::Uuid>,
    voided_at: Option<chrono::DateTime<chrono::Utc>>,
    void_reason: Option<String>,
    void_invoice_id: Option<uuid::Uuid>,
    original_invoice_id: Option<uuid::Uuid>,
    printed_at: Option<chrono::DateTime<chrono::Utc>>,
    emitted_at: chrono::DateTime<chrono::Utc>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl InvoiceRow {
    fn into_invoice(self, lines: Vec<InvoiceLine>) -> Result<Invoice, FiscalError> {
        let invoice_type: InvoiceType = self.invoice_type.parse().unwrap_or(InvoiceType::Standard);
        let status: InvoiceStatus = self.status.parse().unwrap_or(InvoiceStatus::Emitted);

        Ok(Invoice::reconstitute(
            InvoiceId::from_uuid(self.id),
            self.invoice_number,
            StoreId::from_uuid(self.store_id),
            TerminalId::from_uuid(self.terminal_id),
            SaleId::from_uuid(self.sale_id),
            self.cai_range_id,
            invoice_type,
            status,
            self.customer_id.map(CustomerId::from_uuid),
            self.customer_name,
            self.customer_rtn,
            self.customer_address,
            Currency::from_string(self.currency),
            self.subtotal,
            self.exempt_amount,
            self.taxable_amount_15,
            self.taxable_amount_18,
            self.tax_15,
            self.tax_18,
            self.total_tax,
            self.discount_amount,
            self.total,
            self.amount_in_words,
            self.payment_method,
            self.cai_number,
            self.cai_expiry_date,
            self.range_start,
            self.range_end,
            self.voided_by_id.map(UserId::from_uuid),
            self.voided_at,
            self.void_reason,
            self.void_invoice_id.map(InvoiceId::from_uuid),
            self.original_invoice_id.map(InvoiceId::from_uuid),
            self.printed_at,
            self.emitted_at,
            lines,
            self.created_at,
            self.updated_at,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct InvoiceLineRow {
    id: uuid::Uuid,
    invoice_id: uuid::Uuid,
    line_number: i32,
    product_id: uuid::Uuid,
    variant_id: Option<uuid::Uuid>,
    sku: String,
    description: String,
    quantity: rust_decimal::Decimal,
    unit_of_measure: String,
    unit_price: rust_decimal::Decimal,
    discount_amount: rust_decimal::Decimal,
    tax_type: String,
    tax_rate: rust_decimal::Decimal,
    tax_amount: rust_decimal::Decimal,
    subtotal: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    is_exempt: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<InvoiceLineRow> for InvoiceLine {
    type Error = FiscalError;

    fn try_from(row: InvoiceLineRow) -> Result<Self, Self::Error> {
        let tax_type: TaxType = row.tax_type.parse().unwrap_or(TaxType::Isv15);

        Ok(InvoiceLine::reconstitute(
            InvoiceLineId::from_uuid(row.id),
            InvoiceId::from_uuid(row.invoice_id),
            row.line_number,
            row.product_id,
            row.variant_id,
            row.sku,
            row.description,
            row.quantity,
            row.unit_of_measure,
            row.unit_price,
            row.discount_amount,
            tax_type,
            row.tax_rate,
            row.tax_amount,
            row.subtotal,
            row.total,
            row.is_exempt,
            row.created_at,
            row.updated_at,
        ))
    }
}
