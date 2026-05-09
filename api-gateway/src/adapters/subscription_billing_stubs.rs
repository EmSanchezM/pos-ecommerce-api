//! v1.0 stub adapters for the subscriptions billing job.
//!
//! The subscriptions module is fully wired (entities, use cases, persistence,
//! webhook subscriber). What remains for production wiring is plumbing the
//! gateway adapters into `fiscal::GenerateInvoiceUseCase` and
//! `payments::ProcessOnlinePaymentUseCase`. Because v1.0 ships with the
//! Manual gateway only (org admin pays via bank transfer reference and a
//! webhook confirms), the stubs below are sufficient to exercise the rest of
//! the flow end-to-end against a development DB.
//!
//! TODO(billing): replace with concrete adapters that:
//!   * `StubBillingInvoiceGateway::generate_invoice` →
//!     `fiscal::GenerateInvoiceUseCase::execute(GenerateInvoiceCommand { … })`
//!   * `StubBillingPaymentGateway::create_pending_charge` →
//!     `payments::ProcessOnlinePaymentUseCase::execute(ProcessOnlinePaymentCommand { … })`

use async_trait::async_trait;
use uuid::{NoContext, Timestamp, Uuid};

use subscriptions::{
    BillingCycle, BillingInvoiceGateway, BillingPaymentGateway, ChargeCreated, InvoiceCreated,
    SubscriptionError,
};
use tenancy::OrganizationId;

/// Returns a fresh UUID v7 in lieu of a real fiscal invoice id. Logs a warn
/// so the stub is impossible to ignore in production.
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
            "[subscriptions] STUB invoice gateway: replace with fiscal::GenerateInvoiceUseCase"
        );
        Ok(InvoiceCreated { invoice_id })
    }
}

/// Returns a fresh UUID v7 as the gateway transaction id. The webhook
/// subscriber resolves this id back to the originating cycle / attempt.
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
            "[subscriptions] STUB payment gateway: replace with payments::ProcessOnlinePaymentUseCase"
        );
        Ok(ChargeCreated { transaction_id })
    }
}
