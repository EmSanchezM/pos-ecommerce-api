// UserRepository trait - interface for user persistence
//
// Requirements: 3.1, 3.2, 3.3, 3.4, 6.1, 6.5

use async_trait::async_trait;

use crate::domain::entities::{Permission, Role, Store, User};
use crate::domain::value_objects::{Email, RoleId, StoreId, UserId, Username};
use crate::error::IdentityError;

/// Repository trait for User entity persistence
///
/// Defines the contract for storing and retrieving users,
/// managing user-store memberships, and user-role assignments.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Saves a new user to the repository
    ///
    /// # Errors
    /// - `IdentityError::DuplicateUsername` if a user with the same username exists
    /// - `IdentityError::DuplicateEmail` if a user with the same email exists
    /// - `IdentityError::Database` on database errors
    async fn save(&self, user: &User) -> Result<(), IdentityError>;

    /// Finds a user by their ID
    ///
    /// Returns `None` if no user with the given ID exists.
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError>;

    /// Finds a user by their email
    ///
    /// Returns `None` if no user with the given email exists.
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, IdentityError>;

    /// Finds a user by their username
    ///
    /// Returns `None` if no user with the given username exists.
    async fn find_by_username(&self, username: &Username) -> Result<Option<User>, IdentityError>;

    /// Updates an existing user
    ///
    /// # Errors
    /// - `IdentityError::UserNotFound` if the user doesn't exist
    /// - `IdentityError::DuplicateEmail` if the new email conflicts with another user
    /// - `IdentityError::Database` on database errors
    async fn update(&self, user: &User) -> Result<(), IdentityError>;

    /// Assigns a role to a user in a specific store
    ///
    /// # Errors
    /// - `IdentityError::UserNotFound` if the user doesn't exist
    /// - `IdentityError::RoleNotFound` if the role doesn't exist
    /// - `IdentityError::StoreNotFound` if the store doesn't exist
    /// - `IdentityError::UserNotInStore` if the user is not a member of the store
    /// - `IdentityError::Database` on database errors
    async fn assign_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        store_id: StoreId,
    ) -> Result<(), IdentityError>;

    /// Removes a role from a user in a specific store
    ///
    /// # Errors
    /// - `IdentityError::UserNotFound` if the user doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn remove_role(
        &self,
        user_id: UserId,
        role_id: RoleId,
        store_id: StoreId,
    ) -> Result<(), IdentityError>;

    /// Gets all roles assigned to a user in a specific store
    async fn get_roles_for_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<Vec<Role>, IdentityError>;

    /// Gets all permissions for a user in a specific store
    ///
    /// Returns the union of all permissions from all roles assigned
    /// to the user in the specified store.
    async fn get_permissions_for_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<Vec<Permission>, IdentityError>;

    /// Removes a role from all users (across all stores)
    ///
    /// Used when deleting a role to ensure referential integrity.
    async fn remove_role_from_all_users(&self, role_id: RoleId) -> Result<(), IdentityError>;

    /// Adds a user to a store
    ///
    /// # Errors
    /// - `IdentityError::UserNotFound` if the user doesn't exist
    /// - `IdentityError::StoreNotFound` if the store doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn add_to_store(&self, user_id: UserId, store_id: StoreId) -> Result<(), IdentityError>;

    /// Removes a user from a store
    ///
    /// Also removes all role assignments for the user in that store.
    ///
    /// # Errors
    /// - `IdentityError::UserNotFound` if the user doesn't exist
    /// - `IdentityError::Database` on database errors
    async fn remove_from_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<(), IdentityError>;

    /// Gets all stores a user is a member of
    async fn get_stores(&self, user_id: UserId) -> Result<Vec<Store>, IdentityError>;

    /// Checks if a user is a member of a store
    async fn is_member_of_store(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<bool, IdentityError>;
}
