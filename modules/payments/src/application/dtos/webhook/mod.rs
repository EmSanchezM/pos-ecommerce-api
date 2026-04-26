//! Webhook DTOs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    pub gateway_type: String,
    pub raw_body: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub processed: bool,
    pub transaction_id: Option<Uuid>,
    pub event_type: String,
}
