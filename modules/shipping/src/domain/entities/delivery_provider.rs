//! DeliveryProvider - per-store config for an external courier.
//!
//! Mirrors `payments::PaymentGateway`: super-admin manages CUD; credentials
//! are stored encrypted; mutating routes are gated at the handler layer.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{DeliveryProviderId, DeliveryProviderType, ShippingZoneId};
use identity::StoreId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryProvider {
    id: DeliveryProviderId,
    store_id: StoreId,
    name: String,
    provider_type: DeliveryProviderType,
    is_active: bool,
    is_default: bool,
    api_key_encrypted: String,
    secret_key_encrypted: String,
    merchant_id: Option<String>,
    is_sandbox: bool,
    coverage_zone_ids: Vec<Uuid>,
    webhook_secret: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DeliveryProvider {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        name: String,
        provider_type: DeliveryProviderType,
        is_default: bool,
        api_key_encrypted: String,
        secret_key_encrypted: String,
        merchant_id: Option<String>,
        is_sandbox: bool,
        coverage_zone_ids: Vec<Uuid>,
        webhook_secret: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DeliveryProviderId::new(),
            store_id,
            name,
            provider_type,
            is_active: true,
            is_default,
            api_key_encrypted,
            secret_key_encrypted,
            merchant_id,
            is_sandbox,
            coverage_zone_ids,
            webhook_secret,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: DeliveryProviderId,
        store_id: StoreId,
        name: String,
        provider_type: DeliveryProviderType,
        is_active: bool,
        is_default: bool,
        api_key_encrypted: String,
        secret_key_encrypted: String,
        merchant_id: Option<String>,
        is_sandbox: bool,
        coverage_zone_ids: Vec<Uuid>,
        webhook_secret: Option<String>,
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
            merchant_id,
            is_sandbox,
            coverage_zone_ids,
            webhook_secret,
            created_at,
            updated_at,
        }
    }

    pub fn covers_zone(&self, zone_id: ShippingZoneId) -> bool {
        // Empty coverage_zone_ids = covers all zones in the store.
        self.coverage_zone_ids.is_empty() || self.coverage_zone_ids.contains(&zone_id.into_uuid())
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
        merchant_id: Option<Option<String>>,
        is_sandbox: Option<bool>,
    ) {
        if let Some(k) = api_key {
            self.api_key_encrypted = k;
        }
        if let Some(s) = secret_key {
            self.secret_key_encrypted = s;
        }
        if let Some(m) = merchant_id {
            self.merchant_id = m;
        }
        if let Some(b) = is_sandbox {
            self.is_sandbox = b;
        }
        self.touch();
    }
    pub fn set_coverage_zones(&mut self, zones: Vec<Uuid>) {
        self.coverage_zone_ids = zones;
        self.touch();
    }
    pub fn set_webhook_secret(&mut self, secret: Option<String>) {
        self.webhook_secret = secret;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> DeliveryProviderId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn provider_type(&self) -> DeliveryProviderType {
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
    pub fn merchant_id(&self) -> Option<&str> {
        self.merchant_id.as_deref()
    }
    pub fn is_sandbox(&self) -> bool {
        self.is_sandbox
    }
    pub fn coverage_zone_ids(&self) -> &[Uuid] {
        &self.coverage_zone_ids
    }
    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_deref()
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
