//! CartItem entity - represents a line item in a shopping cart

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{CartId, CartItemId};
use crate::SalesError;
use inventory::{ProductId, ReservationId, UnitOfMeasure, VariantId};

/// CartItem entity representing a line item in a shopping cart.
///
/// Invariants:
/// - Quantity must be positive
/// - Unit price must be non-negative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    id: CartItemId,
    cart_id: CartId,
    product_id: ProductId,
    variant_id: Option<VariantId>,
    sku: String,
    name: String,
    quantity: Decimal,
    unit_of_measure: UnitOfMeasure,
    unit_price: Decimal,
    discount_percent: Decimal,
    discount_amount: Decimal,
    tax_rate: Decimal,
    tax_amount: Decimal,
    subtotal: Decimal,
    total: Decimal,
    reservation_id: Option<ReservationId>,
    image_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CartItem {
    /// Creates a new CartItem
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        cart_id: CartId,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        sku: String,
        name: String,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_price: Decimal,
        tax_rate: Decimal,
    ) -> Result<Self, SalesError> {
        if quantity <= Decimal::ZERO {
            return Err(SalesError::InvalidQuantity);
        }
        if unit_price < Decimal::ZERO {
            return Err(SalesError::InvalidUnitPrice);
        }

        let now = Utc::now();
        let subtotal = quantity * unit_price;
        let tax_amount = subtotal * (tax_rate / Decimal::from(100));
        let total = subtotal + tax_amount;

        Ok(Self {
            id: CartItemId::new(),
            cart_id,
            product_id,
            variant_id,
            sku,
            name,
            quantity,
            unit_of_measure,
            unit_price,
            discount_percent: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            reservation_id: None,
            image_url: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a CartItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CartItemId,
        cart_id: CartId,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        sku: String,
        name: String,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_price: Decimal,
        discount_percent: Decimal,
        discount_amount: Decimal,
        tax_rate: Decimal,
        tax_amount: Decimal,
        subtotal: Decimal,
        total: Decimal,
        reservation_id: Option<ReservationId>,
        image_url: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            cart_id,
            product_id,
            variant_id,
            sku,
            name,
            quantity,
            unit_of_measure,
            unit_price,
            discount_percent,
            discount_amount,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            reservation_id,
            image_url,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Business Methods
    // =========================================================================

    /// Updates the quantity
    pub fn set_quantity(&mut self, quantity: Decimal) -> Result<(), SalesError> {
        if quantity <= Decimal::ZERO {
            return Err(SalesError::InvalidQuantity);
        }
        self.quantity = quantity;
        self.recalculate_totals();
        Ok(())
    }

    /// Updates the unit price
    pub fn set_unit_price(&mut self, unit_price: Decimal) -> Result<(), SalesError> {
        if unit_price < Decimal::ZERO {
            return Err(SalesError::InvalidUnitPrice);
        }
        self.unit_price = unit_price;
        self.recalculate_totals();
        Ok(())
    }

    /// Applies a percentage discount
    pub fn apply_discount(&mut self, percent: Decimal) -> Result<(), SalesError> {
        if percent < Decimal::ZERO || percent > Decimal::from(100) {
            return Err(SalesError::InvalidDiscountPercentage);
        }
        self.discount_percent = percent;
        self.recalculate_totals();
        Ok(())
    }

    /// Recalculates totals
    fn recalculate_totals(&mut self) {
        self.subtotal = self.quantity * self.unit_price;
        self.discount_amount = self.subtotal * (self.discount_percent / Decimal::from(100));
        let after_discount = self.subtotal - self.discount_amount;
        self.tax_amount = after_discount * (self.tax_rate / Decimal::from(100));
        self.total = after_discount + self.tax_amount;
        self.updated_at = Utc::now();
    }

    /// Sets the reservation ID
    pub fn set_reservation_id(&mut self, reservation_id: Option<ReservationId>) {
        self.reservation_id = reservation_id;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> CartItemId {
        self.id
    }

    pub fn cart_id(&self) -> CartId {
        self.cart_id
    }

    pub fn product_id(&self) -> ProductId {
        self.product_id
    }

    pub fn variant_id(&self) -> Option<VariantId> {
        self.variant_id
    }

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn unit_of_measure(&self) -> &UnitOfMeasure {
        &self.unit_of_measure
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
    }

    pub fn discount_percent(&self) -> Decimal {
        self.discount_percent
    }

    pub fn discount_amount(&self) -> Decimal {
        self.discount_amount
    }

    pub fn tax_rate(&self) -> Decimal {
        self.tax_rate
    }

    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn reservation_id(&self) -> Option<ReservationId> {
        self.reservation_id
    }

    pub fn image_url(&self) -> Option<&str> {
        self.image_url.as_deref()
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

    pub fn set_image_url(&mut self, url: Option<String>) {
        self.image_url = url;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn create_test_item() -> CartItem {
        CartItem::create(
            CartId::new(),
            ProductId::new(),
            None,
            "SKU-001".to_string(),
            "Test Product".to_string(),
            dec!(2),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(50.00),
            dec!(15),
        )
        .unwrap()
    }

    #[test]
    fn test_create_item() {
        let item = create_test_item();

        assert_eq!(item.quantity(), dec!(2));
        assert_eq!(item.unit_price(), dec!(50.00));
        assert_eq!(item.subtotal(), dec!(100.00));
        assert_eq!(item.tax_amount(), dec!(15.00)); // 15% of 100
        assert_eq!(item.total(), dec!(115.00));
    }

    #[test]
    fn test_update_quantity() {
        let mut item = create_test_item();

        item.set_quantity(dec!(5)).unwrap();

        assert_eq!(item.quantity(), dec!(5));
        assert_eq!(item.subtotal(), dec!(250.00));
    }

    #[test]
    fn test_apply_discount() {
        let mut item = create_test_item();

        item.apply_discount(dec!(10)).unwrap();

        assert_eq!(item.discount_percent(), dec!(10));
        assert_eq!(item.discount_amount(), dec!(10.00)); // 10% of 100
        assert_eq!(item.tax_amount(), dec!(13.50)); // 15% of 90
        assert_eq!(item.total(), dec!(103.50)); // 90 + 13.50
    }

    #[test]
    fn test_invalid_quantity() {
        let result = CartItem::create(
            CartId::new(),
            ProductId::new(),
            None,
            "SKU-001".to_string(),
            "Test".to_string(),
            dec!(0),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(50.00),
            dec!(15),
        );

        assert!(matches!(result, Err(SalesError::InvalidQuantity)));
    }
}
