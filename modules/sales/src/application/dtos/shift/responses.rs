//! Shift response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::CashierShift;

/// Response for a cashier shift
#[derive(Debug, Serialize)]
pub struct ShiftResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub terminal_id: Uuid,
    pub cashier_id: Uuid,
    pub status: String,
    pub opened_at: DateTime<Utc>,
    pub opening_balance: Decimal,
    pub closed_at: Option<DateTime<Utc>>,
    pub closing_balance: Option<Decimal>,
    pub expected_balance: Decimal,
    pub difference: Option<Decimal>,
    pub cash_sales: Decimal,
    pub card_sales: Decimal,
    pub other_sales: Decimal,
    pub total_sales: Decimal,
    pub refunds: Decimal,
    pub cash_in: Decimal,
    pub cash_out: Decimal,
    pub transaction_count: i32,
    pub notes: Option<String>,
    pub closing_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<CashierShift> for ShiftResponse {
    fn from(s: CashierShift) -> Self {
        Self {
            id: s.id().into_uuid(),
            store_id: s.store_id().into_uuid(),
            terminal_id: s.terminal_id().into_uuid(),
            cashier_id: s.cashier_id().into_uuid(),
            status: s.status().to_string(),
            opened_at: s.opened_at(),
            opening_balance: s.opening_balance(),
            closed_at: s.closed_at(),
            closing_balance: s.closing_balance(),
            expected_balance: s.expected_balance(),
            difference: s.cash_difference(),
            cash_sales: s.cash_sales(),
            card_sales: s.card_sales(),
            other_sales: s.other_sales(),
            total_sales: s.total_sales(),
            refunds: s.refunds(),
            cash_in: s.cash_in(),
            cash_out: s.cash_out(),
            transaction_count: s.transaction_count(),
            notes: s.notes().map(String::from),
            closing_notes: s.closing_notes().map(String::from),
            created_at: s.created_at(),
            updated_at: s.updated_at(),
        }
    }
}

/// Shift report response with detailed breakdown
#[derive(Debug, Serialize)]
pub struct ShiftReportResponse {
    pub shift: ShiftResponse,
    pub sales_breakdown: SalesBreakdown,
    pub payment_breakdown: Vec<PaymentBreakdownItem>,
}

/// Sales breakdown in the shift report
#[derive(Debug, Serialize)]
pub struct SalesBreakdown {
    pub total_sales: Decimal,
    pub total_refunds: Decimal,
    pub net_sales: Decimal,
    pub transaction_count: i32,
    pub average_transaction: Decimal,
}

/// Payment method breakdown item
#[derive(Debug, Serialize)]
pub struct PaymentBreakdownItem {
    pub payment_method: String,
    pub amount: Decimal,
    pub count: i32,
}

/// Paginated response for shift list
#[derive(Debug, Serialize)]
pub struct ShiftListResponse {
    pub data: Vec<ShiftResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
