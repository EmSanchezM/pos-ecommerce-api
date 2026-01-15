// TransferItem entity - line item for stock transfers

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::value_objects::{ProductId, TransferId, VariantId};
use crate::InventoryError;

/// TransferItem entity representing a line item in a stock transfer.
/// Tracks requested, shipped, and received quantities to handle discrepancies.
///
/// Invariants:
/// - Either product_id OR variant_id must be set, but not both (XOR constraint)
/// - quantity_shipped and quantity_received are set during workflow progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItem {
    id: Uuid,
    transfer_id: TransferId,
    product_id: Option<ProductId>,
    variant_id: Option<VariantId>,
    quantity_requested: Decimal,
    quantity_shipped: Option<Decimal>,
    quantity_received: Option<Decimal>,
    unit_cost: Option<Decimal>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl TransferItem {
    /// Creates a new TransferItem for a product (not a variant)
    pub fn create_for_product(
        transfer_id: TransferId,
        product_id: ProductId,
        quantity_requested: Decimal,
        unit_cost: Option<Decimal>,
    ) -> Result<Self, InventoryError> {
        Ok(Self {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            transfer_id,
            product_id: Some(product_id),
            variant_id: None,
            quantity_requested,
            quantity_shipped: None,
            quantity_received: None,
            unit_cost,
            notes: None,
            created_at: Utc::now(),
        })
    }


    /// Creates a new TransferItem for a variant
    pub fn create_for_variant(
        transfer_id: TransferId,
        variant_id: VariantId,
        quantity_requested: Decimal,
        unit_cost: Option<Decimal>,
    ) -> Result<Self, InventoryError> {
        Ok(Self {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            transfer_id,
            product_id: None,
            variant_id: Some(variant_id),
            quantity_requested,
            quantity_shipped: None,
            quantity_received: None,
            unit_cost,
            notes: None,
            created_at: Utc::now(),
        })
    }

    /// Reconstitutes a TransferItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: Uuid,
        transfer_id: TransferId,
        product_id: Option<ProductId>,
        variant_id: Option<VariantId>,
        quantity_requested: Decimal,
        quantity_shipped: Option<Decimal>,
        quantity_received: Option<Decimal>,
        unit_cost: Option<Decimal>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, InventoryError> {
        // Validate XOR constraint
        Self::validate_product_variant_constraint(product_id, variant_id)?;
        
        Ok(Self {
            id,
            transfer_id,
            product_id,
            variant_id,
            quantity_requested,
            quantity_shipped,
            quantity_received,
            unit_cost,
            notes,
            created_at,
        })
    }

    /// Validates that exactly one of product_id or variant_id is set
    fn validate_product_variant_constraint(
        product_id: Option<ProductId>,
        variant_id: Option<VariantId>,
    ) -> Result<(), InventoryError> {
        match (product_id, variant_id) {
            (Some(_), None) | (None, Some(_)) => Ok(()),
            _ => Err(InventoryError::InvalidProductVariantConstraint),
        }
    }

    /// Records the shipped quantity (called when transfer is shipped)
    pub fn record_shipped(&mut self, quantity: Decimal) {
        self.quantity_shipped = Some(quantity);
    }

    /// Records the received quantity (called when transfer is received)
    pub fn record_received(&mut self, quantity: Decimal) {
        self.quantity_received = Some(quantity);
    }

    /// Calculates the discrepancy between shipped and received quantities
    pub fn shipping_discrepancy(&self) -> Option<Decimal> {
        match (self.quantity_shipped, self.quantity_received) {
            (Some(shipped), Some(received)) => Some(shipped - received),
            _ => None,
        }
    }

    /// Calculates the total cost of this item based on requested quantity
    pub fn total_requested_cost(&self) -> Option<Decimal> {
        self.unit_cost.map(|cost| cost * self.quantity_requested)
    }

    /// Calculates the total cost of this item based on shipped quantity
    pub fn total_shipped_cost(&self) -> Option<Decimal> {
        match (self.unit_cost, self.quantity_shipped) {
            (Some(cost), Some(qty)) => Some(cost * qty),
            _ => None,
        }
    }

    /// Calculates the total cost of this item based on received quantity
    pub fn total_received_cost(&self) -> Option<Decimal> {
        match (self.unit_cost, self.quantity_received) {
            (Some(cost), Some(qty)) => Some(cost * qty),
            _ => None,
        }
    }

    // Getters
    pub fn id(&self) -> Uuid { self.id }
    pub fn transfer_id(&self) -> TransferId { self.transfer_id }
    pub fn product_id(&self) -> Option<ProductId> { self.product_id }
    pub fn variant_id(&self) -> Option<VariantId> { self.variant_id }
    pub fn quantity_requested(&self) -> Decimal { self.quantity_requested }
    pub fn quantity_shipped(&self) -> Option<Decimal> { self.quantity_shipped }
    pub fn quantity_received(&self) -> Option<Decimal> { self.quantity_received }
    pub fn unit_cost(&self) -> Option<Decimal> { self.unit_cost }
    pub fn notes(&self) -> Option<&str> { self.notes.as_deref() }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }

    // Setters
    pub fn set_notes(&mut self, notes: Option<String>) { self.notes = notes; }
    pub fn set_unit_cost(&mut self, unit_cost: Option<Decimal>) { self.unit_cost = unit_cost; }
    pub fn set_quantity_requested(&mut self, quantity: Decimal) { self.quantity_requested = quantity; }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_for_product() {
        let transfer_id = TransferId::new();
        let product_id = ProductId::new();
        
        let item = TransferItem::create_for_product(
            transfer_id, product_id, dec!(10), Some(dec!(5.00)),
        ).unwrap();
        
        assert_eq!(item.transfer_id(), transfer_id);
        assert_eq!(item.product_id(), Some(product_id));
        assert!(item.variant_id().is_none());
        assert_eq!(item.quantity_requested(), dec!(10));
        assert_eq!(item.unit_cost(), Some(dec!(5.00)));
        assert!(item.quantity_shipped().is_none());
        assert!(item.quantity_received().is_none());
    }

    #[test]
    fn test_create_for_variant() {
        let transfer_id = TransferId::new();
        let variant_id = VariantId::new();
        
        let item = TransferItem::create_for_variant(
            transfer_id, variant_id, dec!(10), Some(dec!(5.00)),
        ).unwrap();
        
        assert!(item.product_id().is_none());
        assert_eq!(item.variant_id(), Some(variant_id));
    }

    #[test]
    fn test_record_shipped_and_received() {
        let mut item = TransferItem::create_for_product(
            TransferId::new(), ProductId::new(), dec!(10), Some(dec!(5.00)),
        ).unwrap();
        
        item.record_shipped(dec!(10));
        assert_eq!(item.quantity_shipped(), Some(dec!(10)));
        
        item.record_received(dec!(9));
        assert_eq!(item.quantity_received(), Some(dec!(9)));
    }

    #[test]
    fn test_shipping_discrepancy() {
        let mut item = TransferItem::create_for_product(
            TransferId::new(), ProductId::new(), dec!(10), Some(dec!(5.00)),
        ).unwrap();
        
        assert!(item.shipping_discrepancy().is_none());
        
        item.record_shipped(dec!(10));
        assert!(item.shipping_discrepancy().is_none());
        
        item.record_received(dec!(8));
        assert_eq!(item.shipping_discrepancy(), Some(dec!(2)));
    }

    #[test]
    fn test_total_costs() {
        let mut item = TransferItem::create_for_product(
            TransferId::new(), ProductId::new(), dec!(10), Some(dec!(5.00)),
        ).unwrap();
        
        assert_eq!(item.total_requested_cost(), Some(dec!(50.00)));
        assert!(item.total_shipped_cost().is_none());
        
        item.record_shipped(dec!(8));
        assert_eq!(item.total_shipped_cost(), Some(dec!(40.00)));
        
        item.record_received(dec!(9));
        assert_eq!(item.total_received_cost(), Some(dec!(45.00)));
    }

    #[test]
    fn test_xor_constraint_both_set() {
        let result = TransferItem::reconstitute(
            Uuid::new_v7(Timestamp::now(NoContext)),
            TransferId::new(),
            Some(ProductId::new()),
            Some(VariantId::new()),
            dec!(10), None, None, None, None, Utc::now(),
        );
        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_xor_constraint_neither_set() {
        let result = TransferItem::reconstitute(
            Uuid::new_v7(Timestamp::now(NoContext)),
            TransferId::new(),
            None, None,
            dec!(10), None, None, None, None, Utc::now(),
        );
        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_reconstitute() {
        let id = Uuid::new_v7(Timestamp::now(NoContext));
        let transfer_id = TransferId::new();
        let product_id = ProductId::new();
        
        let item = TransferItem::reconstitute(
            id, transfer_id, Some(product_id), None,
            dec!(10), Some(dec!(10)), Some(dec!(9)),
            Some(dec!(5.00)), Some("Test notes".to_string()), Utc::now(),
        ).unwrap();
        
        assert_eq!(item.id(), id);
        assert_eq!(item.transfer_id(), transfer_id);
        assert_eq!(item.product_id(), Some(product_id));
        assert_eq!(item.quantity_requested(), dec!(10));
        assert_eq!(item.quantity_shipped(), Some(dec!(10)));
        assert_eq!(item.quantity_received(), Some(dec!(9)));
        assert_eq!(item.unit_cost(), Some(dec!(5.00)));
        assert_eq!(item.notes(), Some("Test notes"));
    }
}
