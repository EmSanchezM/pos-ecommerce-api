//! Infrastructure layer for the inventory module.
//!
//! This layer contains implementations of repository traits and other
//! external integrations (database, external services, etc.).
//!
//! ## Persistence
//!
//! PostgreSQL repository implementations are available in the [`persistence`] module.

pub mod persistence;
