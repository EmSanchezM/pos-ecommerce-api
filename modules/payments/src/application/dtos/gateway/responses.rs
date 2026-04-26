//! Gateway response DTOs.
//!
//! NOTE: api_key / secret_key are NEVER serialized — only the sandbox flag and
//! the merchant id are exposed.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::PaymentGateway;

#[derive(Debug, Serialize)]
pub struct GatewayResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub gateway_type: String,
    pub is_active: bool,
    pub is_default: bool,
    pub is_sandbox: bool,
    pub merchant_id: Option<String>,
    pub supported_methods: Vec<String>,
    pub supported_currencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PaymentGateway> for GatewayResponse {
    fn from(g: PaymentGateway) -> Self {
        Self {
            id: g.id().into_uuid(),
            store_id: g.store_id().into_uuid(),
            name: g.name().to_string(),
            gateway_type: g.gateway_type().to_string(),
            is_active: g.is_active(),
            is_default: g.is_default(),
            is_sandbox: g.config().is_sandbox(),
            merchant_id: g.config().merchant_id().map(str::to_string),
            supported_methods: g.supported_methods().to_vec(),
            supported_currencies: g.supported_currencies().to_vec(),
            created_at: g.created_at(),
            updated_at: g.updated_at(),
        }
    }
}

impl From<&PaymentGateway> for GatewayResponse {
    fn from(g: &PaymentGateway) -> Self {
        Self {
            id: g.id().into_uuid(),
            store_id: g.store_id().into_uuid(),
            name: g.name().to_string(),
            gateway_type: g.gateway_type().to_string(),
            is_active: g.is_active(),
            is_default: g.is_default(),
            is_sandbox: g.config().is_sandbox(),
            merchant_id: g.config().merchant_id().map(str::to_string),
            supported_methods: g.supported_methods().to_vec(),
            supported_currencies: g.supported_currencies().to_vec(),
            created_at: g.created_at(),
            updated_at: g.updated_at(),
        }
    }
}
