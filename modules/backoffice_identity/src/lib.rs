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
pub use domain::entities::{BackofficePermission, BackofficeRole, BackofficeUser, MfaRecoveryCode};

// Domain — Value Objects
pub use domain::value_objects::{
    BackofficeEmail, BackofficePermissionId, BackofficeRoleId, BackofficeUserId,
    PlatformPermissionCode, TotpSecret,
};

// Domain — Repository Traits
pub use domain::repositories::{
    BackofficePermissionRepository, BackofficeRoleRepository, BackofficeUserRepository,
    MfaRecoveryCodeRepository,
};

// Domain — Auth Traits
pub use domain::auth::{BackofficeTokenService, ImpersonationTokenIssuer, TotpService};

// Application — Use Cases
pub use application::use_cases::AuthenticateBackofficeUserUseCase;
pub use application::use_cases::SuspendOrganizationWithAuditUseCase;
pub use application::use_cases::{
    IMPERSONATION_TOKEN_EXPIRY_SECONDS, IssueImpersonationTokenWithAuditUseCase,
};
pub use application::use_cases::{
    ManageMfaEnrollmentUseCase, MfaEnrollmentStarted, MfaRecoveryCodesIssued, MfaStatus,
    verify_recovery_code,
};

// Application — DTOs
pub use application::dtos::{
    AuthenticateBackofficeCommand, BackofficeAuthResponse, ImpersonationTokenResponse,
};

// Infrastructure — Pg Implementations
pub use infrastructure::persistence::{
    PgBackofficePermissionRepository, PgBackofficeRoleRepository, PgBackofficeUserRepository,
    PgMfaRecoveryCodeRepository,
};

// Infrastructure — JWT Token Service
pub use infrastructure::JwtBackofficeTokenService;

// Infrastructure — TOTP
pub use infrastructure::TotpRsService;
