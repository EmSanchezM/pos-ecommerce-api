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
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------

// ID value objects
pub use domain::value_objects::ProductId;
pub use domain::value_objects::VariantId;
pub use domain::value_objects::StockId;
pub use domain::value_objects::MovementId;
pub use domain::value_objects::ReservationId;
pub use domain::value_objects::RecipeId;
pub use domain::value_objects::IngredientId;
pub use domain::value_objects::SubstituteId;
pub use domain::value_objects::AdjustmentId;
pub use domain::value_objects::TransferId;
pub use domain::value_objects::CategoryId;

// Validated value objects
pub use domain::value_objects::Sku;
pub use domain::value_objects::Barcode;
pub use domain::value_objects::Currency;
pub use domain::value_objects::UnitOfMeasure;

// Enum value objects
pub use domain::value_objects::MovementType;
pub use domain::value_objects::ReservationStatus;
pub use domain::value_objects::AdjustmentStatus;
pub use domain::value_objects::AdjustmentType;
pub use domain::value_objects::AdjustmentReason;
pub use domain::value_objects::TransferStatus;

// -----------------------------------------------------------------------------
// Domain Layer - Entities (to be implemented in later tasks)
// -----------------------------------------------------------------------------
// pub use domain::entities::{...};

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
