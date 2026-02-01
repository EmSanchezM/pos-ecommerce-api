//! Shift command DTOs

use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

/// Command to open a new cashier shift
#[derive(Debug, Deserialize)]
pub struct OpenShiftCommand {
    pub store_id: Uuid,
    pub terminal_id: Uuid,
    pub opening_balance: Decimal,
    pub notes: Option<String>,
}

/// Command to close a cashier shift
#[derive(Debug, Deserialize)]
pub struct CloseShiftCommand {
    pub shift_id: Uuid,
    pub closing_balance: Decimal,
    pub closing_notes: Option<String>,
}

/// Command for cash movement (cash in or cash out)
#[derive(Debug, Deserialize)]
pub struct CashMovementCommand {
    pub shift_id: Uuid,
    pub amount: Decimal,
    pub reason: String,
}

/// Filter for listing shifts
#[derive(Debug, Default, Deserialize)]
pub struct ListShiftsQuery {
    pub store_id: Option<Uuid>,
    pub terminal_id: Option<Uuid>,
    pub cashier_id: Option<Uuid>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
