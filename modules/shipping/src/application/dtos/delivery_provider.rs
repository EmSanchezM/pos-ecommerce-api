use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::DeliveryProvider;

#[derive(Debug, Deserialize)]
pub struct ConfigureDeliveryProviderCommand {
    pub store_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub api_key: String,
    pub secret_key: String,
    pub merchant_id: Option<String>,
    pub is_sandbox: bool,
    pub is_default: bool,
    pub coverage_zone_ids: Vec<Uuid>,
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateDeliveryProviderCommand {
    #[serde(default)]
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub merchant_id: Option<Option<String>>,
    pub is_sandbox: Option<bool>,
    pub is_active: Option<bool>,
    pub is_default: Option<bool>,
    pub coverage_zone_ids: Option<Vec<Uuid>>,
    pub webhook_secret: Option<Option<String>>,
}

/// Credentials are NEVER serialized.
#[derive(Debug, Serialize)]
pub struct DeliveryProviderResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub is_active: bool,
    pub is_default: bool,
    pub is_sandbox: bool,
    pub merchant_id: Option<String>,
    pub coverage_zone_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DeliveryProvider> for DeliveryProviderResponse {
    fn from(p: DeliveryProvider) -> Self {
        Self {
            id: p.id().into_uuid(),
            store_id: p.store_id().into_uuid(),
            name: p.name().to_string(),
            provider_type: p.provider_type().to_string(),
            is_active: p.is_active(),
            is_default: p.is_default(),
            is_sandbox: p.is_sandbox(),
            merchant_id: p.merchant_id().map(str::to_string),
            coverage_zone_ids: p.coverage_zone_ids().to_vec(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}
