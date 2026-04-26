//! ImageStorageProvider - per-store image storage configuration.
//!
//! Mirrors `payments::PaymentGateway` and `shipping::DeliveryProvider`:
//! per-store credentials encrypted at rest, super-admin manages CUD,
//! adapter-specific knobs go in `config_json` (bucket, region, root_path,
//! public_base_url, etc).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{ImageStorageProviderId, StorageProviderType};
use identity::StoreId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStorageProvider {
    id: ImageStorageProviderId,
    store_id: StoreId,
    name: String,
    provider_type: StorageProviderType,
    is_active: bool,
    is_default: bool,
    api_key_encrypted: String,
    secret_key_encrypted: String,
    config_json: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ImageStorageProvider {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        name: String,
        provider_type: StorageProviderType,
        is_default: bool,
        api_key_encrypted: String,
        secret_key_encrypted: String,
        config_json: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ImageStorageProviderId::new(),
            store_id,
            name,
            provider_type,
            is_active: true,
            is_default,
            api_key_encrypted,
            secret_key_encrypted,
            config_json,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ImageStorageProviderId,
        store_id: StoreId,
        name: String,
        provider_type: StorageProviderType,
        is_active: bool,
        is_default: bool,
        api_key_encrypted: String,
        secret_key_encrypted: String,
        config_json: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            provider_type,
            is_active,
            is_default,
            api_key_encrypted,
            secret_key_encrypted,
            config_json,
            created_at,
            updated_at,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.touch();
    }
    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
        self.touch();
    }
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }
    pub fn set_credentials(
        &mut self,
        api_key: Option<String>,
        secret_key: Option<String>,
        config_json: Option<Option<String>>,
    ) {
        if let Some(k) = api_key {
            self.api_key_encrypted = k;
        }
        if let Some(s) = secret_key {
            self.secret_key_encrypted = s;
        }
        if let Some(c) = config_json {
            self.config_json = c;
        }
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ImageStorageProviderId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn provider_type(&self) -> StorageProviderType {
        self.provider_type
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn is_default(&self) -> bool {
        self.is_default
    }
    pub fn api_key_encrypted(&self) -> &str {
        &self.api_key_encrypted
    }
    pub fn secret_key_encrypted(&self) -> &str {
        &self.secret_key_encrypted
    }
    pub fn config_json(&self) -> Option<&str> {
        self.config_json.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
