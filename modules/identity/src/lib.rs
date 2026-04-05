// Identity module - Users, roles, and permissions (RBAC)
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
pub use error::IdentityError;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------
pub use domain::entities::{AuditAction, AuditEntry, Permission, Role, Store, User};

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------
pub use domain::value_objects::{
    Email, PermissionCode, PermissionId, RoleId, StoreId, UserId, Username,
};

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------
pub use domain::repositories::{
    AuditRepository, PermissionRepository, RoleRepository, StoreRepository, UserRepository,
};

// -----------------------------------------------------------------------------
// Domain Layer - Services (UserContext)
// -----------------------------------------------------------------------------
pub use domain::services::{PermissionCheckResult, UserContext};

// -----------------------------------------------------------------------------
// Domain Layer - Authentication
// -----------------------------------------------------------------------------
pub use domain::auth::{AuthError, LoginIdentifier, TokenClaims, TokenService};

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------
pub use application::use_cases::{
    // Role use cases
    AddPermissionToRoleUseCase,
    // Store use cases
    AddUserToStoreUseCase,
    // User use cases
    AssignRoleUseCase,
    // UserContext use case
    BuildUserContextUseCase,
    // Permission use cases
    CreatePermissionUseCase,
    CreateRoleUseCase,
    CreateStoreUseCase,
    CreateUserUseCase,
    DeletePermissionUseCase,
    DeleteRoleUseCase,
    ListPermissionsUseCase,
    // Auth use cases
    LoginUseCase,
    RefreshTokenUseCase,
    RegisterUserUseCase,
    RemovePermissionFromRoleUseCase,
    RemoveRoleUseCase,
    RemoveUserFromStoreUseCase,
    SetStoreActiveUseCase,
    SetUserActiveUseCase,
    UpdateStoreUseCase,
    UpdateUserUseCase,
};

// -----------------------------------------------------------------------------
// Application Layer - DTOs (Commands)
// -----------------------------------------------------------------------------
pub use application::dtos::{
    AddUserToStoreCommand,
    AssignRoleCommand,
    CreatePermissionCommand,
    CreateRoleCommand,
    CreateStoreCommand,
    CreateUserCommand,
    // Auth commands
    LoginCommand,
    RefreshCommand,
    RegisterEcommerceCommand,
    RegisterPosCommand,
    UpdateStoreCommand,
    UpdateUserCommand,
};

// -----------------------------------------------------------------------------
// Application Layer - DTOs (Responses)
// -----------------------------------------------------------------------------
pub use application::dtos::{ErrorResponse, ListResponse, LoginResponse, RegisterResponse};

// -----------------------------------------------------------------------------
// Application Layer - Validators
// -----------------------------------------------------------------------------
pub use application::validators::{
    MAX_NAME_LENGTH, MIN_PASSWORD_LENGTH, validate_name, validate_password,
};

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repositories
// -----------------------------------------------------------------------------
pub use infrastructure::persistence::{
    PgAuditRepository, PgPermissionRepository, PgRoleRepository, PgStoreRepository,
    PgUserRepository,
};

// -----------------------------------------------------------------------------
// Infrastructure Layer - JWT Token Service
// -----------------------------------------------------------------------------
pub use infrastructure::JwtTokenService;
