//! CreditNoteItem entity - represents a line item in a credit note/return

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{CreditNoteId, CreditNoteItemId, SaleItemId};
use crate::SalesError;
use inventory::{ProductId, UnitOfMeasure, VariantId};

/// CreditNoteItem entity representing a line item in a credit note.
///
/// Invariants:
/// - Quantity must be positive
/// - Return quantity cannot exceed original sale item quantity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditNoteItem {
    id: CreditNoteItemId,
    credit_note_id: CreditNoteId,
    original_sale_item_id: SaleItemId,
    product_id: ProductId,
    variant_id: Option<VariantId>,
    sku: String,
    description: String,
    return_quantity: Decimal,
    unit_of_measure: UnitOfMeasure,
    unit_price: Decimal,
    tax_rate: Decimal,
    tax_amount: Decimal,
    subtotal: Decimal,
    total: Decimal,
    restock: bool,
    condition: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CreditNoteItem {
    /// Creates a new CreditNoteItem
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        credit_note_id: CreditNoteId,
        original_sale_item_id: SaleItemId,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        sku: String,
        description: String,
        return_quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_price: Decimal,
        tax_rate: Decimal,
    ) -> Result<Self, SalesError> {
        if return_quantity <= Decimal::ZERO {
            return Err(SalesError::InvalidQuantity);
        }

        let now = Utc::now();
        let subtotal = return_quantity * unit_price;
        let tax_amount = subtotal * (tax_rate / Decimal::from(100));
        let total = subtotal + tax_amount;

        Ok(Self {
            id: CreditNoteItemId::new(),
            credit_note_id,
            original_sale_item_id,
            product_id,
            variant_id,
            sku,
            description,
            return_quantity,
            unit_of_measure,
            unit_price,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            restock: true,
            condition: None,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a CreditNoteItem from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CreditNoteItemId,
        credit_note_id: CreditNoteId,
        original_sale_item_id: SaleItemId,
        product_id: ProductId,
        variant_id: Option<VariantId>,
        sku: String,
        description: String,
        return_quantity: Decimal,
        unit_of_measure: UnitOfMeasure,
        unit_price: Decimal,
        tax_rate: Decimal,
        tax_amount: Decimal,
        subtotal: Decimal,
        total: Decimal,
        restock: bool,
        condition: Option<String>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            credit_note_id,
            original_sale_item_id,
            product_id,
            variant_id,
            sku,
            description,
            return_quantity,
            unit_of_measure,
            unit_price,
            tax_rate,
            tax_amount,
            subtotal,
            total,
            restock,
            condition,
            notes,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Business Methods
    // =========================================================================

    /// Sets whether the item should be restocked
    pub fn set_restock(&mut self, restock: bool) {
        self.restock = restock;
        self.updated_at = Utc::now();
    }

    /// Updates the return quantity
    pub fn set_return_quantity(&mut self, quantity: Decimal) -> Result<(), SalesError> {
        if quantity <= Decimal::ZERO {
            return Err(SalesError::InvalidQuantity);
        }
        self.return_quantity = quantity;
        self.recalculate_totals();
        Ok(())
    }

    /// Recalculates totals
    fn recalculate_totals(&mut self) {
        self.subtotal = self.return_quantity * self.unit_price;
        self.tax_amount = self.subtotal * (self.tax_rate / Decimal::from(100));
        self.total = self.subtotal + self.tax_amount;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> CreditNoteItemId {
        self.id
    }

    pub fn credit_note_id(&self) -> CreditNoteId {
        self.credit_note_id
    }

    pub fn original_sale_item_id(&self) -> SaleItemId {
        self.original_sale_item_id
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

    pub fn return_quantity(&self) -> Decimal {
        self.return_quantity
    }

    pub fn unit_of_measure(&self) -> &UnitOfMeasure {
        &self.unit_of_measure
    }

    pub fn unit_price(&self) -> Decimal {
        self.unit_price
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

    pub fn restock(&self) -> bool {
        self.restock
    }

    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
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

    pub fn set_condition(&mut self, condition: Option<String>) {
        self.condition = condition;
        self.updated_at = Utc::now();
    }

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

    fn create_test_item() -> CreditNoteItem {
        CreditNoteItem::create(
            CreditNoteId::new(),
            SaleItemId::new(),
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

        assert_eq!(item.return_quantity(), dec!(2));
        assert_eq!(item.unit_price(), dec!(50.00));
        assert_eq!(item.subtotal(), dec!(100.00));
        assert_eq!(item.tax_amount(), dec!(15.00)); // 15% of 100
        assert_eq!(item.total(), dec!(115.00));
        assert!(item.restock()); // Default to restock
    }

    #[test]
    fn test_invalid_quantity() {
        let result = CreditNoteItem::create(
            CreditNoteId::new(),
            SaleItemId::new(),
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

    #[test]
    fn test_set_restock() {
        let mut item = create_test_item();

        item.set_restock(false);

        assert!(!item.restock());
    }

    #[test]
    fn test_update_quantity() {
        let mut item = create_test_item();

        item.set_return_quantity(dec!(5)).unwrap();

        assert_eq!(item.return_quantity(), dec!(5));
        assert_eq!(item.subtotal(), dec!(250.00));
    }
}
