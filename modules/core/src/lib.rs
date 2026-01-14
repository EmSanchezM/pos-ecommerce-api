// Core module - Store and Terminal Management
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

// Error type (to be implemented in task 4)
pub use error::*;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------
pub use domain::entities::{CaiRange, Terminal};

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------
pub use domain::value_objects::{CaiNumber, TerminalCode, TerminalId};

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------
pub use domain::repositories::TerminalRepository;

// -----------------------------------------------------------------------------
// Application Layer - DTOs
// -----------------------------------------------------------------------------
pub use application::dtos::*;

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------
pub use application::use_cases::{
    AssignCaiUseCase, CreateTerminalUseCase, GetCaiStatusUseCase, GetNextInvoiceNumberUseCase,
    GetStoreDetailUseCase, GetTerminalDetailUseCase, ListStoresUseCase, ListTerminalsUseCase,
    SetStoreActiveUseCaseExtended, SetTerminalActiveUseCase, UpdateTerminalUseCase,
};

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repositories
// -----------------------------------------------------------------------------
pub use infrastructure::persistence::PgTerminalRepository;
