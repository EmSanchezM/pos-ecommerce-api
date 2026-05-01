//! Domain entities for the events module.

mod outbox_event;

pub use outbox_event::{MAX_DELIVERY_ATTEMPTS, OutboxEvent};
