// AdjustmentItem entity - line item for stock adjustments

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::value_objects::{AdjustmentId, StockId};

/// AdjustmentItem entity representing a line item in a stock adjustment.
/// Stores the quantity change and cost information for a specific stock record.
///
/// Fields:
/// - quantity: positive for increase, negative for decrease
/// - balance_before/balance_after: recorded when adjustment is applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentItem {
    id: Uuid,
    adjustment_id: AdjustmentId,
    stock_id: StockId,
    quantity: Decimal,
    unit_cost: Option<Decimal>,
    balance_before: Option<Decimal>,
    balance_after: Option<Decimal>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl AdjustmentItem {
    /// Creates a new AdjustmentItem
    pub fn create(
        adjustment_id: AdjustmentId,
        stock_id: StockId,
        quantity: Decimal,
        unit_cost: Option<Decimal>,
    ) -> Self {
        Self {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            adjustment_id,
            stock_id,
            quantity,
            unit_cost,
            balance_before: None,
            balance_after: None,
            notes: None,
            created_at: Utc::now(),
        }
    }

    /// Reconstitutes an AdjustmentItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        adjustment_id: AdjustmentId,
        stock_id: StockId,
        quantity: Decimal,
        unit_cost: Option<Decimal>,
        balance_before: Option<Decimal>,
        balance_after: Option<Decimal>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            adjustment_id,
            stock_id,
            quantity,
            unit_cost,
            balance_before,
            balance_after,
            notes,
            created_at,
        }
    }

    /// Records the balance before and after applying the adjustment
    pub fn record_balances(&mut self, before: Decimal, after: Decimal) {
        self.balance_before = Some(before);
        self.balance_after = Some(after);
    }

    /// Calculates the total cost impact of this item
    pub fn total_cost(&self) -> Option<Decimal> {
        self.unit_cost.map(|cost| cost * self.quantity.abs())
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn adjustment_id(&self) -> AdjustmentId {
        self.adjustment_id
    }

    pub fn stock_id(&self) -> StockId {
        self.stock_id
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn unit_cost(&self) -> Option<Decimal> {
        self.unit_cost
    }

    pub fn balance_before(&self) -> Option<Decimal> {
        self.balance_before
    }

    pub fn balance_after(&self) -> Option<Decimal> {
        self.balance_after
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }

    pub fn set_unit_cost(&mut self, unit_cost: Option<Decimal>) {
        self.unit_cost = unit_cost;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_adjustment_item() {
        let adjustment_id = AdjustmentId::new();
        let stock_id = StockId::new();
        
        let item = AdjustmentItem::create(
            adjustment_id,
            stock_id,
            dec!(-10),
            Some(dec!(5.00)),
        );
        
        assert_eq!(item.adjustment_id(), adjustment_id);
        assert_eq!(item.stock_id(), stock_id);
        assert_eq!(item.quantity(), dec!(-10));
        assert_eq!(item.unit_cost(), Some(dec!(5.00)));
        assert!(item.balance_before().is_none());
        assert!(item.balance_after().is_none());
    }

    #[test]
    fn test_record_balances() {
        let mut item = AdjustmentItem::create(
            AdjustmentId::new(),
            StockId::new(),
            dec!(-10),
            Some(dec!(5.00)),
        );
        
        item.record_balances(dec!(100), dec!(90));
        
        assert_eq!(item.balance_before(), Some(dec!(100)));
        assert_eq!(item.balance_after(), Some(dec!(90)));
    }

    #[test]
    fn test_total_cost_with_unit_cost() {
        let item = AdjustmentItem::create(
            AdjustmentId::new(),
            StockId::new(),
            dec!(-10),
            Some(dec!(5.00)),
        );
        
        // Total cost should be absolute value of quantity * unit_cost
        assert_eq!(item.total_cost(), Some(dec!(50.00)));
    }

    #[test]
    fn test_total_cost_positive_quantity() {
        let item = AdjustmentItem::create(
            AdjustmentId::new(),
            StockId::new(),
            dec!(10),
            Some(dec!(5.00)),
        );
        
        assert_eq!(item.total_cost(), Some(dec!(50.00)));
    }

    #[test]
    fn test_total_cost_without_unit_cost() {
        let item = AdjustmentItem::create(
            AdjustmentId::new(),
            StockId::new(),
            dec!(-10),
            None,
        );
        
        assert!(item.total_cost().is_none());
    }

    #[test]
    fn test_set_notes() {
        let mut item = AdjustmentItem::create(
            AdjustmentId::new(),
            StockId::new(),
            dec!(-10),
            None,
        );
        
        item.set_notes(Some("Damaged goods".to_string()));
        
        assert_eq!(item.notes(), Some("Damaged goods"));
    }

    #[test]
    fn test_reconstitute() {
        let id = Uuid::new_v7(Timestamp::now(NoContext));
        let adjustment_id = AdjustmentId::new();
        let stock_id = StockId::new();
        let now = Utc::now();
        
        let item = AdjustmentItem::reconstitute(
            id,
            adjustment_id,
            stock_id,
            dec!(-5),
            Some(dec!(10.00)),
            Some(dec!(100)),
            Some(dec!(95)),
            Some("Test notes".to_string()),
            now,
        );
        
        assert_eq!(item.id(), id);
        assert_eq!(item.adjustment_id(), adjustment_id);
        assert_eq!(item.stock_id(), stock_id);
        assert_eq!(item.quantity(), dec!(-5));
        assert_eq!(item.unit_cost(), Some(dec!(10.00)));
        assert_eq!(item.balance_before(), Some(dec!(100)));
        assert_eq!(item.balance_after(), Some(dec!(95)));
        assert_eq!(item.notes(), Some("Test notes"));
    }
}
