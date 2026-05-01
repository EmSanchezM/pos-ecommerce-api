//! Gateway command DTOs

use serde::Deserialize;
use uuid::Uuid;

/// Create a new payment gateway. Restricted to super admins at the API layer.
#[derive(Debug, Deserialize)]
pub struct ConfigureGatewayCommand {
    pub store_id: Uuid,
    pub name: String,
    pub gateway_type: String,
    pub api_key: String,
    pub secret_key: String,
    pub merchant_id: Option<String>,
    pub is_sandbox: bool,
    pub is_default: bool,
    pub supported_methods: Vec<String>,
    pub supported_currencies: Vec<String>,
    pub webhook_secret: Option<String>,
}

/// Update an existing payment gateway. Restricted to super admins at the API layer.
#[derive(Debug, Deserialize, Default)]
pub struct UpdateGatewayCommand {
    #[serde(default)]
    pub gateway_id: Uuid,
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub merchant_id: Option<Option<String>>,
    pub is_sandbox: Option<bool>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
    pub supported_methods: Option<Vec<String>>,
    pub supported_currencies: Option<Vec<String>>,
    pub webhook_secret: Option<Option<String>>,
}
