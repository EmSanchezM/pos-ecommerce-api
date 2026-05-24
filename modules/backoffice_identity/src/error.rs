use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum BackofficeIdentityError {
    #[error("Invalid email format")]
    InvalidEmailFormat,

    #[error("Invalid permission code format: must match 'platform:resource.action'")]
    InvalidPermissionCodeFormat,

    #[error("Backoffice user not found: {0}")]
    UserNotFound(Uuid),

    #[error("Backoffice user email '{0}' already in use")]
    DuplicateEmail(String),

    #[error("Backoffice role not found: {0}")]
    RoleNotFound(Uuid),

    #[error("Backoffice role '{0}' already exists")]
    DuplicateRole(String),

    #[error("Cannot modify system-protected role")]
    ProtectedRoleCannotBeModified,

    #[error("Backoffice permission not found: {0}")]
    PermissionNotFound(Uuid),

    #[error("Backoffice permission '{0}' already exists")]
    DuplicatePermission(String),

    #[error("Backoffice user account is inactive")]
    UserInactive,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Password hash error: {0}")]
    PasswordHashError(String),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
