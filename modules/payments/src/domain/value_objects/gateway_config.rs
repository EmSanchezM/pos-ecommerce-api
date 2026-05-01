//! GatewayConfig value object - holds the (encrypted) credentials for a gateway.
//!
//! Credentials are stored encrypted at rest. The current implementation simply
//! holds the encrypted blob; encryption/decryption belongs to an infrastructure
//! adapter and is intentionally not done here so the domain stays pure.

use serde::{Deserialize, Serialize};

/// Credentials and runtime flags for a payment gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    api_key_encrypted: String,
    secret_key_encrypted: String,
    merchant_id: Option<String>,
    is_sandbox: bool,
}

impl GatewayConfig {
    pub fn new(
        api_key_encrypted: String,
        secret_key_encrypted: String,
        merchant_id: Option<String>,
        is_sandbox: bool,
    ) -> Self {
        Self {
            api_key_encrypted,
            secret_key_encrypted,
            merchant_id,
            is_sandbox,
        }
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

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key_encrypted = api_key;
    }

    pub fn set_secret_key(&mut self, secret_key: String) {
        self.secret_key_encrypted = secret_key;
    }

    pub fn set_merchant_id(&mut self, merchant_id: Option<String>) {
        self.merchant_id = merchant_id;
    }

    pub fn set_sandbox(&mut self, is_sandbox: bool) {
        self.is_sandbox = is_sandbox;
    }
}
