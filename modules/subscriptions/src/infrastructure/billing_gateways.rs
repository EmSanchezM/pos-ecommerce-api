//! Outbound gateways used by the billing job.
//!
//! Two abstractions sit between this module and the rest of the platform:
//!
//! - `BillingInvoiceGateway` — generates a fiscal invoice for a billing cycle
//!   (delegates to `fiscal::GenerateInvoiceUseCase` in production; mockable
//!   in tests).
//! - `BillingPaymentGateway` — creates a pending payment transaction the org
//!   has to settle (delegates to `payments::ProcessOnlinePaymentUseCase`).
//!
//! The traits live in `infrastructure/` rather than `domain/` because they
//! describe outbound integrations, not domain concepts. The concrete
//! implementations are wired up in the api-gateway crate, which already
//! depends on `fiscal` and `payments` — keeping them outside this crate's
//! Cargo deps avoids a heavy transitive graph.

use async_trait::async_trait;
use uuid::Uuid;

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::domain::entities::BillingCycle;

/// Result of a successful invoice generation: the fiscal `invoice_id`.
#[derive(Debug, Clone, Copy)]
pub struct InvoiceCreated {
    pub invoice_id: Uuid,
}

/// Result of a successful payment charge: the `transaction_id` (the gateway
/// stays `pending` until a webhook confirms or rejects it).
#[derive(Debug, Clone, Copy)]
pub struct ChargeCreated {
    pub transaction_id: Uuid,
}

#[async_trait]
pub trait BillingInvoiceGateway: Send + Sync {
    /// Issues an invoice for `cycle` against `organization_id`. Returns the
    /// `invoice_id` for stamping back on the cycle.
    async fn generate_invoice(
        &self,
        organization_id: OrganizationId,
        cycle: &BillingCycle,
    ) -> Result<InvoiceCreated, SubscriptionError>;
}

#[async_trait]
pub trait BillingPaymentGateway: Send + Sync {
    /// Creates a pending charge for `cycle` (already invoiced — `invoice_id`
    /// is stamped on the cycle). Returns the gateway's `transaction_id`. The
    /// transaction settles asynchronously via webhook → `payment.confirmed`
    /// or `payment.rejected`.
    async fn create_pending_charge(
        &self,
        organization_id: OrganizationId,
        cycle: &BillingCycle,
    ) -> Result<ChargeCreated, SubscriptionError>;
}
