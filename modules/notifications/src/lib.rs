//! # Notifications Module
//!
//! Multi-channel outbound messaging for the POS + eCommerce platform.
//!
//! - **Channels**: email, SMS, WhatsApp, push, webhook (each represented by a
//!   [`NotificationChannel`] variant).
//! - **Adapters**: each external transport implements [`NotificationAdapter`];
//!   the [`NotificationAdapterRegistry`] dispatches a channel to its adapter.
//!   The default registry wires [`LogOnlyAdapter`] for every channel so a
//!   fresh dev environment runs without provider credentials.
//! - **Use cases**: [`SendNotificationUseCase`] enqueues + delivers in one
//!   step; [`RetryFailedNotificationsUseCase`] is the periodic retry job.
//!
//! ## Architecture
//!
//! Hexagonal/clean architecture with three layers:
//!
//! - **Domain**: `Notification` entity, `NotificationRepository` trait
//! - **Application**: `SendNotificationUseCase`, `RetryFailedNotificationsUseCase`
//! - **Infrastructure**: `PgNotificationRepository`, adapter trait + registry

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::NotificationsError;

// Domain
pub use domain::entities::Notification;
pub use domain::repositories::NotificationRepository;
pub use domain::value_objects::{NotificationChannel, NotificationId, NotificationStatus};

// Application
pub use application::dtos::SendNotificationCommand;
pub use application::use_cases::{
    DEFAULT_MAX_ATTEMPTS, RetryFailedNotificationsUseCase, SendNotificationUseCase,
};

// Infrastructure
pub use infrastructure::adapters::{
    DefaultNotificationAdapterRegistry, DeliveryResult, LogOnlyAdapter, NotificationAdapter,
    NotificationAdapterRegistry,
};
pub use infrastructure::persistence::PgNotificationRepository;
