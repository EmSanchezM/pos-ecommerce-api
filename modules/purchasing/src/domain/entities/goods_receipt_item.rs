// GoodsReceiptItem entity - represents a line item in a goods receipt

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{GoodsReceiptId, GoodsReceiptItemId, PurchaseOrderItemId};
use inventory::{ProductId, VariantId};

/// GoodsReceiptItem entity representing a line item in a goods receipt.
///
/// Invariants:
/// - Quantity received must be positive
/// - Unit cost must be non-negative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodsReceiptItem {
    id: GoodsReceiptItemId,
    goods_receipt_id: GoodsReceiptId,
    purchase_order_item_id: PurchaseOrderItemId,
    product_id: ProductId,
    variant_id: Option<VariantId>,
    quantity_received: Decimal,
    unit_cost: Decimal,
    lot_number: Option<String>,
    expiry_date: Option<NaiveDate>,
    notes: Option<String>,
}

impl GoodsReceiptItem {
    /// Creates a new GoodsReceiptItem
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        goods_receipt_id: GoodsReceiptId,
        purchase_order_item_id: PurchaseOrderItemId,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        quantity_received: Decimal,
        unit_cost: Decimal,
    ) -> Self {
        Self {
            id: GoodsReceiptItemId::new(),
            goods_receipt_id,
            purchase_order_item_id,
            product_id,
            variant_id,
            quantity_received,
            unit_cost,
            lot_number: None,
            expiry_date: None,
            notes: None,
        }
    }

    /// Reconstitutes a GoodsReceiptItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: GoodsReceiptItemId,
        goods_receipt_id: GoodsReceiptId,
        purchase_order_item_id: PurchaseOrderItemId,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        quantity_received: Decimal,
        unit_cost: Decimal,
        lot_number: Option<String>,
        expiry_date: Option<NaiveDate>,
        notes: Option<String>,
    ) -> Self {
        Self {
            id,
            goods_receipt_id,
            purchase_order_item_id,
            product_id,
            variant_id,
            quantity_received,
            unit_cost,
            lot_number,
            expiry_date,
            notes,
        }
    }

    /// Calculates the total value of this receipt item
    pub fn total_value(&self) -> Decimal {
        self.quantity_received * self.unit_cost
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> GoodsReceiptItemId {
        self.id
    }

    pub fn goods_receipt_id(&self) -> GoodsReceiptId {
        self.goods_receipt_id
    }

    pub fn purchase_order_item_id(&self) -> PurchaseOrderItemId {
        self.purchase_order_item_id
    }

    pub fn product_id(&self) -> ProductId {
        self.product_id
    }

    pub fn variant_id(&self) -> Option<VariantId> {
        self.variant_id
    }

    pub fn quantity_received(&self) -> Decimal {
        self.quantity_received
    }

    pub fn unit_cost(&self) -> Decimal {
        self.unit_cost
    }

    pub fn lot_number(&self) -> Option<&str> {
        self.lot_number.as_deref()
    }

    pub fn expiry_date(&self) -> Option<NaiveDate> {
        self.expiry_date
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_quantity_received(&mut self, quantity: Decimal) {
        self.quantity_received = quantity;
    }

    pub fn set_unit_cost(&mut self, cost: Decimal) {
        self.unit_cost = cost;
    }

    pub fn set_lot_number(&mut self, lot_number: Option<String>) {
        self.lot_number = lot_number;
    }

    pub fn set_expiry_date(&mut self, expiry_date: Option<NaiveDate>) {
        self.expiry_date = expiry_date;
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_item() {
        let item = GoodsReceiptItem::create(
            GoodsReceiptId::new(),
            PurchaseOrderItemId::new(),
            ProductId::new(),
            None,
            dec!(10),
            dec!(100.00),
        );

        assert_eq!(item.quantity_received(), dec!(10));
        assert_eq!(item.unit_cost(), dec!(100.00));
        assert_eq!(item.total_value(), dec!(1000.00));
    }

    #[test]
    fn test_total_value() {
        let item = GoodsReceiptItem::create(
            GoodsReceiptId::new(),
            PurchaseOrderItemId::new(),
            ProductId::new(),
            None,
            dec!(5),
            dec!(25.50),
        );

        assert_eq!(item.total_value(), dec!(127.50));
    }

    #[test]
    fn test_setters() {
        let mut item = GoodsReceiptItem::create(
            GoodsReceiptId::new(),
            PurchaseOrderItemId::new(),
            ProductId::new(),
            None,
            dec!(10),
            dec!(100.00),
        );

        item.set_lot_number(Some("LOT-001".to_string()));
        item.set_expiry_date(Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()));
        item.set_notes(Some("Test notes".to_string()));

        assert_eq!(item.lot_number(), Some("LOT-001"));
        assert_eq!(
            item.expiry_date(),
            Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
        );
        assert_eq!(item.notes(), Some("Test notes"));
    }
}
