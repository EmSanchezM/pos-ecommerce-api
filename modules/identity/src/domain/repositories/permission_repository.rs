// PermissionRepository trait - interface for permission persistence

use async_trait::async_trait;

use crate::domain::entities::Permission;
use crate::domain::value_objects::{PermissionCode, PermissionId};
use crate::error::IdentityError;

/// Repository trait for Permission entity persistence
///
/// Defines the contract for storing and retrieving permissions.
/// Implementations handle the actual database operations.
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    /// Saves a new permission to the repository
    ///
    /// # Errors
    /// - `IdentityError::DuplicatePermission` if a permission with the same code exists
    /// - `IdentityError::Database` on database errors
    async fn save(&self, permission: &Permission) -> Result<(), IdentityError>;

    /// Finds a permission by its ID
    ///
    /// Returns `None` if no permission with the given ID exists.
    async fn find_by_id(&self, id: PermissionId) -> Result<Option<Permission>, IdentityError>;

    /// Finds a permission by its code
    ///
    /// Returns `None` if no permission with the given code exists.
    async fn find_by_code(&self, code: &PermissionCode) -> Result<Option<Permission>, IdentityError>;

    /// Returns all permissions in the repository
    async fn find_all(&self) -> Result<Vec<Permission>, IdentityError>;

    /// Finds all permissions belonging to a specific module
    ///
    /// Filters permissions whose code starts with the given module prefix.
    /// For example, `find_by_module("sales")` returns permissions like
    /// `sales:create_invoice`, `sales:view_orders`, etc.
    async fn find_by_module(&self, module: &str) -> Result<Vec<Permission>, IdentityError>;

    /// Deletes a permission by its ID
    ///
    /// # Errors
    /// - `IdentityError::PermissionNotFound` if the permission doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn delete(&self, id: PermissionId) -> Result<(), IdentityError>;

    /// Checks if a permission with the given code exists
    async fn exists(&self, code: &PermissionCode) -> Result<bool, IdentityError>;
}
