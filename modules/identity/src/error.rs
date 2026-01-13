// Identity module errors

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Permission format invalid: must be 'module:action' with non-empty parts")]
    InvalidPermissionFormat,

    #[error("Invalid email format")]
    InvalidEmailFormat,

    #[error("Invalid username format: must be 3-50 alphanumeric characters or underscores")]
    InvalidUsernameFormat,

    #[error("Permission '{0}' already exists")]
    DuplicatePermission(String),

    #[error("Permission not found: {0}")]
    PermissionNotFound(Uuid),

    #[error("Role '{0}' already exists")]
    DuplicateRole(String),

    #[error("Role not found: {0}")]
    RoleNotFound(Uuid),

    #[error("Cannot delete system-protected role")]
    ProtectedRoleCannotBeDeleted,

    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    #[error("Email '{0}' already in use")]
    DuplicateEmail(String),

    #[error("Username '{0}' already in use")]
    DuplicateUsername(String),

    #[error("User account is inactive")]
    UserInactive,

    #[error("Store not found: {0}")]
    StoreNotFound(Uuid),

    #[error("Store is inactive: {0}")]
    StoreInactive(Uuid),

    #[error("User is not a member of store: {0}")]
    UserNotInStore(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not implemented")]
    NotImplemented,
}
