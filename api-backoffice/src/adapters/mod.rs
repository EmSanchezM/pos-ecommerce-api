//! Outbound gateway adapters for the backoffice binary.
//!
//! v1.0 ships the Manual billing gateway only, so manual dunning triggers use
//! the same stub payment gateway as the api-gateway billing job: it stamps a
//! fresh transaction_id and leaves the charge `pending` until a webhook
//! resolves it. Mirrors `api-gateway/src/adapters/subscription_billing_stubs`.
//!
//! TODO(billing): replace with a concrete adapter delegating to
//! `payments::ProcessOnlinePaymentUseCase`, shared between both binaries.

use async_trait::async_trait;
use uuid::{NoContext, Timestamp, Uuid};

use subscriptions::{BillingCycle, BillingPaymentGateway, ChargeCreated, SubscriptionError};
use tenancy::OrganizationId;

/// Returns a fresh UUID v7 as the gateway transaction id. The webhook
/// subscriber resolves this id back to the originating dunning attempt.
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
            "[backoffice] STUB payment gateway: replace with payments::ProcessOnlinePaymentUseCase"
        );
        Ok(ChargeCreated { transaction_id })
    }
}
