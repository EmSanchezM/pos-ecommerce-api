use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct DeliveryWebhookPayload {
    pub provider_type: String,
    pub raw_body: String,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct DeliveryWebhookResponse {
    pub processed: bool,
    pub shipment_id: Option<Uuid>,
    pub event_type: String,
}
