//! ProcessRefund - refund (full or partial) a previous charge.

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{ProcessRefundCommand, TransactionResponse};
use crate::domain::entities::Transaction;
use crate::domain::repositories::{PaymentGatewayRepository, TransactionRepository};
use crate::domain::value_objects::TransactionId;
use crate::infrastructure::gateways::GatewayAdapterRegistry;

pub struct ProcessRefundUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    registry: Arc<dyn GatewayAdapterRegistry>,
}

impl ProcessRefundUseCase {
    pub fn new(
        gateway_repo: Arc<dyn PaymentGatewayRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
        registry: Arc<dyn GatewayAdapterRegistry>,
    ) -> Self {
        Self {
            gateway_repo,
            transaction_repo,
            registry,
        }
    }

    pub async fn execute(
        &self,
        cmd: ProcessRefundCommand,
    ) -> Result<TransactionResponse, PaymentsError> {
        if let Some(existing) = self
            .transaction_repo
            .find_by_idempotency_key(&cmd.idempotency_key)
            .await?
        {
            return Ok(TransactionResponse::from(existing));
        }

        let original_id = TransactionId::from_uuid(cmd.transaction_id);
        let mut original = self
            .transaction_repo
            .find_by_id(original_id)
            .await?
            .ok_or(PaymentsError::TransactionNotFound(cmd.transaction_id))?;

        // Look up the original gateway so we route the refund through the
        // matching adapter (Stripe → Stripe, BAC → BAC, …).
        let gateway = self
            .gateway_repo
            .find_by_id(original.gateway_id())
            .await?
            .ok_or_else(|| PaymentsError::GatewayNotFound(original.gateway_id().into_uuid()))?;
        let adapter = self.registry.for_type(gateway.gateway_type());

        let refund_amount = cmd.amount.unwrap_or(original.amount());
        let is_partial = refund_amount < original.amount();

        let mut refund = Transaction::create_refund(
            &original,
            refund_amount,
            cmd.reason.clone(),
            cmd.idempotency_key.clone(),
            is_partial,
        )?;

        self.transaction_repo.save(&refund).await?;
        refund.mark_processing();
        self.transaction_repo.update(&refund).await?;

        let gateway_tx_id = original.gateway_transaction_id().unwrap_or_default();
        match adapter
            .refund(gateway_tx_id, Some(refund_amount), &cmd.reason)
            .await
        {
            Ok(result) => {
                refund.mark_succeeded(
                    Some(result.gateway_transaction_id),
                    None,
                    None,
                    None,
                    result.raw_response,
                );
                self.transaction_repo.update(&refund).await?;

                original.apply_refund(refund_amount)?;
                self.transaction_repo.update(&original).await?;
            }
            Err(err) => {
                refund.mark_failed(None, Some(err.to_string()), None);
                self.transaction_repo.update(&refund).await?;
                return Err(err);
            }
        }

        Ok(TransactionResponse::from(refund))
    }
}
