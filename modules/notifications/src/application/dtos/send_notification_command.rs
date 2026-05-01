//! DTO for `SendNotificationUseCase`.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::NotificationChannel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendNotificationCommand {
    pub channel: NotificationChannel,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    #[serde(default)]
    pub metadata: JsonValue,
}
