//! # Events Module
//!
//! Transactional outbox + in-process event dispatch.
//!
//! Other modules build a domain event payload and call
//! [`PublishEventUseCase`] from inside the same `sqlx::Transaction` they use
//! to persist the aggregate change. The event lands in `outbox_events` only
//! if the transaction commits, which guarantees at-least-once delivery
//! without distributed coordination.
//!
//! A background worker periodically calls
//! [`DispatchPendingEventsUseCase`], which fans events out to in-process
//! [`EventSubscriber`]s (analytics, notifications, accounting, ...). Each
//! subscriber fails independently; on any subscriber failure the event is
//! re-attempted until [`MAX_DELIVERY_ATTEMPTS`] is reached, after which it
//! is marked `failed` and requires manual review.
//!
//! ## Architecture
//!
//! Hexagonal/clean architecture with three layers:
//!
//! - **Domain**: `OutboxEvent` entity, `OutboxRepository` trait
//! - **Application**: `PublishEventUseCase`, `DispatchPendingEventsUseCase`,
//!   `EventSubscriber` trait, `SubscriberRegistry`
//! - **Infrastructure**: `PgOutboxRepository`

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::EventsError;

// Domain
pub use domain::entities::{MAX_DELIVERY_ATTEMPTS, OutboxEvent};
pub use domain::repositories::OutboxRepository;
pub use domain::value_objects::{EventStatus, OutboxEventId};

// Application
pub use application::subscriber::{EventSubscriber, SubscriberRegistry};
pub use application::use_cases::{DispatchPendingEventsUseCase, PublishEventUseCase};

// Infrastructure
pub use infrastructure::persistence::PgOutboxRepository;
