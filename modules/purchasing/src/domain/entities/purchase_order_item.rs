// PurchaseOrderItem entity - represents a line item in a purchase order

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{PurchaseOrderId, PurchaseOrderItemId};
use inventory::{ProductId, UnitOfMeasure, VariantId};

/// PurchaseOrderItem entity representing a line item in a purchase order.
///
/// Invariants:
/// - Quantity ordered must be positive
/// - Unit cost must be non-negative
/// - Quantity received cannot exceed quantity ordered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    id: PurchaseOrderItemId,
    purchase_order_id: PurchaseOrderId,
    line_number: i32,
    product_id: ProductId,
    variant_id: Option<VariantId>,
    description: String,
    quantity_ordered: Decimal,
    quantity_received: Decimal,
    unit_of_measure: UnitOfMeasure,
    unit_cost: Decimal,
    discount_percent: Decimal,
    tax_percent: Decimal,
    line_total: Decimal,
    notes: Option<String>,
}

impl PurchaseOrderItem {
    /// Creates a new PurchaseOrderItem
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        purchase_order_id: PurchaseOrderId,
        line_number: i32,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        description: String,
        quantity_ordered: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_cost: Decimal,
        discount_percent: Decimal,
        tax_percent: Decimal,
    ) -> Self {
        let line_total = Self::calculate_line_total(
            quantity_ordered,
            unit_cost,
            discount_percent,
            tax_percent,
        );

        Self {
            id: PurchaseOrderItemId::new(),
            purchase_order_id,
            line_number,
            product_id,
            variant_id,
            description,
            quantity_ordered,
            quantity_received: Decimal::ZERO,
            unit_of_measure,
            unit_cost,
            discount_percent,
            tax_percent,
            line_total,
            notes: None,
        }
    }

    /// Reconstitutes a PurchaseOrderItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PurchaseOrderItemId,
        purchase_order_id: PurchaseOrderId,
        line_number: i32,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        description: String,
        quantity_ordered: Decimal,
        quantity_received: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_cost: Decimal,
        discount_percent: Decimal,
        tax_percent: Decimal,
        line_total: Decimal,
        notes: Option<String>,
    ) -> Self {
        Self {
            id,
            purchase_order_id,
            line_number,
            product_id,
            variant_id,
            description,
            quantity_ordered,
            quantity_received,
            unit_of_measure,
            unit_cost,
            discount_percent,
            tax_percent,
            line_total,
            notes,
        }
    }

    /// Calculates the line total with discount and tax
    fn calculate_line_total(
        quantity: Decimal,
        unit_cost: Decimal,
        discount_percent: Decimal,
        tax_percent: Decimal,
    ) -> Decimal {
        let subtotal = quantity * unit_cost;
        let discount = subtotal * (discount_percent / Decimal::from(100));
        let after_discount = subtotal - discount;
        let tax = after_discount * (tax_percent / Decimal::from(100));
        after_discount + tax
    }

    /// Updates the line total based on current values
    fn recalculate_line_total(&mut self) {
        self.line_total = Self::calculate_line_total(
            self.quantity_ordered,
            self.unit_cost,
            self.discount_percent,
            self.tax_percent,
        );
    }

    /// Returns the pending quantity (ordered - received)
    pub fn quantity_pending(&self) -> Decimal {
        self.quantity_ordered - self.quantity_received
    }

    /// Returns true if the item is fully received
    pub fn is_fully_received(&self) -> bool {
        self.quantity_received >= self.quantity_ordered
    }

    /// Adds to the received quantity
    pub fn add_received_quantity(&mut self, quantity: Decimal) {
        self.quantity_received += quantity;
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> PurchaseOrderItemId {
        self.id
    }

    pub fn purchase_order_id(&self) -> PurchaseOrderId {
        self.purchase_order_id
    }

    pub fn line_number(&self) -> i32 {
        self.line_number
    }

    pub fn product_id(&self) -> ProductId {
        self.product_id
    }

    pub fn variant_id(&self) -> Option<VariantId> {
        self.variant_id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn quantity_ordered(&self) -> Decimal {
        self.quantity_ordered
    }

    pub fn quantity_received(&self) -> Decimal {
        self.quantity_received
    }

    pub fn unit_of_measure(&self) -> &UnitOfMeasure {
        &self.unit_of_measure
    }

    pub fn unit_cost(&self) -> Decimal {
        self.unit_cost
    }

    pub fn discount_percent(&self) -> Decimal {
        self.discount_percent
    }

    pub fn tax_percent(&self) -> Decimal {
        self.tax_percent
    }

    pub fn line_total(&self) -> Decimal {
        self.line_total
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_line_number(&mut self, line_number: i32) {
        self.line_number = line_number;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_quantity_ordered(&mut self, quantity: Decimal) {
        self.quantity_ordered = quantity;
        self.recalculate_line_total();
    }

    pub fn set_unit_cost(&mut self, cost: Decimal) {
        self.unit_cost = cost;
        self.recalculate_line_total();
    }

    pub fn set_discount_percent(&mut self, discount: Decimal) {
        self.discount_percent = discount;
        self.recalculate_line_total();
    }

    pub fn set_tax_percent(&mut self, tax: Decimal) {
        self.tax_percent = tax;
        self.recalculate_line_total();
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn create_test_item() -> PurchaseOrderItem {
        PurchaseOrderItem::create(
            PurchaseOrderId::new(),
            1,
            ProductId::new(),
            None,
            "Test Product".to_string(),
            dec!(10),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(100.00),
            dec!(0),
            dec!(15),
        )
    }

    #[test]
    fn test_create_item() {
        let item = create_test_item();

        assert_eq!(item.line_number(), 1);
        assert_eq!(item.description(), "Test Product");
        assert_eq!(item.quantity_ordered(), dec!(10));
        assert_eq!(item.quantity_received(), dec!(0));
        assert_eq!(item.unit_cost(), dec!(100.00));
        // 10 * 100 = 1000, no discount, +15% tax = 1150
        assert_eq!(item.line_total(), dec!(1150.00));
    }

    #[test]
    fn test_line_total_with_discount() {
        let item = PurchaseOrderItem::create(
            PurchaseOrderId::new(),
            1,
            ProductId::new(),
            None,
            "Test Product".to_string(),
            dec!(10),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(100.00),
            dec!(10), // 10% discount
            dec!(15), // 15% tax
        );

        // 10 * 100 = 1000
        // -10% discount = 900
        // +15% tax = 1035
        assert_eq!(item.line_total(), dec!(1035.00));
    }

    #[test]
    fn test_quantity_pending() {
        let mut item = create_test_item();
        assert_eq!(item.quantity_pending(), dec!(10));

        item.add_received_quantity(dec!(3));
        assert_eq!(item.quantity_pending(), dec!(7));
        assert!(!item.is_fully_received());

        item.add_received_quantity(dec!(7));
        assert_eq!(item.quantity_pending(), dec!(0));
        assert!(item.is_fully_received());
    }

    #[test]
    fn test_recalculate_on_update() {
        let mut item = create_test_item();
        let original_total = item.line_total();

        item.set_quantity_ordered(dec!(20));
        assert!(item.line_total() > original_total);
        // 20 * 100 * 1.15 = 2300
        assert_eq!(item.line_total(), dec!(2300.00));
    }
}
