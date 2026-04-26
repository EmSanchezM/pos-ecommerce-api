//! PaymentGateway entity - per-store gateway configuration.
//!
//! A PaymentGateway represents a configured payment provider for a store
//! (e.g. "Stripe Production" or "BAC Sandbox"). Mutating these records is
//! restricted to super admins at the API layer; the domain itself is
//! permission-agnostic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{GatewayConfig, GatewayType, PaymentGatewayId};
use identity::StoreId;

/// PaymentGateway aggregate root.
///
/// Invariants:
/// - At most one default gateway per (store_id) — enforced at the use case
///   layer (the repository may also enforce it via a partial unique index).
/// - `supported_methods` and `supported_currencies` are non-empty after creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentGateway {
    id: PaymentGatewayId,
    store_id: StoreId,
    name: String,
    gateway_type: GatewayType,
    is_active: bool,
    is_default: bool,
    config: GatewayConfig,
    supported_methods: Vec<String>,
    supported_currencies: Vec<String>,
    webhook_secret: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PaymentGateway {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        name: String,
        gateway_type: GatewayType,
        is_default: bool,
        config: GatewayConfig,
        supported_methods: Vec<String>,
        supported_currencies: Vec<String>,
        webhook_secret: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: PaymentGatewayId::new(),
            store_id,
            name,
            gateway_type,
            is_active: true,
            is_default,
            config,
            supported_methods,
            supported_currencies,
            webhook_secret,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PaymentGatewayId,
        store_id: StoreId,
        name: String,
        gateway_type: GatewayType,
        is_active: bool,
        is_default: bool,
        config: GatewayConfig,
        supported_methods: Vec<String>,
        supported_currencies: Vec<String>,
        webhook_secret: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            gateway_type,
            is_active,
            is_default,
            config,
            supported_methods,
            supported_currencies,
            webhook_secret,
            created_at,
            updated_at,
        }
    }

    // -------------------------------------------------------------------------
    // Mutators
    // -------------------------------------------------------------------------

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

    pub fn set_supported_methods(&mut self, methods: Vec<String>) {
        self.supported_methods = methods;
        self.touch();
    }

    pub fn set_supported_currencies(&mut self, currencies: Vec<String>) {
        self.supported_currencies = currencies;
        self.touch();
    }

    pub fn set_webhook_secret(&mut self, webhook_secret: Option<String>) {
        self.webhook_secret = webhook_secret;
        self.touch();
    }

    pub fn config_mut(&mut self) -> &mut GatewayConfig {
        self.touch();
        &mut self.config
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -------------------------------------------------------------------------
    // Predicates
    // -------------------------------------------------------------------------

    pub fn supports_method(&self, method: &str) -> bool {
        self.supported_methods.iter().any(|m| m == method)
    }

    pub fn supports_currency(&self, currency: &str) -> bool {
        self.supported_currencies.iter().any(|c| c == currency)
    }

    // -------------------------------------------------------------------------
    // Getters
    // -------------------------------------------------------------------------

    pub fn id(&self) -> PaymentGatewayId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn gateway_type(&self) -> GatewayType {
        self.gateway_type
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    pub fn supported_methods(&self) -> &[String] {
        &self.supported_methods
    }

    pub fn supported_currencies(&self) -> &[String] {
        &self.supported_currencies
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
