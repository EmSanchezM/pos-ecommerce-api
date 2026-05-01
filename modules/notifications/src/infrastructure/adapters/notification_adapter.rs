//! NotificationAdapter trait — every transport (SendGrid, Twilio, WhatsApp
//! Cloud, OneSignal, ...) implements it. Use cases stay provider-agnostic.

use async_trait::async_trait;

use crate::NotificationsError;
use crate::domain::entities::Notification;
use crate::domain::value_objects::NotificationChannel;

/// Outcome of handing a notification to a transport.
#[derive(Debug, Clone)]
pub struct DeliveryResult {
    /// Provider-side identifier we can store for reconciliation/lookups.
    pub provider_message_id: Option<String>,
}

#[async_trait]
pub trait NotificationAdapter: Send + Sync {
    /// Channel this adapter handles. The registry uses this to dispatch.
    fn channel(&self) -> NotificationChannel;

    /// Hand the notification to the transport. Returning `Ok` means the
    /// transport accepted responsibility (sent to SMTP, queued at SendGrid,
    /// etc.) — final delivery may still fail asynchronously.
    async fn send(&self, notification: &Notification)
    -> Result<DeliveryResult, NotificationsError>;
}
