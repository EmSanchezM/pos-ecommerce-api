// RoleRepository trait - interface for role persistence

use async_trait::async_trait;

use crate::domain::entities::{Permission, Role};
use crate::domain::value_objects::{PermissionId, RoleId};
use crate::error::IdentityError;

/// Repository trait for Role entity persistence
///
/// Defines the contract for storing and retrieving roles,
/// as well as managing role-permission relationships.
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// Saves a new role to the repository
    ///
    /// # Errors
    /// - `IdentityError::DuplicateRole` if a role with the same name exists
    /// - `IdentityError::Database` on database errors
    async fn save(&self, role: &Role) -> Result<(), IdentityError>;

    /// Finds a role by its ID
    ///
    /// Returns `None` if no role with the given ID exists.
    async fn find_by_id(&self, id: RoleId) -> Result<Option<Role>, IdentityError>;

    /// Finds a role by its name
    ///
    /// Returns `None` if no role with the given name exists.
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, IdentityError>;

    /// Returns all roles in the repository
    async fn find_all(&self) -> Result<Vec<Role>, IdentityError>;

    /// Deletes a role by its ID
    ///
    /// # Errors
    /// - `IdentityError::RoleNotFound` if the role doesn't exist
    /// - `IdentityError::ProtectedRoleCannotBeDeleted` if the role is system-protected
    /// - `IdentityError::Database` on database errors
    async fn delete(&self, id: RoleId) -> Result<(), IdentityError>;

    /// Updates an existing role
    ///
    /// # Errors
    /// - `IdentityError::RoleNotFound` if the role doesn't exist
    /// - `IdentityError::DuplicateRole` if the new name conflicts with another role
    /// - `IdentityError::Database` on database errors
    async fn update(&self, role: &Role) -> Result<(), IdentityError>;

    /// Adds a permission to a role
    ///
    /// # Errors
    /// - `IdentityError::RoleNotFound` if the role doesn't exist
    /// - `IdentityError::PermissionNotFound` if the permission doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn add_permission(
        &self,
        role_id: RoleId,
        permission_id: PermissionId,
    ) -> Result<(), IdentityError>;

    /// Removes a permission from a role
    ///
    /// # Errors
    /// - `IdentityError::RoleNotFound` if the role doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn remove_permission(
        &self,
        role_id: RoleId,
        permission_id: PermissionId,
    ) -> Result<(), IdentityError>;

    /// Gets all permissions assigned to a role
    async fn get_permissions(&self, role_id: RoleId) -> Result<Vec<Permission>, IdentityError>;

    /// Removes a permission from all roles
    ///
    /// Used when deleting a permission to ensure referential integrity.
    async fn remove_permission_from_all_roles(
        &self,
        permission_id: PermissionId,
    ) -> Result<(), IdentityError>;
}
