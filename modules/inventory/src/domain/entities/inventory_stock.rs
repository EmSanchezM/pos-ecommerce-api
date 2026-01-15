// InventoryStock entity - stock record per store with optimistic locking

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{ProductId, StockId, VariantId};
use crate::InventoryError;
use identity::StoreId;

/// InventoryStock entity representing stock levels for a product or variant at a specific store.
/// Uses optimistic locking via version field to prevent concurrent update conflicts.
/// 
/// Invariants:
/// - Either product_id OR variant_id must be set, but not both (XOR constraint)
/// - reserved_quantity cannot exceed quantity
/// - quantity cannot be negative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryStock {
    id: StockId,
    store_id: StoreId,
    product_id: Option<ProductId>,
    variant_id: Option<VariantId>,
    quantity: Decimal,
    reserved_quantity: Decimal,
    version: i32,
    min_stock_level: Decimal,
    max_stock_level: Option<Decimal>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl InventoryStock {
    /// Creates a new InventoryStock for a product (not a variant)
    pub fn create_for_product(
        store_id: StoreId,
        product_id: ProductId,
    ) -> Result<Self, InventoryError> {
        let now = Utc::now();
        Ok(Self {
            id: StockId::new(),
            store_id,
            product_id: Some(product_id),
            variant_id: None,
            quantity: Decimal::ZERO,
            reserved_quantity: Decimal::ZERO,
            version: 1,
            min_stock_level: Decimal::ZERO,
            max_stock_level: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Creates a new InventoryStock for a variant
    pub fn create_for_variant(
        store_id: StoreId,
        variant_id: VariantId,
    ) -> Result<Self, InventoryError> {
        let now = Utc::now();
        Ok(Self {
            id: StockId::new(),
            store_id,
            product_id: None,
            variant_id: Some(variant_id),
            quantity: Decimal::ZERO,
            reserved_quantity: Decimal::ZERO,
            version: 1,
            min_stock_level: Decimal::ZERO,
            max_stock_level: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes an InventoryStock from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: StockId,
        store_id: StoreId,
        product_id: Option<ProductId>,
        variant_id: Option<VariantId>,
        quantity: Decimal,
        reserved_quantity: Decimal,
        version: i32,
        min_stock_level: Decimal,
        max_stock_level: Option<Decimal>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, InventoryError> {
        // Validate XOR constraint
        Self::validate_product_variant_constraint(product_id, variant_id)?;
        
        Ok(Self {
            id,
            store_id,
            product_id,
            variant_id,
            quantity,
            reserved_quantity,
            version,
            min_stock_level,
            max_stock_level,
            created_at,
            updated_at,
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

    /// Returns the available quantity (quantity - reserved_quantity)
    pub fn available_quantity(&self) -> Decimal {
        self.quantity - self.reserved_quantity
    }

    /// Returns true if available quantity is at or below min_stock_level
    pub fn is_low_stock(&self) -> bool {
        self.available_quantity() <= self.min_stock_level
    }

    /// Attempts to reserve quantity, returns error if insufficient available stock
    pub fn reserve(&mut self, qty: Decimal) -> Result<(), InventoryError> {
        if qty <= Decimal::ZERO {
            return Err(InventoryError::InvalidReleaseQuantity);
        }
        if qty > self.available_quantity() {
            return Err(InventoryError::InsufficientStock);
        }
        self.reserved_quantity += qty;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Releases reserved quantity
    pub fn release(&mut self, qty: Decimal) -> Result<(), InventoryError> {
        if qty <= Decimal::ZERO {
            return Err(InventoryError::InvalidReleaseQuantity);
        }
        if qty > self.reserved_quantity {
            return Err(InventoryError::InvalidReleaseQuantity);
        }
        self.reserved_quantity -= qty;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Adjusts quantity by delta (positive for increase, negative for decrease)
    pub fn adjust_quantity(&mut self, delta: Decimal) -> Result<(), InventoryError> {
        let new_qty = self.quantity + delta;
        if new_qty < Decimal::ZERO {
            return Err(InventoryError::NegativeStock);
        }
        if new_qty < self.reserved_quantity {
            return Err(InventoryError::ReservedExceedsQuantity);
        }
        self.quantity = new_qty;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Returns the current version for optimistic locking
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Increments the version number (called after successful update)
    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> StockId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn product_id(&self) -> Option<ProductId> {
        self.product_id
    }

    pub fn variant_id(&self) -> Option<VariantId> {
        self.variant_id
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn reserved_quantity(&self) -> Decimal {
        self.reserved_quantity
    }

    pub fn min_stock_level(&self) -> Decimal {
        self.min_stock_level
    }

    pub fn max_stock_level(&self) -> Option<Decimal> {
        self.max_stock_level
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

    pub fn set_min_stock_level(&mut self, level: Decimal) {
        self.min_stock_level = level;
        self.updated_at = Utc::now();
    }

    pub fn set_max_stock_level(&mut self, level: Option<Decimal>) {
        self.max_stock_level = level;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_for_product() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        
        assert_eq!(stock.store_id(), store_id);
        assert_eq!(stock.product_id(), Some(product_id));
        assert!(stock.variant_id().is_none());
        assert_eq!(stock.quantity(), Decimal::ZERO);
        assert_eq!(stock.reserved_quantity(), Decimal::ZERO);
        assert_eq!(stock.version(), 1);
    }

    #[test]
    fn test_create_for_variant() {
        let store_id = StoreId::new();
        let variant_id = VariantId::new();
        
        let stock = InventoryStock::create_for_variant(store_id, variant_id).unwrap();
        
        assert_eq!(stock.store_id(), store_id);
        assert!(stock.product_id().is_none());
        assert_eq!(stock.variant_id(), Some(variant_id));
    }

    #[test]
    fn test_available_quantity() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(30)).unwrap();
        
        assert_eq!(stock.quantity(), dec!(100));
        assert_eq!(stock.reserved_quantity(), dec!(30));
        assert_eq!(stock.available_quantity(), dec!(70));
    }

    #[test]
    fn test_is_low_stock() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.set_min_stock_level(dec!(10));
        stock.adjust_quantity(dec!(15)).unwrap();
        
        assert!(!stock.is_low_stock()); // 15 > 10
        
        stock.reserve(dec!(6)).unwrap();
        assert!(stock.is_low_stock()); // available = 9 <= 10
    }

    #[test]
    fn test_reserve_success() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        
        stock.reserve(dec!(50)).unwrap();
        assert_eq!(stock.reserved_quantity(), dec!(50));
        assert_eq!(stock.available_quantity(), dec!(50));
    }

    #[test]
    fn test_reserve_insufficient_stock() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(50)).unwrap();
        
        let result = stock.reserve(dec!(60));
        assert!(matches!(result, Err(InventoryError::InsufficientStock)));
    }

    #[test]
    fn test_release_success() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(50)).unwrap();
        
        stock.release(dec!(30)).unwrap();
        assert_eq!(stock.reserved_quantity(), dec!(20));
        assert_eq!(stock.available_quantity(), dec!(80));
    }

    #[test]
    fn test_release_too_much() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(30)).unwrap();
        
        let result = stock.release(dec!(50));
        assert!(matches!(result, Err(InventoryError::InvalidReleaseQuantity)));
    }

    #[test]
    fn test_adjust_quantity_increase() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        
        assert_eq!(stock.quantity(), dec!(100));
    }

    #[test]
    fn test_adjust_quantity_decrease() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.adjust_quantity(dec!(-30)).unwrap();
        
        assert_eq!(stock.quantity(), dec!(70));
    }

    #[test]
    fn test_adjust_quantity_negative_result() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(50)).unwrap();
        
        let result = stock.adjust_quantity(dec!(-60));
        assert!(matches!(result, Err(InventoryError::NegativeStock)));
    }

    #[test]
    fn test_adjust_quantity_below_reserved() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        stock.reserve(dec!(50)).unwrap();
        
        // Try to reduce quantity below reserved amount
        let result = stock.adjust_quantity(dec!(-60));
        assert!(matches!(result, Err(InventoryError::ReservedExceedsQuantity)));
    }

    #[test]
    fn test_version_increment() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        assert_eq!(stock.version(), 1);
        
        stock.increment_version();
        assert_eq!(stock.version(), 2);
        
        stock.increment_version();
        assert_eq!(stock.version(), 3);
    }

    #[test]
    fn test_xor_constraint_both_set() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let variant_id = VariantId::new();
        let now = Utc::now();
        
        let result = InventoryStock::reconstitute(
            StockId::new(),
            store_id,
            Some(product_id),
            Some(variant_id),
            Decimal::ZERO,
            Decimal::ZERO,
            1,
            Decimal::ZERO,
            None,
            now,
            now,
        );
        
        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_xor_constraint_neither_set() {
        let store_id = StoreId::new();
        let now = Utc::now();
        
        let result = InventoryStock::reconstitute(
            StockId::new(),
            store_id,
            None,
            None,
            Decimal::ZERO,
            Decimal::ZERO,
            1,
            Decimal::ZERO,
            None,
            now,
            now,
        );
        
        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[test]
    fn test_set_stock_levels() {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        
        stock.set_min_stock_level(dec!(10));
        assert_eq!(stock.min_stock_level(), dec!(10));
        
        stock.set_max_stock_level(Some(dec!(100)));
        assert_eq!(stock.max_stock_level(), Some(dec!(100)));
    }
}
