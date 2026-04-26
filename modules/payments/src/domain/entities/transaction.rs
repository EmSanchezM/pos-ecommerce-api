//! Transaction entity - records a single charge/refund/void against a gateway.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::PaymentsError;
use crate::domain::value_objects::{
    PaymentGatewayId, TransactionId, TransactionStatus, TransactionType,
};
use identity::{StoreId, UserId};
use sales::{PaymentId, SaleId};

/// Transaction aggregate root.
///
/// Invariants enforced by use cases:
/// - `idempotency_key` is globally unique (also enforced by DB constraint).
/// - `original_transaction_id` is required for refund / partial_refund / void.
/// - Status transitions follow `TransactionStatus` rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    id: TransactionId,
    store_id: StoreId,
    gateway_id: PaymentGatewayId,
    sale_id: SaleId,
    payment_id: Option<PaymentId>,
    transaction_type: TransactionType,
    status: TransactionStatus,
    amount: Decimal,
    currency: String,
    gateway_transaction_id: Option<String>,
    gateway_response: Option<String>,
    authorization_code: Option<String>,
    card_last_four: Option<String>,
    card_brand: Option<String>,
    failure_code: Option<String>,
    failure_message: Option<String>,
    refund_reason: Option<String>,
    original_transaction_id: Option<TransactionId>,
    idempotency_key: String,
    metadata: Option<String>,
    // Manual-confirmation flow fields:
    reference_number: Option<String>,
    confirmed_by_id: Option<UserId>,
    confirmed_at: Option<DateTime<Utc>>,
    rejected_by_id: Option<UserId>,
    rejected_at: Option<DateTime<Utc>>,
    rejection_reason: Option<String>,
    processed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Transaction {
    /// Creates a new charge transaction in `Pending` status.
    #[allow(clippy::too_many_arguments)]
    pub fn create_charge(
        store_id: StoreId,
        gateway_id: PaymentGatewayId,
        sale_id: SaleId,
        payment_id: Option<PaymentId>,
        amount: Decimal,
        currency: String,
        idempotency_key: String,
        metadata: Option<String>,
        reference_number: Option<String>,
    ) -> Result<Self, PaymentsError> {
        if amount <= Decimal::ZERO {
            return Err(PaymentsError::InvalidAmount);
        }
        let now = Utc::now();
        Ok(Self {
            id: TransactionId::new(),
            store_id,
            gateway_id,
            sale_id,
            payment_id,
            transaction_type: TransactionType::Charge,
            status: TransactionStatus::Pending,
            amount,
            currency,
            gateway_transaction_id: None,
            gateway_response: None,
            authorization_code: None,
            card_last_four: None,
            card_brand: None,
            failure_code: None,
            failure_message: None,
            refund_reason: None,
            original_transaction_id: None,
            idempotency_key,
            metadata,
            reference_number,
            confirmed_by_id: None,
            confirmed_at: None,
            rejected_by_id: None,
            rejected_at: None,
            rejection_reason: None,
            processed_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Creates a refund transaction linked to the original charge.
    #[allow(clippy::too_many_arguments)]
    pub fn create_refund(
        original: &Transaction,
        amount: Decimal,
        reason: String,
        idempotency_key: String,
        is_partial: bool,
    ) -> Result<Self, PaymentsError> {
        if amount <= Decimal::ZERO {
            return Err(PaymentsError::InvalidAmount);
        }
        if amount > original.amount {
            return Err(PaymentsError::RefundExceedsOriginal);
        }
        if !original.status.can_refund() {
            return Err(PaymentsError::CannotRefundTransaction);
        }
        let tx_type = if is_partial {
            TransactionType::PartialRefund
        } else {
            TransactionType::Refund
        };
        let now = Utc::now();
        Ok(Self {
            id: TransactionId::new(),
            store_id: original.store_id,
            gateway_id: original.gateway_id,
            sale_id: original.sale_id,
            payment_id: original.payment_id,
            transaction_type: tx_type,
            status: TransactionStatus::Pending,
            amount,
            currency: original.currency.clone(),
            gateway_transaction_id: None,
            gateway_response: None,
            authorization_code: None,
            card_last_four: original.card_last_four.clone(),
            card_brand: original.card_brand.clone(),
            failure_code: None,
            failure_message: None,
            refund_reason: Some(reason),
            original_transaction_id: Some(original.id),
            idempotency_key,
            metadata: None,
            reference_number: None,
            confirmed_by_id: None,
            confirmed_at: None,
            rejected_by_id: None,
            rejected_at: None,
            rejection_reason: None,
            processed_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: TransactionId,
        store_id: StoreId,
        gateway_id: PaymentGatewayId,
        sale_id: SaleId,
        payment_id: Option<PaymentId>,
        transaction_type: TransactionType,
        status: TransactionStatus,
        amount: Decimal,
        currency: String,
        gateway_transaction_id: Option<String>,
        gateway_response: Option<String>,
        authorization_code: Option<String>,
        card_last_four: Option<String>,
        card_brand: Option<String>,
        failure_code: Option<String>,
        failure_message: Option<String>,
        refund_reason: Option<String>,
        original_transaction_id: Option<TransactionId>,
        idempotency_key: String,
        metadata: Option<String>,
        reference_number: Option<String>,
        confirmed_by_id: Option<UserId>,
        confirmed_at: Option<DateTime<Utc>>,
        rejected_by_id: Option<UserId>,
        rejected_at: Option<DateTime<Utc>>,
        rejection_reason: Option<String>,
        processed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            gateway_id,
            sale_id,
            payment_id,
            transaction_type,
            status,
            amount,
            currency,
            gateway_transaction_id,
            gateway_response,
            authorization_code,
            card_last_four,
            card_brand,
            failure_code,
            failure_message,
            refund_reason,
            original_transaction_id,
            idempotency_key,
            metadata,
            reference_number,
            confirmed_by_id,
            confirmed_at,
            rejected_by_id,
            rejected_at,
            rejection_reason,
            processed_at,
            created_at,
            updated_at,
        }
    }

    // -------------------------------------------------------------------------
    // State transitions
    // -------------------------------------------------------------------------

    pub fn mark_processing(&mut self) {
        self.status = TransactionStatus::Processing;
        self.touch();
    }

    /// Marks the transaction as succeeded with details from the gateway.
    pub fn mark_succeeded(
        &mut self,
        gateway_transaction_id: Option<String>,
        authorization_code: Option<String>,
        card_last_four: Option<String>,
        card_brand: Option<String>,
        gateway_response: Option<String>,
    ) {
        self.status = TransactionStatus::Succeeded;
        self.gateway_transaction_id = gateway_transaction_id;
        self.authorization_code = authorization_code;
        if card_last_four.is_some() {
            self.card_last_four = card_last_four;
        }
        if card_brand.is_some() {
            self.card_brand = card_brand;
        }
        if gateway_response.is_some() {
            self.gateway_response = gateway_response;
        }
        self.processed_at = Some(Utc::now());
        self.touch();
    }

    pub fn mark_failed(
        &mut self,
        failure_code: Option<String>,
        failure_message: Option<String>,
        gateway_response: Option<String>,
    ) {
        self.status = TransactionStatus::Failed;
        self.failure_code = failure_code;
        self.failure_message = failure_message;
        if gateway_response.is_some() {
            self.gateway_response = gateway_response;
        }
        self.processed_at = Some(Utc::now());
        self.touch();
    }

    pub fn mark_cancelled(&mut self) {
        self.status = TransactionStatus::Cancelled;
        self.processed_at = Some(Utc::now());
        self.touch();
    }

    /// Attaches gateway-side identifiers WITHOUT changing status. Used by
    /// the manual flow so the transaction stays `Pending` while still
    /// carrying enough metadata for reconciliation to match it.
    pub fn attach_gateway_identifiers(
        &mut self,
        gateway_transaction_id: Option<String>,
        authorization_code: Option<String>,
        card_last_four: Option<String>,
        card_brand: Option<String>,
        gateway_response: Option<String>,
    ) {
        if gateway_transaction_id.is_some() {
            self.gateway_transaction_id = gateway_transaction_id;
        }
        if authorization_code.is_some() {
            self.authorization_code = authorization_code;
        }
        if card_last_four.is_some() {
            self.card_last_four = card_last_four;
        }
        if card_brand.is_some() {
            self.card_brand = card_brand;
        }
        if gateway_response.is_some() {
            self.gateway_response = gateway_response;
        }
        self.touch();
    }

    /// Manually confirms a pending transaction (e.g. after verifying the
    /// deposit appeared in the bank statement). Optionally records the
    /// reference number captured at confirmation time.
    pub fn confirm(
        &mut self,
        confirmed_by: UserId,
        reference_number: Option<String>,
    ) -> Result<(), PaymentsError> {
        if self.confirmed_at.is_some() {
            return Err(PaymentsError::TransactionAlreadyConfirmed(
                self.id.into_uuid(),
            ));
        }
        if self.rejected_at.is_some() {
            return Err(PaymentsError::TransactionAlreadyRejected(
                self.id.into_uuid(),
            ));
        }
        if !matches!(
            self.status,
            TransactionStatus::Pending | TransactionStatus::Processing
        ) {
            return Err(PaymentsError::TransactionNotPending(self.id.into_uuid()));
        }

        let now = Utc::now();
        self.status = TransactionStatus::Succeeded;
        self.confirmed_by_id = Some(confirmed_by);
        self.confirmed_at = Some(now);
        if let Some(reference) = reference_number {
            self.reference_number = Some(reference);
        }
        self.processed_at = Some(now);
        self.touch();
        Ok(())
    }

    /// Manually rejects a pending transaction with a reason (e.g. the deposit
    /// never appeared in the bank statement).
    pub fn reject(&mut self, rejected_by: UserId, reason: String) -> Result<(), PaymentsError> {
        if self.confirmed_at.is_some() {
            return Err(PaymentsError::TransactionAlreadyConfirmed(
                self.id.into_uuid(),
            ));
        }
        if self.rejected_at.is_some() {
            return Err(PaymentsError::TransactionAlreadyRejected(
                self.id.into_uuid(),
            ));
        }
        if !matches!(
            self.status,
            TransactionStatus::Pending | TransactionStatus::Processing
        ) {
            return Err(PaymentsError::TransactionNotPending(self.id.into_uuid()));
        }

        let now = Utc::now();
        self.status = TransactionStatus::Failed;
        self.rejected_by_id = Some(rejected_by);
        self.rejected_at = Some(now);
        self.rejection_reason = Some(reason);
        self.failure_message = self.rejection_reason.clone();
        self.processed_at = Some(now);
        self.touch();
        Ok(())
    }

    /// Marks the original charge as refunded (or partially refunded) after a
    /// successful refund transaction is recorded.
    pub fn apply_refund(&mut self, refund_amount: Decimal) -> Result<(), PaymentsError> {
        if refund_amount > self.amount {
            return Err(PaymentsError::RefundExceedsOriginal);
        }
        if refund_amount == self.amount {
            self.status = TransactionStatus::Refunded;
        } else {
            self.status = TransactionStatus::PartiallyRefunded;
        }
        self.touch();
        Ok(())
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    pub fn id(&self) -> TransactionId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn gateway_id(&self) -> PaymentGatewayId {
        self.gateway_id
    }
    pub fn sale_id(&self) -> SaleId {
        self.sale_id
    }
    pub fn payment_id(&self) -> Option<PaymentId> {
        self.payment_id
    }
    pub fn transaction_type(&self) -> TransactionType {
        self.transaction_type
    }
    pub fn status(&self) -> TransactionStatus {
        self.status
    }
    pub fn amount(&self) -> Decimal {
        self.amount
    }
    pub fn currency(&self) -> &str {
        &self.currency
    }
    pub fn gateway_transaction_id(&self) -> Option<&str> {
        self.gateway_transaction_id.as_deref()
    }
    pub fn gateway_response(&self) -> Option<&str> {
        self.gateway_response.as_deref()
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
    pub fn failure_code(&self) -> Option<&str> {
        self.failure_code.as_deref()
    }
    pub fn failure_message(&self) -> Option<&str> {
        self.failure_message.as_deref()
    }
    pub fn refund_reason(&self) -> Option<&str> {
        self.refund_reason.as_deref()
    }
    pub fn original_transaction_id(&self) -> Option<TransactionId> {
        self.original_transaction_id
    }
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }
    pub fn metadata(&self) -> Option<&str> {
        self.metadata.as_deref()
    }
    pub fn reference_number(&self) -> Option<&str> {
        self.reference_number.as_deref()
    }
    pub fn confirmed_by_id(&self) -> Option<UserId> {
        self.confirmed_by_id
    }
    pub fn confirmed_at(&self) -> Option<DateTime<Utc>> {
        self.confirmed_at
    }
    pub fn rejected_by_id(&self) -> Option<UserId> {
        self.rejected_by_id
    }
    pub fn rejected_at(&self) -> Option<DateTime<Utc>> {
        self.rejected_at
    }
    pub fn rejection_reason(&self) -> Option<&str> {
        self.rejection_reason.as_deref()
    }
    pub fn processed_at(&self) -> Option<DateTime<Utc>> {
        self.processed_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
