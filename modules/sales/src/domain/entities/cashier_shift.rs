//! CashierShift entity - represents a cashier's work shift at a terminal

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{ShiftId, ShiftStatus};
use crate::SalesError;
use pos_core::TerminalId;
use identity::{StoreId, UserId};

/// CashierShift entity representing a cashier's work shift at a terminal.
///
/// Invariants:
/// - Only one open shift per terminal at a time
/// - Only one open shift per cashier at a time
/// - Shift must be open to record sales
/// - Opening balance must be non-negative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashierShift {
    id: ShiftId,
    store_id: StoreId,
    terminal_id: TerminalId,
    cashier_id: UserId,
    status: ShiftStatus,
    opened_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    opening_balance: Decimal,
    closing_balance: Option<Decimal>,
    expected_balance: Decimal,
    cash_sales: Decimal,
    card_sales: Decimal,
    other_sales: Decimal,
    refunds: Decimal,
    cash_in: Decimal,
    cash_out: Decimal,
    transaction_count: i32,
    notes: Option<String>,
    closing_notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CashierShift {
    /// Creates a new open CashierShift
    pub fn create(
        store_id: StoreId,
        terminal_id: TerminalId,
        cashier_id: UserId,
        opening_balance: Decimal,
    ) -> Result<Self, SalesError> {
        if opening_balance < Decimal::ZERO {
            return Err(SalesError::InvalidOpeningBalance);
        }

        let now = Utc::now();
        Ok(Self {
            id: ShiftId::new(),
            store_id,
            terminal_id,
            cashier_id,
            status: ShiftStatus::Open,
            opened_at: now,
            closed_at: None,
            opening_balance,
            closing_balance: None,
            expected_balance: opening_balance,
            cash_sales: Decimal::ZERO,
            card_sales: Decimal::ZERO,
            other_sales: Decimal::ZERO,
            refunds: Decimal::ZERO,
            cash_in: Decimal::ZERO,
            cash_out: Decimal::ZERO,
            transaction_count: 0,
            notes: None,
            closing_notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a CashierShift from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ShiftId,
        store_id: StoreId,
        terminal_id: TerminalId,
        cashier_id: UserId,
        status: ShiftStatus,
        opened_at: DateTime<Utc>,
        closed_at: Option<DateTime<Utc>>,
        opening_balance: Decimal,
        closing_balance: Option<Decimal>,
        expected_balance: Decimal,
        cash_sales: Decimal,
        card_sales: Decimal,
        other_sales: Decimal,
        refunds: Decimal,
        cash_in: Decimal,
        cash_out: Decimal,
        transaction_count: i32,
        notes: Option<String>,
        closing_notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            terminal_id,
            cashier_id,
            status,
            opened_at,
            closed_at,
            opening_balance,
            closing_balance,
            expected_balance,
            cash_sales,
            card_sales,
            other_sales,
            refunds,
            cash_in,
            cash_out,
            transaction_count,
            notes,
            closing_notes,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Closes the shift
    pub fn close(&mut self, closing_balance: Decimal, notes: Option<String>) -> Result<(), SalesError> {
        if !self.status.can_close() {
            return Err(SalesError::ShiftAlreadyClosed);
        }

        self.status = ShiftStatus::Closed;
        self.closed_at = Some(Utc::now());
        self.closing_balance = Some(closing_balance);
        self.closing_notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Validates that the shift is open for operations
    pub fn validate_open(&self) -> Result<(), SalesError> {
        if !self.status.is_open() {
            return Err(SalesError::ShiftAlreadyClosed);
        }
        Ok(())
    }

    // =========================================================================
    // Transaction Recording
    // =========================================================================

    /// Records a cash sale
    pub fn record_cash_sale(&mut self, amount: Decimal) -> Result<(), SalesError> {
        self.validate_open()?;
        self.cash_sales += amount;
        self.expected_balance += amount;
        self.transaction_count += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Records a card sale
    pub fn record_card_sale(&mut self, amount: Decimal) -> Result<(), SalesError> {
        self.validate_open()?;
        self.card_sales += amount;
        self.transaction_count += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Records a sale with other payment method
    pub fn record_other_sale(&mut self, amount: Decimal) -> Result<(), SalesError> {
        self.validate_open()?;
        self.other_sales += amount;
        self.transaction_count += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Records a cash refund
    pub fn record_cash_refund(&mut self, amount: Decimal) -> Result<(), SalesError> {
        self.validate_open()?;
        self.refunds += amount;
        self.expected_balance -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Records cash added to drawer
    pub fn record_cash_in(&mut self, amount: Decimal) -> Result<(), SalesError> {
        self.validate_open()?;
        self.cash_in += amount;
        self.expected_balance += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Records cash removed from drawer
    pub fn record_cash_out(&mut self, amount: Decimal) -> Result<(), SalesError> {
        self.validate_open()?;
        self.cash_out += amount;
        self.expected_balance -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns the total sales for the shift
    pub fn total_sales(&self) -> Decimal {
        self.cash_sales + self.card_sales + self.other_sales
    }

    /// Returns the net sales (total - refunds)
    pub fn net_sales(&self) -> Decimal {
        self.total_sales() - self.refunds
    }

    /// Returns the cash difference (closing - expected)
    pub fn cash_difference(&self) -> Option<Decimal> {
        self.closing_balance.map(|closing| closing - self.expected_balance)
    }

    /// Returns true if the shift is open
    pub fn is_open(&self) -> bool {
        self.status.is_open()
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> ShiftId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn terminal_id(&self) -> TerminalId {
        self.terminal_id
    }

    pub fn cashier_id(&self) -> UserId {
        self.cashier_id
    }

    pub fn status(&self) -> ShiftStatus {
        self.status
    }

    pub fn opened_at(&self) -> DateTime<Utc> {
        self.opened_at
    }

    pub fn closed_at(&self) -> Option<DateTime<Utc>> {
        self.closed_at
    }

    pub fn opening_balance(&self) -> Decimal {
        self.opening_balance
    }

    pub fn closing_balance(&self) -> Option<Decimal> {
        self.closing_balance
    }

    pub fn expected_balance(&self) -> Decimal {
        self.expected_balance
    }

    pub fn cash_sales(&self) -> Decimal {
        self.cash_sales
    }

    pub fn card_sales(&self) -> Decimal {
        self.card_sales
    }

    pub fn other_sales(&self) -> Decimal {
        self.other_sales
    }

    pub fn refunds(&self) -> Decimal {
        self.refunds
    }

    pub fn cash_in(&self) -> Decimal {
        self.cash_in
    }

    pub fn cash_out(&self) -> Decimal {
        self.cash_out
    }

    pub fn transaction_count(&self) -> i32 {
        self.transaction_count
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn closing_notes(&self) -> Option<&str> {
        self.closing_notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_shift() -> CashierShift {
        CashierShift::create(
            StoreId::new(),
            TerminalId::new(),
            UserId::new(),
            dec!(100.00),
        )
        .unwrap()
    }

    #[test]
    fn test_create_shift() {
        let shift = create_test_shift();

        assert!(shift.is_open());
        assert_eq!(shift.opening_balance(), dec!(100.00));
        assert_eq!(shift.expected_balance(), dec!(100.00));
        assert_eq!(shift.transaction_count(), 0);
    }

    #[test]
    fn test_invalid_opening_balance() {
        let result = CashierShift::create(
            StoreId::new(),
            TerminalId::new(),
            UserId::new(),
            dec!(-50.00),
        );

        assert!(matches!(result, Err(SalesError::InvalidOpeningBalance)));
    }

    #[test]
    fn test_record_cash_sale() {
        let mut shift = create_test_shift();

        shift.record_cash_sale(dec!(50.00)).unwrap();

        assert_eq!(shift.cash_sales(), dec!(50.00));
        assert_eq!(shift.expected_balance(), dec!(150.00));
        assert_eq!(shift.transaction_count(), 1);
    }

    #[test]
    fn test_record_card_sale() {
        let mut shift = create_test_shift();

        shift.record_card_sale(dec!(75.00)).unwrap();

        assert_eq!(shift.card_sales(), dec!(75.00));
        assert_eq!(shift.expected_balance(), dec!(100.00)); // Card doesn't affect cash
        assert_eq!(shift.transaction_count(), 1);
    }

    #[test]
    fn test_close_shift() {
        let mut shift = create_test_shift();
        shift.record_cash_sale(dec!(50.00)).unwrap();

        shift.close(dec!(148.00), Some("Slight shortage".to_string())).unwrap();

        assert!(!shift.is_open());
        assert_eq!(shift.closing_balance(), Some(dec!(148.00)));
        assert_eq!(shift.cash_difference(), Some(dec!(-2.00)));
    }

    #[test]
    fn test_cannot_record_on_closed_shift() {
        let mut shift = create_test_shift();
        shift.close(dec!(100.00), None).unwrap();

        let result = shift.record_cash_sale(dec!(50.00));

        assert!(matches!(result, Err(SalesError::ShiftAlreadyClosed)));
    }

    #[test]
    fn test_total_and_net_sales() {
        let mut shift = create_test_shift();
        shift.record_cash_sale(dec!(100.00)).unwrap();
        shift.record_card_sale(dec!(50.00)).unwrap();
        shift.record_other_sale(dec!(25.00)).unwrap();
        shift.record_cash_refund(dec!(10.00)).unwrap();

        assert_eq!(shift.total_sales(), dec!(175.00));
        assert_eq!(shift.net_sales(), dec!(165.00));
    }
}
