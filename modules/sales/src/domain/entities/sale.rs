//! Sale entity - represents a POS or online sale transaction

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::entities::{Payment, SaleItem};
use crate::domain::value_objects::{
    CustomerId, DiscountType, OrderStatus, PaymentMethod, SaleId, SaleItemId, SaleStatus, SaleType,
    ShiftId,
};
use crate::SalesError;
use pos_core::TerminalId;
use identity::{StoreId, UserId};
use inventory::Currency;

/// Sale entity representing a POS or online sale transaction.
///
/// Invariants:
/// - POS sales require terminal_id, shift_id, and cashier_id
/// - Sale must have items before completing
/// - Sale must be fully paid before completing
/// - Only draft sales can be modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sale {
    id: SaleId,
    sale_number: String,
    store_id: StoreId,
    sale_type: SaleType,
    status: SaleStatus,
    order_status: Option<OrderStatus>,
    terminal_id: Option<TerminalId>,
    shift_id: Option<ShiftId>,
    cashier_id: Option<UserId>,
    customer_id: Option<CustomerId>,
    currency: Currency,
    subtotal: Decimal,
    discount_type: Option<DiscountType>,
    discount_value: Decimal,
    discount_amount: Decimal,
    tax_amount: Decimal,
    total: Decimal,
    amount_paid: Decimal,
    amount_due: Decimal,
    change_given: Decimal,
    invoice_number: Option<String>,
    invoice_date: Option<DateTime<Utc>>,
    notes: Option<String>,
    internal_notes: Option<String>,
    voided_by_id: Option<UserId>,
    voided_at: Option<DateTime<Utc>>,
    void_reason: Option<String>,
    completed_at: Option<DateTime<Utc>>,
    items: Vec<SaleItem>,
    payments: Vec<Payment>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Sale {
    /// Creates a new POS Sale
    pub fn create_pos(
        sale_number: String,
        store_id: StoreId,
        terminal_id: TerminalId,
        shift_id: ShiftId,
        cashier_id: UserId,
        currency: Currency,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: SaleId::new(),
            sale_number,
            store_id,
            sale_type: SaleType::Pos,
            status: SaleStatus::Draft,
            order_status: None,
            terminal_id: Some(terminal_id),
            shift_id: Some(shift_id),
            cashier_id: Some(cashier_id),
            customer_id: None,
            currency,
            subtotal: Decimal::ZERO,
            discount_type: None,
            discount_value: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            amount_paid: Decimal::ZERO,
            amount_due: Decimal::ZERO,
            change_given: Decimal::ZERO,
            invoice_number: None,
            invoice_date: None,
            notes: None,
            internal_notes: None,
            voided_by_id: None,
            voided_at: None,
            void_reason: None,
            completed_at: None,
            items: Vec::new(),
            payments: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new Online Sale
    pub fn create_online(sale_number: String, store_id: StoreId, currency: Currency) -> Self {
        let now = Utc::now();
        Self {
            id: SaleId::new(),
            sale_number,
            store_id,
            sale_type: SaleType::Online,
            status: SaleStatus::Draft,
            order_status: Some(OrderStatus::PendingPayment),
            terminal_id: None,
            shift_id: None,
            cashier_id: None,
            customer_id: None,
            currency,
            subtotal: Decimal::ZERO,
            discount_type: None,
            discount_value: Decimal::ZERO,
            discount_amount: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            amount_paid: Decimal::ZERO,
            amount_due: Decimal::ZERO,
            change_given: Decimal::ZERO,
            invoice_number: None,
            invoice_date: None,
            notes: None,
            internal_notes: None,
            voided_by_id: None,
            voided_at: None,
            void_reason: None,
            completed_at: None,
            items: Vec::new(),
            payments: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a Sale from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: SaleId,
        sale_number: String,
        store_id: StoreId,
        sale_type: SaleType,
        status: SaleStatus,
        order_status: Option<OrderStatus>,
        terminal_id: Option<TerminalId>,
        shift_id: Option<ShiftId>,
        cashier_id: Option<UserId>,
        customer_id: Option<CustomerId>,
        currency: Currency,
        subtotal: Decimal,
        discount_type: Option<DiscountType>,
        discount_value: Decimal,
        discount_amount: Decimal,
        tax_amount: Decimal,
        total: Decimal,
        amount_paid: Decimal,
        amount_due: Decimal,
        change_given: Decimal,
        invoice_number: Option<String>,
        invoice_date: Option<DateTime<Utc>>,
        notes: Option<String>,
        internal_notes: Option<String>,
        voided_by_id: Option<UserId>,
        voided_at: Option<DateTime<Utc>>,
        void_reason: Option<String>,
        completed_at: Option<DateTime<Utc>>,
        items: Vec<SaleItem>,
        payments: Vec<Payment>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sale_number,
            store_id,
            sale_type,
            status,
            order_status,
            terminal_id,
            shift_id,
            cashier_id,
            customer_id,
            currency,
            subtotal,
            discount_type,
            discount_value,
            discount_amount,
            tax_amount,
            total,
            amount_paid,
            amount_due,
            change_given,
            invoice_number,
            invoice_date,
            notes,
            internal_notes,
            voided_by_id,
            voided_at,
            void_reason,
            completed_at,
            items,
            payments,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Completes the sale (POS workflow)
    pub fn complete(&mut self, invoice_number: String) -> Result<(), SalesError> {
        if !self.status.can_complete() {
            return Err(SalesError::InvalidStatusTransition);
        }
        if self.items.is_empty() {
            return Err(SalesError::EmptySale);
        }
        if !self.is_fully_paid() {
            return Err(SalesError::SaleNotFullyPaid);
        }

        let now = Utc::now();
        self.status = SaleStatus::Completed;
        self.invoice_number = Some(invoice_number);
        self.invoice_date = Some(now);
        self.completed_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    /// Voids the sale
    pub fn void(&mut self, voided_by_id: UserId, reason: String) -> Result<(), SalesError> {
        if !self.status.can_void() {
            return Err(SalesError::SaleAlreadyCompleted);
        }

        let now = Utc::now();
        self.status = SaleStatus::Voided;
        self.voided_by_id = Some(voided_by_id);
        self.voided_at = Some(now);
        self.void_reason = Some(reason);
        self.updated_at = now;
        Ok(())
    }

    /// Marks the sale as returned (after credit note)
    pub fn mark_returned(&mut self) -> Result<(), SalesError> {
        if !self.status.can_return() {
            return Err(SalesError::SaleNotCompleted);
        }

        self.status = SaleStatus::Returned;
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Order Workflow Methods (E-commerce)
    // =========================================================================

    /// Marks payment as received for online order
    pub fn mark_paid(&mut self) -> Result<(), SalesError> {
        if let Some(order_status) = self.order_status {
            if !order_status.can_transition_to(OrderStatus::Paid) {
                return Err(SalesError::InvalidStatusTransition);
            }
            self.order_status = Some(OrderStatus::Paid);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SalesError::InvalidStatusTransition)
        }
    }

    /// Starts processing the order
    pub fn start_processing(&mut self) -> Result<(), SalesError> {
        if let Some(order_status) = self.order_status {
            if !order_status.can_process() {
                return Err(SalesError::OrderNotPaid);
            }
            self.order_status = Some(OrderStatus::Processing);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SalesError::InvalidStatusTransition)
        }
    }

    /// Marks the order as shipped
    pub fn ship(&mut self) -> Result<(), SalesError> {
        if let Some(order_status) = self.order_status {
            if !order_status.can_ship() {
                return Err(SalesError::OrderNotProcessing);
            }
            self.order_status = Some(OrderStatus::Shipped);
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SalesError::InvalidStatusTransition)
        }
    }

    /// Marks the order as delivered
    pub fn deliver(&mut self) -> Result<(), SalesError> {
        if let Some(order_status) = self.order_status {
            if !order_status.can_deliver() {
                return Err(SalesError::OrderNotShipped);
            }
            self.order_status = Some(OrderStatus::Delivered);
            self.status = SaleStatus::Completed;
            self.completed_at = Some(Utc::now());
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SalesError::InvalidStatusTransition)
        }
    }

    /// Cancels the order
    pub fn cancel_order(&mut self) -> Result<(), SalesError> {
        if let Some(order_status) = self.order_status {
            if !order_status.can_cancel() {
                return Err(SalesError::CannotCancelShippedOrder);
            }
            self.order_status = Some(OrderStatus::Cancelled);
            self.status = SaleStatus::Voided;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SalesError::InvalidStatusTransition)
        }
    }

    // =========================================================================
    // Item Management
    // =========================================================================

    /// Adds an item to the sale
    pub fn add_item(&mut self, item: SaleItem) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }
        self.items.push(item);
        self.recalculate_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Removes an item from the sale
    pub fn remove_item(&mut self, item_id: SaleItemId) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }
        self.items.retain(|i| i.id() != item_id);
        self.recalculate_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Adds a payment to the sale
    pub fn add_payment(&mut self, payment: Payment) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }

        let payment_amount = payment.amount();
        if payment_amount > self.amount_due {
            // For cash, this is okay (we give change)
            if payment.payment_method().is_cash() {
                self.change_given = payment_amount - self.amount_due;
            }
        }

        self.payments.push(payment);
        self.recalculate_payment_totals();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Applies a percentage discount to the entire sale
    pub fn apply_percentage_discount(&mut self, percent: Decimal) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }
        if percent < Decimal::ZERO || percent > Decimal::from(100) {
            return Err(SalesError::InvalidDiscountPercentage);
        }

        self.discount_type = Some(DiscountType::Percentage);
        self.discount_value = percent;
        self.recalculate_totals();
        Ok(())
    }

    /// Applies a fixed discount to the entire sale
    pub fn apply_fixed_discount(&mut self, amount: Decimal) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }

        self.discount_type = Some(DiscountType::Fixed);
        self.discount_value = amount;
        self.recalculate_totals();
        Ok(())
    }

    /// Recalculates all sale totals
    pub fn recalculate_totals(&mut self) {
        // Sum up item totals
        self.subtotal = self.items.iter().map(|i| i.subtotal()).sum();
        self.tax_amount = self.items.iter().map(|i| i.tax_amount()).sum();

        // Calculate sale-level discount
        self.discount_amount = match self.discount_type {
            Some(DiscountType::Percentage) => {
                self.subtotal * (self.discount_value / Decimal::from(100))
            }
            Some(DiscountType::Fixed) => self.discount_value.min(self.subtotal),
            None => self.items.iter().map(|i| i.discount_amount()).sum(),
        };

        // Calculate total
        self.total = self.subtotal - self.discount_amount + self.tax_amount;
        self.recalculate_payment_totals();
    }

    /// Recalculates payment totals
    fn recalculate_payment_totals(&mut self) {
        self.amount_paid = self
            .payments
            .iter()
            .filter(|p| p.is_successful())
            .map(|p| p.net_amount())
            .sum();

        self.amount_due = (self.total - self.amount_paid).max(Decimal::ZERO);
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns true if the sale can be edited
    pub fn is_editable(&self) -> bool {
        self.status.is_editable()
    }

    /// Returns true if the sale is fully paid
    pub fn is_fully_paid(&self) -> bool {
        self.amount_paid >= self.total
    }

    /// Returns true if this is a POS sale
    pub fn is_pos(&self) -> bool {
        self.sale_type.is_pos()
    }

    /// Returns the number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Returns the total quantity of all items
    pub fn total_quantity(&self) -> Decimal {
        self.items.iter().map(|i| i.quantity()).sum()
    }

    /// Gets the primary payment method (largest payment)
    pub fn primary_payment_method(&self) -> Option<PaymentMethod> {
        self.payments
            .iter()
            .filter(|p| p.is_successful())
            .max_by(|a, b| a.amount().cmp(&b.amount()))
            .map(|p| p.payment_method())
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> SaleId {
        self.id
    }

    pub fn sale_number(&self) -> &str {
        &self.sale_number
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn sale_type(&self) -> SaleType {
        self.sale_type
    }

    pub fn status(&self) -> SaleStatus {
        self.status
    }

    pub fn order_status(&self) -> Option<OrderStatus> {
        self.order_status
    }

    pub fn terminal_id(&self) -> Option<TerminalId> {
        self.terminal_id
    }

    pub fn shift_id(&self) -> Option<ShiftId> {
        self.shift_id
    }

    pub fn cashier_id(&self) -> Option<UserId> {
        self.cashier_id
    }

    pub fn customer_id(&self) -> Option<CustomerId> {
        self.customer_id
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn subtotal(&self) -> Decimal {
        self.subtotal
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

    pub fn tax_amount(&self) -> Decimal {
        self.tax_amount
    }

    pub fn total(&self) -> Decimal {
        self.total
    }

    pub fn amount_paid(&self) -> Decimal {
        self.amount_paid
    }

    pub fn amount_due(&self) -> Decimal {
        self.amount_due
    }

    pub fn change_given(&self) -> Decimal {
        self.change_given
    }

    pub fn invoice_number(&self) -> Option<&str> {
        self.invoice_number.as_deref()
    }

    pub fn invoice_date(&self) -> Option<DateTime<Utc>> {
        self.invoice_date
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn internal_notes(&self) -> Option<&str> {
        self.internal_notes.as_deref()
    }

    pub fn voided_by_id(&self) -> Option<UserId> {
        self.voided_by_id
    }

    pub fn voided_at(&self) -> Option<DateTime<Utc>> {
        self.voided_at
    }

    pub fn void_reason(&self) -> Option<&str> {
        self.void_reason.as_deref()
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    pub fn items(&self) -> &[SaleItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<SaleItem> {
        &mut self.items
    }

    pub fn payments(&self) -> &[Payment] {
        &self.payments
    }

    pub fn payments_mut(&mut self) -> &mut Vec<Payment> {
        &mut self.payments
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

    pub fn set_customer_id(&mut self, customer_id: Option<CustomerId>) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }
        self.customer_id = customer_id;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_notes(&mut self, notes: Option<String>) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }
        self.notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_internal_notes(&mut self, notes: Option<String>) -> Result<(), SalesError> {
        if !self.is_editable() {
            return Err(SalesError::SaleNotEditable);
        }
        self.internal_notes = notes;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::str::FromStr;
    use inventory::UnitOfMeasure;

    fn create_test_pos_sale() -> Sale {
        Sale::create_pos(
            "SALE-001".to_string(),
            StoreId::new(),
            TerminalId::new(),
            ShiftId::new(),
            UserId::new(),
            Currency::new("USD").unwrap(),
        )
    }

    fn create_test_item(sale_id: SaleId) -> SaleItem {
        use inventory::ProductId;
        SaleItem::create(
            sale_id,
            1,
            ProductId::new(),
            None,
            "SKU-001".to_string(),
            "Test Product".to_string(),
            dec!(2),
            UnitOfMeasure::from_str("unit").unwrap(),
            dec!(50.00),
            dec!(30.00),
            dec!(15),
        )
        .unwrap()
    }

    #[test]
    fn test_create_pos_sale() {
        let sale = create_test_pos_sale();

        assert!(sale.is_pos());
        assert!(sale.is_editable());
        assert_eq!(sale.status(), SaleStatus::Draft);
        assert!(sale.terminal_id().is_some());
        assert!(sale.shift_id().is_some());
        assert!(sale.cashier_id().is_some());
    }

    #[test]
    fn test_create_online_sale() {
        let sale = Sale::create_online(
            "ORDER-001".to_string(),
            StoreId::new(),
            Currency::new("USD").unwrap(),
        );

        assert!(!sale.is_pos());
        assert_eq!(sale.order_status(), Some(OrderStatus::PendingPayment));
    }

    #[test]
    fn test_add_item() {
        let mut sale = create_test_pos_sale();
        let item = create_test_item(sale.id());

        sale.add_item(item).unwrap();

        assert_eq!(sale.item_count(), 1);
        assert!(sale.total() > Decimal::ZERO);
    }

    #[test]
    fn test_complete_sale() {
        let mut sale = create_test_pos_sale();
        let item = create_test_item(sale.id());
        sale.add_item(item).unwrap();

        // Add payment
        let payment = Payment::create_cash(
            sale.id(),
            sale.total(),
            Currency::new("USD").unwrap(),
            sale.total(),
        )
        .unwrap();
        sale.add_payment(payment).unwrap();

        sale.complete("INV-001".to_string()).unwrap();

        assert_eq!(sale.status(), SaleStatus::Completed);
        assert_eq!(sale.invoice_number(), Some("INV-001"));
    }

    #[test]
    fn test_cannot_complete_unpaid() {
        let mut sale = create_test_pos_sale();
        let item = create_test_item(sale.id());
        sale.add_item(item).unwrap();

        let result = sale.complete("INV-001".to_string());

        assert!(matches!(result, Err(SalesError::SaleNotFullyPaid)));
    }

    #[test]
    fn test_void_sale() {
        let mut sale = create_test_pos_sale();

        sale.void(UserId::new(), "Customer cancelled".to_string())
            .unwrap();

        assert_eq!(sale.status(), SaleStatus::Voided);
        assert!(sale.void_reason().is_some());
    }

    #[test]
    fn test_sale_discount() {
        let mut sale = create_test_pos_sale();
        let item = create_test_item(sale.id());
        sale.add_item(item).unwrap();

        let original_total = sale.total();
        sale.apply_percentage_discount(dec!(10)).unwrap();

        assert!(sale.total() < original_total);
        assert_eq!(sale.discount_type(), Some(DiscountType::Percentage));
    }
}
