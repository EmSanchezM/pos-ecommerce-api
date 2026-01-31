//! Cart entity - represents a shopping cart for e-commerce

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::entities::CartItem;
use crate::domain::value_objects::{CartId, CartItemId, CustomerId};
use crate::SalesError;
use identity::StoreId;
use inventory::Currency;

/// Cart entity representing a shopping cart for e-commerce.
///
/// Invariants:
/// - Cart expires after a configurable period
/// - Items can only be added to active carts
/// - Cart total is recalculated on item changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    id: CartId,
    store_id: StoreId,
    customer_id: Option<CustomerId>,
    session_id: Option<String>,
    currency: Currency,
    subtotal: Decimal,
    discount_amount: Decimal,
    tax_amount: Decimal,
    total: Decimal,
    item_count: i32,
    items: Vec<CartItem>,
    expires_at: DateTime<Utc>,
    converted_to_sale: bool,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Cart {
    /// Default cart expiration in hours
    const DEFAULT_EXPIRATION_HOURS: i64 = 24;

    /// Creates a new Cart
    pub fn create(
        store_id: StoreId,
        customer_id: Option<CustomerId>,
        session_id: Option<String>,
        currency: Currency,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(Self::DEFAULT_EXPIRATION_HOURS);

        Self {
            id: CartId::new(),
            store_id,
            customer_id,
            session_id,
            currency,
            subtotal: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            item_count: 0,
            items: Vec::new(),
            expires_at,
            converted_to_sale: false,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a Cart with custom expiration
    pub fn create_with_expiration(
        store_id: StoreId,
        customer_id: Option<CustomerId>,
        session_id: Option<String>,
        currency: Currency,
        expiration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(expiration_hours);

        Self {
            id: CartId::new(),
            store_id,
            customer_id,
            session_id,
            currency,
            subtotal: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            item_count: 0,
            items: Vec::new(),
            expires_at,
            converted_to_sale: false,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a Cart from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CartId,
        store_id: StoreId,
        customer_id: Option<CustomerId>,
        session_id: Option<String>,
        currency: Currency,
        subtotal: Decimal,
        discount_amount: Decimal,
        tax_amount: Decimal,
        total: Decimal,
        item_count: i32,
        items: Vec<CartItem>,
        expires_at: DateTime<Utc>,
        converted_to_sale: bool,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            customer_id,
            session_id,
            currency,
            subtotal,
            discount_amount,
            tax_amount,
            total,
            item_count,
            items,
            expires_at,
            converted_to_sale,
            notes,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Cart Operations
    // =========================================================================

    /// Adds an item to the cart
    pub fn add_item(&mut self, item: CartItem) -> Result<(), SalesError> {
        if self.is_expired() {
            return Err(SalesError::CartExpired);
        }

        self.items.push(item);
        self.recalculate_totals();
        self.touch();
        Ok(())
    }

    /// Updates an item quantity in the cart
    pub fn update_item_quantity(
        &mut self,
        item_id: CartItemId,
        quantity: Decimal,
    ) -> Result<(), SalesError> {
        if self.is_expired() {
            return Err(SalesError::CartExpired);
        }

        let item = self
            .items
            .iter_mut()
            .find(|i| i.id() == item_id)
            .ok_or(SalesError::CartItemNotFound(item_id.into_uuid()))?;

        item.set_quantity(quantity)?;
        self.recalculate_totals();
        self.touch();
        Ok(())
    }

    /// Removes an item from the cart
    pub fn remove_item(&mut self, item_id: CartItemId) -> Result<(), SalesError> {
        if self.is_expired() {
            return Err(SalesError::CartExpired);
        }

        let initial_len = self.items.len();
        self.items.retain(|i| i.id() != item_id);

        if self.items.len() == initial_len {
            return Err(SalesError::CartItemNotFound(item_id.into_uuid()));
        }

        self.recalculate_totals();
        self.touch();
        Ok(())
    }

    /// Clears all items from the cart
    pub fn clear(&mut self) -> Result<(), SalesError> {
        if self.is_expired() {
            return Err(SalesError::CartExpired);
        }

        self.items.clear();
        self.recalculate_totals();
        self.touch();
        Ok(())
    }

    /// Marks the cart as converted to a sale
    pub fn mark_converted(&mut self) {
        self.converted_to_sale = true;
        self.updated_at = Utc::now();
    }

    /// Extends the cart expiration
    pub fn extend_expiration(&mut self, hours: i64) {
        self.expires_at = Utc::now() + Duration::hours(hours);
        self.updated_at = Utc::now();
    }

    /// Associates a customer with the cart
    pub fn set_customer(&mut self, customer_id: Option<CustomerId>) {
        self.customer_id = customer_id;
        self.updated_at = Utc::now();
    }

    /// Recalculates all cart totals
    fn recalculate_totals(&mut self) {
        self.subtotal = self.items.iter().map(|i| i.subtotal()).sum();
        self.discount_amount = self.items.iter().map(|i| i.discount_amount()).sum();
        self.tax_amount = self.items.iter().map(|i| i.tax_amount()).sum();
        self.total = self.subtotal - self.discount_amount + self.tax_amount;
        self.item_count = self.items.len() as i32;
    }

    /// Updates timestamps
    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns true if the cart has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Returns true if the cart is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns true if the cart is active (not expired, not converted)
    pub fn is_active(&self) -> bool {
        !self.is_expired() && !self.converted_to_sale
    }

    /// Returns the total quantity of all items
    pub fn total_quantity(&self) -> Decimal {
        self.items.iter().map(|i| i.quantity()).sum()
    }

    /// Validates that the cart can be converted to a sale
    pub fn validate_for_checkout(&self) -> Result<(), SalesError> {
        if self.is_expired() {
            return Err(SalesError::CartExpired);
        }
        if self.is_empty() {
            return Err(SalesError::EmptyCart);
        }
        Ok(())
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> CartId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn customer_id(&self) -> Option<CustomerId> {
        self.customer_id
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
    }

    pub fn discount_amount(&self) -> Decimal {
        self.discount_amount
    }

    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn item_count(&self) -> i32 {
        self.item_count
    }

    pub fn items(&self) -> &[CartItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<CartItem> {
        &mut self.items
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn converted_to_sale(&self) -> bool {
        self.converted_to_sale
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
    use inventory::{ProductId, UnitOfMeasure};

    fn create_test_cart() -> Cart {
        Cart::create(
            StoreId::new(),
            None,
            Some("session-123".to_string()),
            Currency::new("USD").unwrap(),
        )
    }

    fn create_test_item(cart_id: CartId) -> CartItem {
        CartItem::create(
            cart_id,
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
    fn test_create_cart() {
        let cart = create_test_cart();

        assert!(cart.is_empty());
        assert!(!cart.is_expired());
        assert!(cart.is_active());
        assert_eq!(cart.total(), Decimal::ZERO);
    }

    #[test]
    fn test_add_item() {
        let mut cart = create_test_cart();
        let item = create_test_item(cart.id());

        cart.add_item(item).unwrap();

        assert!(!cart.is_empty());
        assert_eq!(cart.item_count(), 1);
        assert!(cart.total() > Decimal::ZERO);
    }

    #[test]
    fn test_remove_item() {
        let mut cart = create_test_cart();
        let item = create_test_item(cart.id());
        let item_id = item.id();
        cart.add_item(item).unwrap();

        cart.remove_item(item_id).unwrap();

        assert!(cart.is_empty());
    }

    #[test]
    fn test_clear_cart() {
        let mut cart = create_test_cart();
        cart.add_item(create_test_item(cart.id())).unwrap();
        cart.add_item(create_test_item(cart.id())).unwrap();

        cart.clear().unwrap();

        assert!(cart.is_empty());
    }

    #[test]
    fn test_validate_for_checkout() {
        let mut cart = create_test_cart();

        // Empty cart should fail
        assert!(matches!(
            cart.validate_for_checkout(),
            Err(SalesError::EmptyCart)
        ));

        // With item should succeed
        cart.add_item(create_test_item(cart.id())).unwrap();
        assert!(cart.validate_for_checkout().is_ok());
    }

    #[test]
    fn test_mark_converted() {
        let mut cart = create_test_cart();

        cart.mark_converted();

        assert!(cart.converted_to_sale());
        assert!(!cart.is_active());
    }
}
