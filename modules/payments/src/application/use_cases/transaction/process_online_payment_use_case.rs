//! ProcessOnlinePayment - charge a customer through the configured gateway.
//!
//! For remote gateways (Stripe, PayPal, …) this is a single-shot flow: call
//! the adapter, mark the transaction `Succeeded` or `Failed`. For manual
//! adapters (`requires_manual_confirmation = true`) the transaction is left
//! in `Pending` after the adapter call so a human can confirm it later via
//! `ConfirmTransactionUseCase` or the reconciliation use case.

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{ProcessOnlinePaymentCommand, TransactionResponse};
use crate::domain::entities::Transaction;
use crate::domain::repositories::{PaymentGatewayRepository, TransactionRepository};
use crate::domain::value_objects::PaymentGatewayId;
use crate::infrastructure::gateways::GatewayAdapterRegistry;
use identity::StoreId;
use sales::SaleId;

pub struct ProcessOnlinePaymentUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    registry: Arc<dyn GatewayAdapterRegistry>,
}

impl ProcessOnlinePaymentUseCase {
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
        cmd: ProcessOnlinePaymentCommand,
    ) -> Result<TransactionResponse, PaymentsError> {
        // Idempotency: if a transaction already exists for this key, return it.
        if let Some(existing) = self
            .transaction_repo
            .find_by_idempotency_key(&cmd.idempotency_key)
            .await?
        {
            return Ok(TransactionResponse::from(existing));
        }

        let store_id = StoreId::from_uuid(cmd.store_id);

        // Resolve the gateway: explicit id wins, otherwise the store default.
        let gateway = match cmd.gateway_id {
            Some(id) => self
                .gateway_repo
                .find_by_id(PaymentGatewayId::from_uuid(id))
                .await?
                .ok_or(PaymentsError::GatewayNotFound(id))?,
            None => self
                .gateway_repo
                .find_default(store_id)
                .await?
                .ok_or(PaymentsError::NoDefaultGateway(cmd.store_id))?,
        };

        if !gateway.is_active() {
            return Err(PaymentsError::GatewayNotActive(gateway.id().into_uuid()));
        }
        if !gateway.supports_method(&cmd.payment_method) {
            return Err(PaymentsError::UnsupportedPaymentMethod);
        }
        if !gateway.supports_currency(&cmd.currency) {
            return Err(PaymentsError::UnsupportedCurrency);
        }

        let adapter = self.registry.for_type(gateway.gateway_type());

        // Structured manual-payment details, when provided, become the
        // metadata payload (JSON-serialized) so the bank/COD context survives
        // the round trip through the database.
        let metadata = match cmd.manual_details.as_ref() {
            Some(details) => Some(
                details
                    .to_json()
                    .map_err(|e| PaymentsError::ProcessingFailed(e.to_string()))?,
            ),
            None => cmd.metadata.clone(),
        };

        let mut tx = Transaction::create_charge(
            store_id,
            gateway.id(),
            SaleId::from_uuid(cmd.sale_id),
            None,
            cmd.amount,
            cmd.currency.clone(),
            cmd.idempotency_key.clone(),
            metadata,
            cmd.reference_number.clone(),
        )?;

        // Persist as `pending` BEFORE calling the gateway so the row exists
        // even when the remote call panics or times out.
        self.transaction_repo.save(&tx).await?;

        let manual_flow = adapter.requires_manual_confirmation();
        if !manual_flow {
            tx.mark_processing();
            self.transaction_repo.update(&tx).await?;
        }

        let token = cmd.card_token.unwrap_or_default();
        match adapter
            .charge(cmd.amount, &cmd.currency, &token, &cmd.idempotency_key)
            .await
        {
            Ok(result) => {
                if manual_flow {
                    // Stash the gateway-side identifier so reconciliation can
                    // match it later, but leave the status as Pending until a
                    // human confirms.
                    tx.attach_gateway_identifiers(
                        Some(result.gateway_transaction_id),
                        result.authorization_code,
                        result.card_last_four,
                        result.card_brand,
                        result.raw_response,
                    );
                } else {
                    tx.mark_succeeded(
                        Some(result.gateway_transaction_id),
                        result.authorization_code,
                        result.card_last_four,
                        result.card_brand,
                        result.raw_response,
                    );
                }
            }
            Err(err) => {
                tx.mark_failed(None, Some(err.to_string()), None);
                self.transaction_repo.update(&tx).await?;
                return Err(err);
            }
        }

        self.transaction_repo.update(&tx).await?;

        Ok(TransactionResponse::from(tx))
    }
}
