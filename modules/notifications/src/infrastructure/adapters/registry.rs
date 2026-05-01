//! NotificationAdapterRegistry — selects the adapter for a given channel.
//!
//! Mirrors `payments::DefaultGatewayAdapterRegistry`: one adapter per
//! channel, swappable per deployment. The default registers `LogOnlyAdapter`
//! for every channel so a fresh dev environment runs without provider
//! credentials. Production builds replace each entry with a real adapter.

use std::sync::Arc;

use crate::NotificationsError;
use crate::domain::value_objects::NotificationChannel;

use super::log_only_adapter::LogOnlyAdapter;
use super::notification_adapter::NotificationAdapter;

pub trait NotificationAdapterRegistry: Send + Sync {
    fn for_channel(
        &self,
        channel: NotificationChannel,
    ) -> Result<Arc<dyn NotificationAdapter>, NotificationsError>;
}

/// Default registry — every channel resolves to a `LogOnlyAdapter`.
pub struct DefaultNotificationAdapterRegistry {
    email: Arc<dyn NotificationAdapter>,
    sms: Arc<dyn NotificationAdapter>,
    whatsapp: Arc<dyn NotificationAdapter>,
    push: Arc<dyn NotificationAdapter>,
    webhook: Arc<dyn NotificationAdapter>,
}

impl DefaultNotificationAdapterRegistry {
    pub fn new() -> Self {
        Self {
            email: Arc::new(LogOnlyAdapter::new(NotificationChannel::Email)),
            sms: Arc::new(LogOnlyAdapter::new(NotificationChannel::Sms)),
            whatsapp: Arc::new(LogOnlyAdapter::new(NotificationChannel::WhatsApp)),
            push: Arc::new(LogOnlyAdapter::new(NotificationChannel::Push)),
            webhook: Arc::new(LogOnlyAdapter::new(NotificationChannel::Webhook)),
        }
    }

    /// Override adapters per channel — used in tests and prod wiring.
    pub fn with_overrides(
        email: Arc<dyn NotificationAdapter>,
        sms: Arc<dyn NotificationAdapter>,
        whatsapp: Arc<dyn NotificationAdapter>,
        push: Arc<dyn NotificationAdapter>,
        webhook: Arc<dyn NotificationAdapter>,
    ) -> Self {
        Self {
            email,
            sms,
            whatsapp,
            push,
            webhook,
        }
    }
}

impl Default for DefaultNotificationAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationAdapterRegistry for DefaultNotificationAdapterRegistry {
    fn for_channel(
        &self,
        channel: NotificationChannel,
    ) -> Result<Arc<dyn NotificationAdapter>, NotificationsError> {
        Ok(match channel {
            NotificationChannel::Email => self.email.clone(),
            NotificationChannel::Sms => self.sms.clone(),
            NotificationChannel::WhatsApp => self.whatsapp.clone(),
            NotificationChannel::Push => self.push.clone(),
            NotificationChannel::Webhook => self.webhook.clone(),
        })
    }
}
