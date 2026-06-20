//! Outbound gateway adapters for the subscriptions billing flow.
//!
//! `subscriptions` defines the `BillingInvoiceGateway` / `BillingPaymentGateway`
//! traits but deliberately does NOT depend on `fiscal` / `payments` (keeps its
//! transitive graph light). The concrete adapters live here instead, in a small
//! crate that BOTH binaries share — the api-gateway billing job and the
//! api-backoffice manual dunning trigger — so the implementation is written
//! once rather than copy-pasted per binary.
//!
//! v1.0 ships the Manual gateway only (org admin pays via bank-transfer
//! reference; a webhook later confirms), so these stubs stamp a fresh
//! `transaction_id` and leave the charge `pending`. They are sufficient to
//! exercise the whole subscription flow end-to-end against a dev DB.
//!
//! TODO(billing): replace with concrete adapters that delegate to
//! `fiscal::GenerateInvoiceUseCase` and `payments::ProcessOnlinePaymentUseCase`.
//! Note: that integration must first resolve a domain mismatch — the payments
//! module is store-scoped + token-based (POS), while subscription billing is
//! org-level and recurring — so it is a billing feature in its own right, not a
//! drop-in. When it lands, only this crate changes; both binaries pick it up.

use async_trait::async_trait;
use uuid::{NoContext, Timestamp, Uuid};

use subscriptions::{
    BillingCycle, BillingInvoiceGateway, BillingPaymentGateway, ChargeCreated, InvoiceCreated,
    SubscriptionError,
};
use tenancy::OrganizationId;

/// Returns a fresh UUID v7 in lieu of a real fiscal invoice id. Logs a warn so
/// the stub is impossible to ignore in production.
pub struct StubBillingInvoiceGateway;

impl StubBillingInvoiceGateway {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubBillingInvoiceGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BillingInvoiceGateway for StubBillingInvoiceGateway {
    async fn generate_invoice(
        &self,
        organization_id: OrganizationId,
        cycle: &BillingCycle,
    ) -> Result<InvoiceCreated, SubscriptionError> {
        let invoice_id = Uuid::new_v7(Timestamp::now(NoContext));
        tracing::warn!(
            organization_id = %organization_id.into_uuid(),
            cycle_id = %cycle.id().into_uuid(),
            invoice_id = %invoice_id,
            "[subscription_billing] STUB invoice gateway: replace with fiscal::GenerateInvoiceUseCase"
        );
        Ok(InvoiceCreated { invoice_id })
    }
}

/// Returns a fresh UUID v7 as the gateway transaction id. The webhook subscriber
/// resolves this id back to the originating cycle / dunning attempt.
pub struct StubBillingPaymentGateway;

impl StubBillingPaymentGateway {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubBillingPaymentGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BillingPaymentGateway for StubBillingPaymentGateway {
    async fn create_pending_charge(
        &self,
        organization_id: OrganizationId,
        cycle: &BillingCycle,
    ) -> Result<ChargeCreated, SubscriptionError> {
        let transaction_id = Uuid::new_v7(Timestamp::now(NoContext));
        tracing::warn!(
            organization_id = %organization_id.into_uuid(),
            cycle_id = %cycle.id().into_uuid(),
            transaction_id = %transaction_id,
            "[subscription_billing] STUB payment gateway: replace with payments::ProcessOnlinePaymentUseCase"
        );
        Ok(ChargeCreated { transaction_id })
    }
}
