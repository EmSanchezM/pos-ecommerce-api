//! PostgreSQL ShiftRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::CashierShift;
use crate::domain::repositories::{ShiftFilter, ShiftRepository};
use crate::domain::value_objects::{ShiftId, ShiftStatus};
use crate::SalesError;
use identity::{StoreId, UserId};
use pos_core::TerminalId;

/// PostgreSQL implementation of ShiftRepository
pub struct PgShiftRepository {
    pool: PgPool,
}

impl PgShiftRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShiftRepository for PgShiftRepository {
    async fn save(&self, shift: &CashierShift) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO cashier_shifts (
                id, store_id, terminal_id, cashier_id, status, opened_at, closed_at,
                opening_balance, closing_balance, expected_balance, cash_sales, card_sales,
                other_sales, refunds, cash_in, cash_out, transaction_count, notes,
                closing_notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            "#,
        )
        .bind(shift.id().into_uuid())
        .bind(shift.store_id().into_uuid())
        .bind(shift.terminal_id().into_uuid())
        .bind(shift.cashier_id().into_uuid())
        .bind(shift.status().to_string())
        .bind(shift.opened_at())
        .bind(shift.closed_at())
        .bind(shift.opening_balance())
        .bind(shift.closing_balance())
        .bind(shift.expected_balance())
        .bind(shift.cash_sales())
        .bind(shift.card_sales())
        .bind(shift.other_sales())
        .bind(shift.refunds())
        .bind(shift.cash_in())
        .bind(shift.cash_out())
        .bind(shift.transaction_count())
        .bind(shift.notes())
        .bind(shift.closing_notes())
        .bind(shift.created_at())
        .bind(shift.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: ShiftId) -> Result<Option<CashierShift>, SalesError> {
        let row = sqlx::query_as::<_, ShiftRow>(
            r#"
            SELECT id, store_id, terminal_id, cashier_id, status, opened_at, closed_at,
                   opening_balance, closing_balance, expected_balance, cash_sales, card_sales,
                   other_sales, refunds, cash_in, cash_out, transaction_count, notes,
                   closing_notes, created_at, updated_at
            FROM cashier_shifts
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_open_by_terminal(
        &self,
        terminal_id: TerminalId,
    ) -> Result<Option<CashierShift>, SalesError> {
        let row = sqlx::query_as::<_, ShiftRow>(
            r#"
            SELECT id, store_id, terminal_id, cashier_id, status, opened_at, closed_at,
                   opening_balance, closing_balance, expected_balance, cash_sales, card_sales,
                   other_sales, refunds, cash_in, cash_out, transaction_count, notes,
                   closing_notes, created_at, updated_at
            FROM cashier_shifts
            WHERE terminal_id = $1 AND status = 'open'
            "#,
        )
        .bind(terminal_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_open_by_cashier(
        &self,
        cashier_id: UserId,
    ) -> Result<Option<CashierShift>, SalesError> {
        let row = sqlx::query_as::<_, ShiftRow>(
            r#"
            SELECT id, store_id, terminal_id, cashier_id, status, opened_at, closed_at,
                   opening_balance, closing_balance, expected_balance, cash_sales, card_sales,
                   other_sales, refunds, cash_in, cash_out, transaction_count, notes,
                   closing_notes, created_at, updated_at
            FROM cashier_shifts
            WHERE cashier_id = $1 AND status = 'open'
            "#,
        )
        .bind(cashier_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, shift: &CashierShift) -> Result<(), SalesError> {
        let result = sqlx::query(
            r#"
            UPDATE cashier_shifts
            SET status = $2, closed_at = $3, closing_balance = $4, expected_balance = $5,
                cash_sales = $6, card_sales = $7, other_sales = $8, refunds = $9,
                cash_in = $10, cash_out = $11, transaction_count = $12, notes = $13,
                closing_notes = $14, updated_at = $15
            WHERE id = $1
            "#,
        )
        .bind(shift.id().into_uuid())
        .bind(shift.status().to_string())
        .bind(shift.closed_at())
        .bind(shift.closing_balance())
        .bind(shift.expected_balance())
        .bind(shift.cash_sales())
        .bind(shift.card_sales())
        .bind(shift.other_sales())
        .bind(shift.refunds())
        .bind(shift.cash_in())
        .bind(shift.cash_out())
        .bind(shift.transaction_count())
        .bind(shift.notes())
        .bind(shift.closing_notes())
        .bind(shift.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(SalesError::ShiftNotFound(shift.id().into_uuid()));
        }

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: ShiftFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CashierShift>, i64), SalesError> {
        let offset = (page - 1) * page_size;

        let mut count_query = String::from("SELECT COUNT(*) FROM cashier_shifts WHERE 1=1");
        let mut param_idx = 1;

        if filter.store_id.is_some() {
            count_query.push_str(&format!(" AND store_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.terminal_id.is_some() {
            count_query.push_str(&format!(" AND terminal_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.cashier_id.is_some() {
            count_query.push_str(&format!(" AND cashier_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            count_query.push_str(&format!(" AND status = ${}", param_idx));
        }

        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query);
        if let Some(store_id) = filter.store_id {
            count_builder = count_builder.bind(store_id.into_uuid());
        }
        if let Some(terminal_id) = filter.terminal_id {
            count_builder = count_builder.bind(terminal_id.into_uuid());
        }
        if let Some(cashier_id) = filter.cashier_id {
            count_builder = count_builder.bind(cashier_id.into_uuid());
        }
        if let Some(status) = filter.status {
            count_builder = count_builder.bind(status.to_string());
        }
        let total_count = count_builder.fetch_one(&self.pool).await?;

        let mut data_query = String::from(
            r#"SELECT id, store_id, terminal_id, cashier_id, status, opened_at, closed_at,
                   opening_balance, closing_balance, expected_balance, cash_sales, card_sales,
                   other_sales, refunds, cash_in, cash_out, transaction_count, notes,
                   closing_notes, created_at, updated_at
            FROM cashier_shifts WHERE 1=1"#,
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
        if filter.cashier_id.is_some() {
            data_query.push_str(&format!(" AND cashier_id = ${}", param_idx));
            param_idx += 1;
        }
        if filter.status.is_some() {
            data_query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        data_query.push_str(&format!(
            " ORDER BY opened_at DESC LIMIT ${} OFFSET ${}",
            param_idx,
            param_idx + 1
        ));

        let mut data_builder = sqlx::query_as::<_, ShiftRow>(&data_query);
        if let Some(store_id) = filter.store_id {
            data_builder = data_builder.bind(store_id.into_uuid());
        }
        if let Some(terminal_id) = filter.terminal_id {
            data_builder = data_builder.bind(terminal_id.into_uuid());
        }
        if let Some(cashier_id) = filter.cashier_id {
            data_builder = data_builder.bind(cashier_id.into_uuid());
        }
        if let Some(status) = filter.status {
            data_builder = data_builder.bind(status.to_string());
        }
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;
        let shifts: Result<Vec<CashierShift>, SalesError> =
            rows.into_iter().map(|r| r.try_into()).collect();

        Ok((shifts?, total_count))
    }
}

#[derive(sqlx::FromRow)]
struct ShiftRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    terminal_id: uuid::Uuid,
    cashier_id: uuid::Uuid,
    status: String,
    opened_at: chrono::DateTime<chrono::Utc>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
    opening_balance: rust_decimal::Decimal,
    closing_balance: Option<rust_decimal::Decimal>,
    expected_balance: rust_decimal::Decimal,
    cash_sales: rust_decimal::Decimal,
    card_sales: rust_decimal::Decimal,
    other_sales: rust_decimal::Decimal,
    refunds: rust_decimal::Decimal,
    cash_in: rust_decimal::Decimal,
    cash_out: rust_decimal::Decimal,
    transaction_count: i32,
    notes: Option<String>,
    closing_notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ShiftRow> for CashierShift {
    type Error = SalesError;

    fn try_from(row: ShiftRow) -> Result<Self, Self::Error> {
        let status: ShiftStatus = row.status.parse().unwrap_or(ShiftStatus::Closed);

        Ok(CashierShift::reconstitute(
            ShiftId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            TerminalId::from_uuid(row.terminal_id),
            UserId::from_uuid(row.cashier_id),
            status,
            row.opened_at,
            row.closed_at,
            row.opening_balance,
            row.closing_balance,
            row.expected_balance,
            row.cash_sales,
            row.card_sales,
            row.other_sales,
            row.refunds,
            row.cash_in,
            row.cash_out,
            row.transaction_count,
            row.notes,
            row.closing_notes,
            row.created_at,
            row.updated_at,
        ))
    }
}
