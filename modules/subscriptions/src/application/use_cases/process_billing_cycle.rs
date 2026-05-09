//! `process_billing_cycle` — invoked by the periodic billing job for every
//! cycle whose `period_start <= now` and `status = 'pending'` (or `trialing`).
//!
//! - `Trialing` cycles never produce an invoice; they're flipped to `Skipped`
//!   so the billing job leaves them alone going forward.
//! - `Pending` cycles call the fiscal gateway for an invoice and the payments
//!   gateway for a pending charge, then transition to `Invoiced`. The webhook
//!   subscriber later resolves Paid/Failed once the gateway confirms.

use std::sync::Arc;

use chrono::Utc;

use crate::SubscriptionError;
use crate::domain::entities::BillingCycle;
use crate::domain::repositories::{BillingCycleRepository, SubscriptionRepository};
use crate::domain::value_objects::BillingCycleStatus;
use crate::infrastructure::{BillingInvoiceGateway, BillingPaymentGateway};

pub struct ProcessBillingCycleUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
    invoice_gw: Arc<dyn BillingInvoiceGateway>,
    payment_gw: Arc<dyn BillingPaymentGateway>,
}

impl ProcessBillingCycleUseCase {
    pub fn new(
        sub_repo: Arc<dyn SubscriptionRepository>,
        cycle_repo: Arc<dyn BillingCycleRepository>,
        invoice_gw: Arc<dyn BillingInvoiceGateway>,
        payment_gw: Arc<dyn BillingPaymentGateway>,
    ) -> Self {
        Self {
            sub_repo,
            cycle_repo,
            invoice_gw,
            payment_gw,
        }
    }

    pub async fn execute(
        &self,
        cycle_id: crate::domain::value_objects::BillingCycleId,
    ) -> Result<(), SubscriptionError> {
        let mut cycle = self.cycle_repo.find_by_id(cycle_id).await?.ok_or(
            SubscriptionError::BillingCycleNotFound(cycle_id.into_uuid()),
        )?;

        match cycle.status() {
            BillingCycleStatus::Trialing => {
                cycle.mark_skipped(Utc::now())?;
                self.cycle_repo.update(&cycle).await?;
                Ok(())
            }
            BillingCycleStatus::Pending => self.invoice_and_charge(&mut cycle).await,
            other => Err(SubscriptionError::Validation(format!(
                "billing cycle {} is not processable in status {}",
                cycle.id().into_uuid(),
                other.as_str()
            ))),
        }
    }

    async fn invoice_and_charge(&self, cycle: &mut BillingCycle) -> Result<(), SubscriptionError> {
        let subscription = self
            .sub_repo
            .find_by_id(cycle.subscription_id())
            .await?
            .ok_or_else(|| {
                SubscriptionError::SubscriptionNotFound(cycle.subscription_id().into_uuid())
            })?;

        let invoice = self
            .invoice_gw
            .generate_invoice(subscription.organization_id(), cycle)
            .await?;
        let charge = self
            .payment_gw
            .create_pending_charge(subscription.organization_id(), cycle)
            .await?;

        cycle.mark_invoiced(invoice.invoice_id, charge.transaction_id, Utc::now())?;
        self.cycle_repo.update(cycle).await?;
        // TODO(events): publish `billing_cycle.invoiced`.
        Ok(())
    }
}
