//! DeliveryProvider CRUD (super-admin gated at the handler layer).

use std::str::FromStr;
use std::sync::Arc;

use uuid::Uuid;

use crate::ShippingError;
use crate::application::dtos::{
    ConfigureDeliveryProviderCommand, DeliveryProviderResponse, UpdateDeliveryProviderCommand,
};
use crate::domain::entities::DeliveryProvider;
use crate::domain::repositories::DeliveryProviderRepository;
use crate::domain::value_objects::{DeliveryProviderId, DeliveryProviderType};
use identity::StoreId;

pub struct ConfigureDeliveryProviderUseCase {
    repo: Arc<dyn DeliveryProviderRepository>,
}

impl ConfigureDeliveryProviderUseCase {
    pub fn new(repo: Arc<dyn DeliveryProviderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: ConfigureDeliveryProviderCommand,
    ) -> Result<DeliveryProviderResponse, ShippingError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let provider_type = DeliveryProviderType::from_str(&cmd.provider_type)?;

        let existing = self.repo.find_by_store(store_id).await?;
        if existing.iter().any(|p| p.name() == cmd.name) {
            return Err(ShippingError::DuplicateProviderName(cmd.name));
        }

        let provider = DeliveryProvider::create(
            store_id,
            cmd.name,
            provider_type,
            cmd.is_default,
            cmd.api_key,
            cmd.secret_key,
            cmd.merchant_id,
            cmd.is_sandbox,
            cmd.coverage_zone_ids,
            cmd.webhook_secret,
        );

        // Same lesson learned from payments: clear any previous default before
        // inserting so the partial unique index doesn't reject the row.
        if provider.is_default() {
            self.repo
                .unset_default_except(store_id, provider.id())
                .await?;
        }
        self.repo.save(&provider).await?;
        Ok(DeliveryProviderResponse::from(provider))
    }
}

pub struct UpdateDeliveryProviderUseCase {
    repo: Arc<dyn DeliveryProviderRepository>,
}

impl UpdateDeliveryProviderUseCase {
    pub fn new(repo: Arc<dyn DeliveryProviderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateDeliveryProviderCommand,
    ) -> Result<DeliveryProviderResponse, ShippingError> {
        let id = DeliveryProviderId::from_uuid(cmd.provider_id);
        let mut provider = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::DeliveryProviderNotFound(cmd.provider_id))?;

        if let Some(name) = cmd.name {
            let siblings = self.repo.find_by_store(provider.store_id()).await?;
            if siblings
                .iter()
                .any(|p| p.id() != provider.id() && p.name() == name)
            {
                return Err(ShippingError::DuplicateProviderName(name));
            }
            provider.set_name(name);
        }
        if let Some(active) = cmd.is_active {
            if active {
                provider.activate();
            } else {
                provider.deactivate();
            }
        }
        if let Some(d) = cmd.is_default {
            provider.set_default(d);
        }
        if let Some(zones) = cmd.coverage_zone_ids {
            provider.set_coverage_zones(zones);
        }
        if let Some(secret) = cmd.webhook_secret {
            provider.set_webhook_secret(secret);
        }
        if cmd.api_key.is_some()
            || cmd.secret_key.is_some()
            || cmd.merchant_id.is_some()
            || cmd.is_sandbox.is_some()
        {
            provider.set_credentials(cmd.api_key, cmd.secret_key, cmd.merchant_id, cmd.is_sandbox);
        }

        if provider.is_default() {
            self.repo
                .unset_default_except(provider.store_id(), provider.id())
                .await?;
        }
        self.repo.update(&provider).await?;
        Ok(DeliveryProviderResponse::from(provider))
    }
}

pub struct DeleteDeliveryProviderUseCase {
    repo: Arc<dyn DeliveryProviderRepository>,
}

impl DeleteDeliveryProviderUseCase {
    pub fn new(repo: Arc<dyn DeliveryProviderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), ShippingError> {
        let pid = DeliveryProviderId::from_uuid(id);
        if self.repo.find_by_id(pid).await?.is_none() {
            return Err(ShippingError::DeliveryProviderNotFound(id));
        }
        self.repo.delete(pid).await
    }
}

pub struct ListDeliveryProvidersUseCase {
    repo: Arc<dyn DeliveryProviderRepository>,
}

impl ListDeliveryProvidersUseCase {
    pub fn new(repo: Arc<dyn DeliveryProviderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
    ) -> Result<Vec<DeliveryProviderResponse>, ShippingError> {
        let providers = self
            .repo
            .find_by_store(StoreId::from_uuid(store_id))
            .await?;
        Ok(providers
            .into_iter()
            .map(DeliveryProviderResponse::from)
            .collect())
    }
}
