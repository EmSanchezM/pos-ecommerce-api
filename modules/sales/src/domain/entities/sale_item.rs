//! SaleItem entity - represents a line item in a sale

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{DiscountType, SaleId, SaleItemId};
use crate::SalesError;
use inventory::{ProductId, ReservationId, UnitOfMeasure, VariantId};

/// SaleItem entity representing a line item in a sale.
///
/// Invariants:
/// - Quantity must be positive
/// - Unit price must be non-negative
/// - Line number must be positive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleItem {
    id: SaleItemId,
    sale_id: SaleId,
    line_number: i32,
    product_id: ProductId,
    variant_id: Option<VariantId>,
    sku: String,
    description: String,
    quantity: Decimal,
    unit_of_measure: UnitOfMeasure,
    unit_price: Decimal,
    unit_cost: Decimal,
    discount_type: Option<DiscountType>,
    discount_value: Decimal,
    discount_amount: Decimal,
    tax_rate: Decimal,
    tax_amount: Decimal,
    subtotal: Decimal,
    total: Decimal,
    reservation_id: Option<ReservationId>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SaleItem {
    /// Creates a new SaleItem
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        sale_id: SaleId,
        line_number: i32,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        sku: String,
        description: String,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_price: Decimal,
        unit_cost: Decimal,
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
            id: SaleItemId::new(),
            sale_id,
            line_number,
            product_id,
            variant_id,
            sku,
            description,
            quantity,
            unit_of_measure,
            unit_price,
            unit_cost,
            discount_type: None,
            discount_value: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            reservation_id: None,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a SaleItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SaleItemId,
        sale_id: SaleId,
        line_number: i32,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        sku: String,
        description: String,
        quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_price: Decimal,
        unit_cost: Decimal,
        discount_type: Option<DiscountType>,
        discount_value: Decimal,
        discount_amount: Decimal,
        tax_rate: Decimal,
        tax_amount: Decimal,
        subtotal: Decimal,
        total: Decimal,
        reservation_id: Option<ReservationId>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sale_id,
            line_number,
            product_id,
            variant_id,
            sku,
            description,
            quantity,
            unit_of_measure,
            unit_price,
            unit_cost,
            discount_type,
            discount_value,
            discount_amount,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            reservation_id,
            notes,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Business Methods
    // =========================================================================

    /// Applies a percentage discount to the item
    pub fn apply_percentage_discount(&mut self, percent: Decimal) -> Result<(), SalesError> {
        if percent < Decimal::ZERO || percent > Decimal::from(100) {
            return Err(SalesError::InvalidDiscountPercentage);
        }

        self.discount_type = Some(DiscountType::Percentage);
        self.discount_value = percent;
        self.recalculate_totals();
        Ok(())
    }

    /// Applies a fixed discount to the item
    pub fn apply_fixed_discount(&mut self, amount: Decimal) -> Result<(), SalesError> {
        if amount < Decimal::ZERO {
            return Err(SalesError::InvalidDiscountPercentage);
        }

        self.discount_type = Some(DiscountType::Fixed);
        self.discount_value = amount;
        self.recalculate_totals();
        Ok(())
    }

    /// Removes any discount from the item
    pub fn remove_discount(&mut self) {
        self.discount_type = None;
        self.discount_value = Decimal::ZERO;
        self.recalculate_totals();
    }

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

    /// Recalculates all totals based on current values
    fn recalculate_totals(&mut self) {
        self.subtotal = self.quantity * self.unit_price;

        // Calculate discount
        self.discount_amount = match self.discount_type {
            Some(DiscountType::Percentage) => {
                self.subtotal * (self.discount_value / Decimal::from(100))
            }
            Some(DiscountType::Fixed) => self.discount_value.min(self.subtotal),
            None => Decimal::ZERO,
        };

        let after_discount = self.subtotal - self.discount_amount;
        self.tax_amount = after_discount * (self.tax_rate / Decimal::from(100));
        self.total = after_discount + self.tax_amount;
        self.updated_at = Utc::now();
    }

    /// Returns the gross profit for this item
    pub fn gross_profit(&self) -> Decimal {
        let revenue = self.subtotal - self.discount_amount;
        let cost = self.quantity * self.unit_cost;
        revenue - cost
    }

    /// Sets the reservation ID
    pub fn set_reservation_id(&mut self, reservation_id: Option<ReservationId>) {
        self.reservation_id = reservation_id;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> SaleItemId {
        self.id
    }

    pub fn sale_id(&self) -> SaleId {
        self.sale_id
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

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn description(&self) -> &str {
        &self.description
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

    pub fn unit_cost(&self) -> Decimal {
        self.unit_cost
    }

    pub fn discount_type(&self) -> Option<DiscountType> {
        self.discount_type
    }

    pub fn discount_value(&self) -> Decimal {
        self.discount_value
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

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
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
    use std::str::FromStr;

    fn create_test_item() -> SaleItem {
        SaleItem::create(
            SaleId::new(),
            1,
            ProductId::new(),
            None,
            "SKU-001".to_string(),
            "Test Product".to_string(),
            dec!(2),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(100.00),
            dec!(60.00),
            dec!(15),
        )
        .unwrap()
    }

    #[test]
    fn test_create_item() {
        let item = create_test_item();

        assert_eq!(item.quantity(), dec!(2));
        assert_eq!(item.unit_price(), dec!(100.00));
        assert_eq!(item.subtotal(), dec!(200.00));
        assert_eq!(item.tax_amount(), dec!(30.00)); // 15% of 200
        assert_eq!(item.total(), dec!(230.00));
    }

    #[test]
    fn test_invalid_quantity() {
        let result = SaleItem::create(
            SaleId::new(),
            1,
            ProductId::new(),
            None,
            "SKU-001".to_string(),
            "Test".to_string(),
            dec!(0),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(100.00),
            dec!(60.00),
            dec!(15),
        );

        assert!(matches!(result, Err(SalesError::InvalidQuantity)));
    }

    #[test]
    fn test_percentage_discount() {
        let mut item = create_test_item();

        item.apply_percentage_discount(dec!(10)).unwrap();

        assert_eq!(item.discount_type(), Some(DiscountType::Percentage));
        assert_eq!(item.discount_amount(), dec!(20.00)); // 10% of 200
        assert_eq!(item.tax_amount(), dec!(27.00)); // 15% of 180
        assert_eq!(item.total(), dec!(207.00)); // 180 + 27
    }

    #[test]
    fn test_fixed_discount() {
        let mut item = create_test_item();

        item.apply_fixed_discount(dec!(25.00)).unwrap();

        assert_eq!(item.discount_type(), Some(DiscountType::Fixed));
        assert_eq!(item.discount_amount(), dec!(25.00));
        assert_eq!(item.tax_amount(), dec!(26.25)); // 15% of 175
        assert_eq!(item.total(), dec!(201.25)); // 175 + 26.25
    }

    #[test]
    fn test_gross_profit() {
        let item = create_test_item();

        // Revenue: 200, Cost: 2 * 60 = 120
        assert_eq!(item.gross_profit(), dec!(80.00));
    }
}
