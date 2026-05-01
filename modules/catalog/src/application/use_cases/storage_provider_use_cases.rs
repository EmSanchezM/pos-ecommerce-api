//! ImageStorageProvider CRUD (super-admin gated at the handler layer).

use std::str::FromStr;
use std::sync::Arc;

use uuid::Uuid;

use crate::CatalogError;
use crate::application::dtos::{
    ConfigureStorageProviderCommand, StorageProviderResponse, UpdateStorageProviderCommand,
};
use crate::domain::entities::ImageStorageProvider;
use crate::domain::repositories::ImageStorageProviderRepository;
use crate::domain::value_objects::{ImageStorageProviderId, StorageProviderType};
use identity::StoreId;

pub struct ConfigureStorageProviderUseCase {
    repo: Arc<dyn ImageStorageProviderRepository>,
}

impl ConfigureStorageProviderUseCase {
    pub fn new(repo: Arc<dyn ImageStorageProviderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: ConfigureStorageProviderCommand,
    ) -> Result<StorageProviderResponse, CatalogError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let provider_type = StorageProviderType::from_str(&cmd.provider_type)?;

        let existing = self.repo.find_by_store(store_id).await?;
        if existing.iter().any(|p| p.name() == cmd.name) {
            return Err(CatalogError::DuplicateProviderName(cmd.name));
        }

        let provider = ImageStorageProvider::create(
            store_id,
            cmd.name,
            provider_type,
            cmd.is_default,
            cmd.api_key,
            cmd.secret_key,
            cmd.config_json,
        );

        // Same lesson learned: clear default before save when this provider
        // claims the default slot (partial unique index).
        if provider.is_default() {
            self.repo
                .unset_default_except(store_id, provider.id())
                .await?;
        }
        self.repo.save(&provider).await?;
        Ok(StorageProviderResponse::from(provider))
    }
}

pub struct UpdateStorageProviderUseCase {
    repo: Arc<dyn ImageStorageProviderRepository>,
}

impl UpdateStorageProviderUseCase {
    pub fn new(repo: Arc<dyn ImageStorageProviderRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateStorageProviderCommand,
    ) -> Result<StorageProviderResponse, CatalogError> {
        let id = ImageStorageProviderId::from_uuid(cmd.provider_id);
        let mut provider = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(CatalogError::StorageProviderNotFound(cmd.provider_id))?;

        if let Some(name) = cmd.name {
            let siblings = self.repo.find_by_store(provider.store_id()).await?;
            if siblings
                .iter()
                .any(|p| p.id() != provider.id() && p.name() == name)
            {
                return Err(CatalogError::DuplicateProviderName(name));
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
        if cmd.api_key.is_some() || cmd.secret_key.is_some() || cmd.config_json.is_some() {
            provider.set_credentials(cmd.api_key, cmd.secret_key, cmd.config_json);
        }

        if provider.is_default() {
            self.repo
                .unset_default_except(provider.store_id(), provider.id())
                .await?;
        }
        self.repo.update(&provider).await?;
        Ok(StorageProviderResponse::from(provider))
    }
}

pub struct DeleteStorageProviderUseCase {
    repo: Arc<dyn ImageStorageProviderRepository>,
}

impl DeleteStorageProviderUseCase {
    pub fn new(repo: Arc<dyn ImageStorageProviderRepository>) -> Self {
        Self { repo }
    }
    pub async fn execute(&self, id: Uuid) -> Result<(), CatalogError> {
        let pid = ImageStorageProviderId::from_uuid(id);
        if self.repo.find_by_id(pid).await?.is_none() {
            return Err(CatalogError::StorageProviderNotFound(id));
        }
        self.repo.delete(pid).await
    }
}

pub struct ListStorageProvidersUseCase {
    repo: Arc<dyn ImageStorageProviderRepository>,
}

impl ListStorageProvidersUseCase {
    pub fn new(repo: Arc<dyn ImageStorageProviderRepository>) -> Self {
        Self { repo }
    }
    pub async fn execute(
        &self,
        store_id: Uuid,
    ) -> Result<Vec<StorageProviderResponse>, CatalogError> {
        let providers = self
            .repo
            .find_by_store(StoreId::from_uuid(store_id))
            .await?;
        Ok(providers
            .into_iter()
            .map(StorageProviderResponse::from)
            .collect())
    }
}
