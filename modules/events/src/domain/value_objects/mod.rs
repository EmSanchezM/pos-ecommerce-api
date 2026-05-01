//! Value objects for the events domain.

mod event_status;
mod outbox_event_id;

pub use event_status::EventStatus;
pub use outbox_event_id::OutboxEventId;
