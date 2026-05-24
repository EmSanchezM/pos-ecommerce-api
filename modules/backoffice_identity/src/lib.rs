// Backoffice Identity module — platform-level RBAC and identity (independent of tenant identity)
//
// Clean Architecture layers:
// - domain: entities, value objects, repository traits
// - application: use cases, DTOs (Phase 2+)
// - infrastructure: PgRepository implementations

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

// =============================================================================
// Public API
// =============================================================================

pub use error::BackofficeIdentityError;

// Domain — Entities
pub use domain::entities::{BackofficePermission, BackofficeRole, BackofficeUser};

// Domain — Value Objects
pub use domain::value_objects::{
    BackofficeEmail, BackofficePermissionId, BackofficeRoleId, BackofficeUserId,
    PlatformPermissionCode,
};

// Domain — Repository Traits
pub use domain::repositories::{
    BackofficePermissionRepository, BackofficeRoleRepository, BackofficeUserRepository,
};

// Domain — Auth Traits
pub use domain::auth::BackofficeTokenService;

// Application — Use Cases
pub use application::use_cases::AuthenticateBackofficeUserUseCase;
pub use application::use_cases::SuspendOrganizationWithAuditUseCase;

// Application — DTOs
pub use application::dtos::{AuthenticateBackofficeCommand, BackofficeAuthResponse};

// Infrastructure — Pg Implementations
pub use infrastructure::persistence::{
    PgBackofficePermissionRepository, PgBackofficeRoleRepository, PgBackofficeUserRepository,
};

// Infrastructure — JWT Token Service
pub use infrastructure::JwtBackofficeTokenService;
