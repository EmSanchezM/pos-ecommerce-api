//! Update an existing payment gateway. Super-admin only at the API layer.

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{GatewayResponse, UpdateGatewayCommand};
use crate::domain::repositories::PaymentGatewayRepository;
use crate::domain::value_objects::PaymentGatewayId;

pub struct UpdateGatewayUseCase {
    gateway_repo: Arc<dyn PaymentGatewayRepository>,
}

impl UpdateGatewayUseCase {
    pub fn new(gateway_repo: Arc<dyn PaymentGatewayRepository>) -> Self {
        Self { gateway_repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateGatewayCommand,
    ) -> Result<GatewayResponse, PaymentsError> {
        let gateway_id = PaymentGatewayId::from_uuid(cmd.gateway_id);
        let mut gateway = self
            .gateway_repo
            .find_by_id(gateway_id)
            .await?
            .ok_or(PaymentsError::GatewayNotFound(cmd.gateway_id))?;

        if let Some(name) = cmd.name {
            // Detect a duplicate within the same store.
            let siblings = self.gateway_repo.find_by_store(gateway.store_id()).await?;
            if siblings
                .iter()
                .any(|g| g.id() != gateway.id() && g.name() == name)
            {
                return Err(PaymentsError::DuplicateGatewayName(name));
            }
            gateway.set_name(name);
        }
        if let Some(active) = cmd.is_active {
            if active {
                gateway.activate();
            } else {
                gateway.deactivate();
            }
        }
        if let Some(default_flag) = cmd.is_default {
            gateway.set_default(default_flag);
        }
        if let Some(methods) = cmd.supported_methods {
            gateway.set_supported_methods(methods);
        }
        if let Some(currencies) = cmd.supported_currencies {
            gateway.set_supported_currencies(currencies);
        }
        if let Some(webhook_secret) = cmd.webhook_secret {
            gateway.set_webhook_secret(webhook_secret);
        }

        // Mutate credentials only when the caller explicitly supplied them.
        if cmd.api_key.is_some()
            || cmd.secret_key.is_some()
            || cmd.merchant_id.is_some()
            || cmd.is_sandbox.is_some()
        {
            let cfg = gateway.config_mut();
            if let Some(api_key) = cmd.api_key {
                cfg.set_api_key(api_key);
            }
            if let Some(secret_key) = cmd.secret_key {
                cfg.set_secret_key(secret_key);
            }
            if let Some(merchant_id) = cmd.merchant_id {
                cfg.set_merchant_id(merchant_id);
            }
            if let Some(is_sandbox) = cmd.is_sandbox {
                cfg.set_sandbox(is_sandbox);
            }
        }

        // Same ordering as ConfigureGatewayUseCase: clear the previous
        // default before persisting so the partial unique index doesn't
        // reject the update.
        if gateway.is_default() {
            self.gateway_repo
                .unset_default_except(gateway.store_id(), gateway.id())
                .await?;
        }

        self.gateway_repo.update(&gateway).await?;

        Ok(GatewayResponse::from(gateway))
    }
}
