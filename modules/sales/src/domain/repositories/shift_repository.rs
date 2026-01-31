//! CashierShift repository trait

use async_trait::async_trait;

use crate::domain::entities::CashierShift;
use crate::domain::value_objects::{ShiftId, ShiftStatus};
use crate::SalesError;
use pos_core::TerminalId;
use identity::{StoreId, UserId};

/// Filter for querying shifts
#[derive(Debug, Clone, Default)]
pub struct ShiftFilter {
    pub store_id: Option<StoreId>,
    pub terminal_id: Option<TerminalId>,
    pub cashier_id: Option<UserId>,
    pub status: Option<ShiftStatus>,
}

/// Repository trait for CashierShift persistence
#[async_trait]
pub trait ShiftRepository: Send + Sync {
    /// Saves a new shift
    async fn save(&self, shift: &CashierShift) -> Result<(), SalesError>;

    /// Finds a shift by ID
    async fn find_by_id(&self, id: ShiftId) -> Result<Option<CashierShift>, SalesError>;

    /// Finds the current open shift for a terminal
    async fn find_open_by_terminal(
        &self,
        terminal_id: TerminalId,
    ) -> Result<Option<CashierShift>, SalesError>;

    /// Finds the current open shift for a cashier
    async fn find_open_by_cashier(
        &self,
        cashier_id: UserId,
    ) -> Result<Option<CashierShift>, SalesError>;

    /// Updates an existing shift
    async fn update(&self, shift: &CashierShift) -> Result<(), SalesError>;

    /// Finds shifts with pagination
    async fn find_paginated(
        &self,
        filter: ShiftFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CashierShift>, i64), SalesError>;
}
