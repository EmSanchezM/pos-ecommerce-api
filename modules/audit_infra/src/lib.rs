//! # audit_infra
//!
//! Neutral crate that owns the backoffice audit pipeline.
//!
//! Per Decision 1 (Opción C) in `sdd/backoffice-api/decisions`, this crate
//! lives under `modules/` (not `crates/`) to match workspace convention.
//! Both `api-gateway` and `backoffice-api` import it and register the
//! `BackofficeAuditSubscriber` in their event_dispatcher.
//!
//! `api-gateway` does NOT import `modules/backoffice_identity` — only this
//! neutral crate. This keeps the blast-radius of `backoffice_identity` changes
//! away from the tenant API binary.
//!
//! ## Architecture
//!
//! Clean Architecture layers:
//! - **Domain**: `BackofficeAuditEvent` (outbox payload), `BackofficeAuditLogRepository` trait
//! - **Infrastructure**: `BackofficeAuditSubscriber` (EventSubscriber impl), `PgBackofficeAuditLogRepository`

pub mod domain;
pub mod infrastructure;

mod error;

// Public surface
pub use error::AuditInfraError;

// Domain
pub use domain::events::{AUDIT_EVENT_TYPE_PREFIX, BackofficeAuditEvent, OrgId};
pub use domain::repositories::{
    AuditLogFilters, BackofficeAuditLogEntry, BackofficeAuditLogRepository,
};

// Infrastructure
pub use infrastructure::{BackofficeAuditSubscriber, PgBackofficeAuditLogRepository};
