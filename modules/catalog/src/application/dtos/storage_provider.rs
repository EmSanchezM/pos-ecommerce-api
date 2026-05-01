use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ImageStorageProvider;

#[derive(Debug, Deserialize)]
pub struct ConfigureStorageProviderCommand {
    pub store_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub api_key: String,
    pub secret_key: String,
    pub is_default: bool,
    pub config_json: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStorageProviderCommand {
    #[serde(default)]
    pub provider_id: Uuid,
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub config_json: Option<Option<String>>,
    pub is_active: Option<bool>,
    pub is_default: Option<bool>,
}

/// Credentials are NEVER serialized.
#[derive(Debug, Serialize)]
pub struct StorageProviderResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub is_active: bool,
    pub is_default: bool,
    pub config_json: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ImageStorageProvider> for StorageProviderResponse {
    fn from(p: ImageStorageProvider) -> Self {
        Self {
            id: p.id().into_uuid(),
            store_id: p.store_id().into_uuid(),
            name: p.name().to_string(),
            provider_type: p.provider_type().to_string(),
            is_active: p.is_active(),
            is_default: p.is_default(),
            config_json: p.config_json().map(str::to_string),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}
