//! NotificationChannel — which transport delivers the notification.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::NotificationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Sms,
    WhatsApp,
    Push,
    Webhook,
}

impl fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationChannel::Email => write!(f, "email"),
            NotificationChannel::Sms => write!(f, "sms"),
            NotificationChannel::WhatsApp => write!(f, "whatsapp"),
            NotificationChannel::Push => write!(f, "push"),
            NotificationChannel::Webhook => write!(f, "webhook"),
        }
    }
}

impl FromStr for NotificationChannel {
    type Err = NotificationsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "email" => Ok(Self::Email),
            "sms" => Ok(Self::Sms),
            "whatsapp" => Ok(Self::WhatsApp),
            "push" => Ok(Self::Push),
            "webhook" => Ok(Self::Webhook),
            other => Err(NotificationsError::InvalidChannel(other.into())),
        }
    }
}
