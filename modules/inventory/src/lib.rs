// Inventory module - Product catalog, stock management, recipes, adjustments, and transfers
//
// Clean Architecture layers:
// - domain: Core business logic, entities, value objects, repository traits
// - application: Use cases, DTOs, orchestration
// - infrastructure: External implementations (PostgreSQL repositories)

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

// =============================================================================
// Public API - Re-exports for convenient access
// =============================================================================

// Error type
pub use error::InventoryError;

// -----------------------------------------------------------------------------
// Domain Layer - Entities (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use domain::entities::{...};

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use domain::value_objects::{...};

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use domain::repositories::{...};

// -----------------------------------------------------------------------------
// Application Layer - Use Cases (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use application::use_cases::{...};

// -----------------------------------------------------------------------------
// Application Layer - DTOs (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use application::dtos::{...};

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repositories (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use infrastructure::persistence::{...};
