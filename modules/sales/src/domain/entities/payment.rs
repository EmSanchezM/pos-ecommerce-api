//! Payment entity - represents a payment for a sale

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{PaymentId, PaymentMethod, PaymentStatus, SaleId};
use crate::SalesError;
use inventory::Currency;

/// Payment entity representing a payment transaction for a sale.
///
/// Invariants:
/// - Amount must be positive
/// - Cash payments must have amount_tendered >= amount
/// - Only completed payments can be refunded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    id: PaymentId,
    sale_id: SaleId,
    payment_method: PaymentMethod,
    status: PaymentStatus,
    amount: Decimal,
    currency: Currency,
    amount_tendered: Option<Decimal>,
    change_given: Option<Decimal>,
    reference_number: Option<String>,
    authorization_code: Option<String>,
    card_last_four: Option<String>,
    card_brand: Option<String>,
    refunded_amount: Decimal,
    refunded_at: Option<DateTime<Utc>>,
    notes: Option<String>,
    processed_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Payment {
    /// Creates a new Payment
    pub fn create(
        sale_id: SaleId,
        payment_method: PaymentMethod,
        amount: Decimal,
        currency: Currency,
    ) -> Result<Self, SalesError> {
        if amount <= Decimal::ZERO {
            return Err(SalesError::InvalidPaymentAmount);
        }

        let now = Utc::now();
        Ok(Self {
            id: PaymentId::new(),
            sale_id,
            payment_method,
            status: PaymentStatus::Pending,
            amount,
            currency,
            amount_tendered: None,
            change_given: None,
            reference_number: None,
            authorization_code: None,
            card_last_four: None,
            card_brand: None,
            refunded_amount: Decimal::ZERO,
            refunded_at: None,
            notes: None,
            processed_at: now,
            created_at: now,
            updated_at: now,
        })
    }

    /// Creates a cash payment with tendered amount
    pub fn create_cash(
        sale_id: SaleId,
        amount: Decimal,
        currency: Currency,
        amount_tendered: Decimal,
    ) -> Result<Self, SalesError> {
        if amount <= Decimal::ZERO {
            return Err(SalesError::InvalidPaymentAmount);
        }
        if amount_tendered < amount {
            return Err(SalesError::InsufficientAmountTendered);
        }

        let change = amount_tendered - amount;
        let now = Utc::now();

        Ok(Self {
            id: PaymentId::new(),
            sale_id,
            payment_method: PaymentMethod::Cash,
            status: PaymentStatus::Completed,
            amount,
            currency,
            amount_tendered: Some(amount_tendered),
            change_given: Some(change),
            reference_number: None,
            authorization_code: None,
            card_last_four: None,
            card_brand: None,
            refunded_amount: Decimal::ZERO,
            refunded_at: None,
            notes: None,
            processed_at: now,
            created_at: now,
            updated_at: now,
        })
    }

    /// Reconstitutes a Payment from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PaymentId,
        sale_id: SaleId,
        payment_method: PaymentMethod,
        status: PaymentStatus,
        amount: Decimal,
        currency: Currency,
        amount_tendered: Option<Decimal>,
        change_given: Option<Decimal>,
        reference_number: Option<String>,
        authorization_code: Option<String>,
        card_last_four: Option<String>,
        card_brand: Option<String>,
        refunded_amount: Decimal,
        refunded_at: Option<DateTime<Utc>>,
        notes: Option<String>,
        processed_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            sale_id,
            payment_method,
            status,
            amount,
            currency,
            amount_tendered,
            change_given,
            reference_number,
            authorization_code,
            card_last_four,
            card_brand,
            refunded_amount,
            refunded_at,
            notes,
            processed_at,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Workflow Methods
    // =========================================================================

    /// Marks the payment as completed
    pub fn complete(&mut self, authorization_code: Option<String>) -> Result<(), SalesError> {
        if !self.status.can_transition_to(PaymentStatus::Completed) {
            return Err(SalesError::InvalidStatusTransition);
        }

        self.status = PaymentStatus::Completed;
        self.authorization_code = authorization_code;
        self.processed_at = Utc::now();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Marks the payment as failed
    pub fn fail(&mut self, reason: Option<String>) -> Result<(), SalesError> {
        if !self.status.can_transition_to(PaymentStatus::Failed) {
            return Err(SalesError::InvalidStatusTransition);
        }

        self.status = PaymentStatus::Failed;
        self.notes = reason;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Refunds the full payment amount
    pub fn refund_full(&mut self) -> Result<(), SalesError> {
        if !self.status.can_refund() {
            return Err(SalesError::PaymentAlreadyRefunded);
        }

        self.refunded_amount = self.amount;
        self.status = PaymentStatus::Refunded;
        self.refunded_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Refunds a partial amount
    pub fn refund_partial(&mut self, refund_amount: Decimal) -> Result<(), SalesError> {
        if !self.status.can_refund() {
            return Err(SalesError::PaymentAlreadyRefunded);
        }

        let max_refundable = self.amount - self.refunded_amount;
        if refund_amount > max_refundable {
            return Err(SalesError::PaymentExceedsBalance);
        }

        self.refunded_amount += refund_amount;

        if self.refunded_amount >= self.amount {
            self.status = PaymentStatus::Refunded;
        } else {
            self.status = PaymentStatus::PartiallyRefunded;
        }

        self.refunded_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Returns the net amount (amount - refunded)
    pub fn net_amount(&self) -> Decimal {
        self.amount - self.refunded_amount
    }

    /// Returns true if the payment is successful
    pub fn is_successful(&self) -> bool {
        self.status.is_successful()
    }

    /// Sets card details for card payments
    pub fn set_card_details(&mut self, last_four: String, brand: String) {
        self.card_last_four = Some(last_four);
        self.card_brand = Some(brand);
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> PaymentId {
        self.id
    }

    pub fn sale_id(&self) -> SaleId {
        self.sale_id
    }

    pub fn payment_method(&self) -> PaymentMethod {
        self.payment_method
    }

    pub fn status(&self) -> PaymentStatus {
        self.status
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn amount_tendered(&self) -> Option<Decimal> {
        self.amount_tendered
    }

    pub fn change_given(&self) -> Option<Decimal> {
        self.change_given
    }

    pub fn reference_number(&self) -> Option<&str> {
        self.reference_number.as_deref()
    }

    pub fn authorization_code(&self) -> Option<&str> {
        self.authorization_code.as_deref()
    }

    pub fn card_last_four(&self) -> Option<&str> {
        self.card_last_four.as_deref()
    }

    pub fn card_brand(&self) -> Option<&str> {
        self.card_brand.as_deref()
    }

    pub fn refunded_amount(&self) -> Decimal {
        self.refunded_amount
    }

    pub fn refunded_at(&self) -> Option<DateTime<Utc>> {
        self.refunded_at
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn processed_at(&self) -> DateTime<Utc> {
        self.processed_at
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

    pub fn set_reference_number(&mut self, reference: Option<String>) {
        self.reference_number = reference;
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

    #[test]
    fn test_create_payment() {
        let payment = Payment::create(
            SaleId::new(),
            PaymentMethod::CreditCard,
            dec!(100.00),
            Currency::new("USD").unwrap(),
        )
        .unwrap();

        assert_eq!(payment.amount(), dec!(100.00));
        assert_eq!(payment.status(), PaymentStatus::Pending);
        assert_eq!(payment.payment_method(), PaymentMethod::CreditCard);
    }

    #[test]
    fn test_create_cash_payment() {
        let payment = Payment::create_cash(
            SaleId::new(),
            dec!(75.00),
            Currency::new("USD").unwrap(),
            dec!(100.00),
        )
        .unwrap();

        assert_eq!(payment.amount(), dec!(75.00));
        assert_eq!(payment.amount_tendered(), Some(dec!(100.00)));
        assert_eq!(payment.change_given(), Some(dec!(25.00)));
        assert_eq!(payment.status(), PaymentStatus::Completed);
    }

    #[test]
    fn test_insufficient_cash() {
        let result = Payment::create_cash(
            SaleId::new(),
            dec!(100.00),
            Currency::new("USD").unwrap(),
            dec!(50.00),
        );

        assert!(matches!(result, Err(SalesError::InsufficientAmountTendered)));
    }

    #[test]
    fn test_complete_payment() {
        let mut payment = Payment::create(
            SaleId::new(),
            PaymentMethod::CreditCard,
            dec!(100.00),
            Currency::new("USD").unwrap(),
        )
        .unwrap();

        payment.complete(Some("AUTH123".to_string())).unwrap();

        assert_eq!(payment.status(), PaymentStatus::Completed);
        assert_eq!(payment.authorization_code(), Some("AUTH123"));
    }

    #[test]
    fn test_refund_full() {
        let mut payment = Payment::create_cash(
            SaleId::new(),
            dec!(100.00),
            Currency::new("USD").unwrap(),
            dec!(100.00),
        )
        .unwrap();

        payment.refund_full().unwrap();

        assert_eq!(payment.status(), PaymentStatus::Refunded);
        assert_eq!(payment.refunded_amount(), dec!(100.00));
        assert_eq!(payment.net_amount(), dec!(0.00));
    }

    #[test]
    fn test_refund_partial() {
        let mut payment = Payment::create_cash(
            SaleId::new(),
            dec!(100.00),
            Currency::new("USD").unwrap(),
            dec!(100.00),
        )
        .unwrap();

        payment.refund_partial(dec!(30.00)).unwrap();

        assert_eq!(payment.status(), PaymentStatus::PartiallyRefunded);
        assert_eq!(payment.refunded_amount(), dec!(30.00));
        assert_eq!(payment.net_amount(), dec!(70.00));
    }
}
