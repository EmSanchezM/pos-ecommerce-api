//! Notification transport adapters.
//!
//! Each external transport implements [`NotificationAdapter`]; the
//! [`NotificationAdapterRegistry`] dispatches a `NotificationChannel` to the
//! right adapter so use cases can stay channel-agnostic.
//!
//! [`LogOnlyAdapter`] is the only adapter wired today — it is the dev
//! workhorse and the default for every channel. SendGrid / SES / Twilio /
//! WhatsApp Cloud / OneSignal go here as they are implemented.

mod log_only_adapter;
mod notification_adapter;
mod registry;

pub use log_only_adapter::LogOnlyAdapter;
pub use notification_adapter::{DeliveryResult, NotificationAdapter};
pub use registry::{DefaultNotificationAdapterRegistry, NotificationAdapterRegistry};
