//! Configure (create) a payment gateway. Super-admin only at the API layer.

use std::str::FromStr;
use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{ConfigureGatewayCommand, GatewayResponse};
use crate::domain::entities::PaymentGateway;
use crate::domain::repositories::PaymentGatewayRepository;
use crate::domain::value_objects::{GatewayConfig, GatewayType};
use identity::StoreId;

pub struct ConfigureGatewayUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
}

impl ConfigureGatewayUseCase {
    pub fn new(gateway_repo: Arc<dyn PaymentGatewayRepository>) -> Self {
        Self { gateway_repo }
    }

    pub async fn execute(
        &self,
        cmd: ConfigureGatewayCommand,
    ) -> Result<GatewayResponse, PaymentsError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let gateway_type = GatewayType::from_str(&cmd.gateway_type)?;

        // Reject duplicate gateway names within the store.
        let existing = self.gateway_repo.find_by_store(store_id).await?;
        if existing.iter().any(|g| g.name() == cmd.name) {
            return Err(PaymentsError::DuplicateGatewayName(cmd.name));
        }

        // NOTE: api_key / secret_key arrive in plaintext from the request and
        // are stored as-is in this iteration. Wrap with an encryption service
        // before going to production.
        let config =
            GatewayConfig::new(cmd.api_key, cmd.secret_key, cmd.merchant_id, cmd.is_sandbox);

        let gateway = PaymentGateway::create(
            store_id,
            cmd.name,
            gateway_type,
            cmd.is_default,
            config,
            cmd.supported_methods,
            cmd.supported_currencies,
            cmd.webhook_secret,
        );

        // The DB enforces a single default per store via a partial unique
        // index, so we must clear any previous default *before* inserting
        // when this gateway claims the default slot.
        if gateway.is_default() {
            self.gateway_repo
                .unset_default_except(store_id, gateway.id())
                .await?;
        }

        self.gateway_repo.save(&gateway).await?;

        Ok(GatewayResponse::from(gateway))
    }
}
