// Command DTOs for application layer use cases
//
// These DTOs represent the input data for various operations in the identity module.
// They use primitive types (String, Uuid, bool) rather than domain value objects
// to keep the application boundary clean and allow validation in use cases.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Permission Commands
// =============================================================================

/// Command to create a new permission
/// Requirements: 1.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionCommand {
    /// Permission code in format `module:action` (e.g., `sales:create_invoice`)
    pub code: String,
    /// Optional description of what this permission allows
    pub description: Option<String>,
}

// =============================================================================
// Role Commands
// =============================================================================

/// Command to create a new role
/// Requirements: 2.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleCommand {
    /// Unique name for the role
    pub name: String,
    /// Optional description of the role's purpose
    pub description: Option<String>,
}

/// Command to update an existing role
/// Requirements: 2.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleCommand {
    /// New name for the role (if changing)
    pub name: Option<String>,
    /// New description for the role (if changing)
    pub description: Option<String>,
}

// =============================================================================
// User Commands
// =============================================================================

/// Command to create a new user
/// Requirements: 6.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserCommand {
    /// Unique username for login
    pub username: String,
    /// Unique email address
    pub email: String,
    /// User's first name
    pub first_name: String,
    /// User's last name
    pub last_name: String,
    /// Plain text password (will be hashed in use case)
    pub password: String,
}

/// Command to update an existing user's profile
/// Requirements: 6.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserCommand {
    /// New first name (if changing)
    pub first_name: Option<String>,
    /// New last name (if changing)
    pub last_name: Option<String>,
    /// New email address (if changing, must be unique)
    pub email: Option<String>,
}

// =============================================================================
// Store Commands
// =============================================================================

/// Command to create a new store
/// Requirements: 7.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStoreCommand {
    /// Store name
    pub name: String,
    /// Store physical address
    pub address: String,
    /// Whether this is an e-commerce store (default: false for POS)
    #[serde(default)]
    pub is_ecommerce: bool,
}

/// Command to update an existing store
/// Requirements: 7.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStoreCommand {
    /// New store name (if changing)
    pub name: Option<String>,
    /// New store address (if changing)
    pub address: Option<String>,
    /// New e-commerce flag (if changing)
    pub is_ecommerce: Option<bool>,
}

// =============================================================================
// Role Assignment Commands
// =============================================================================

/// Command to assign a role to a user in a specific store
/// Requirements: 3.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRoleCommand {
    /// The user to assign the role to
    pub user_id: Uuid,
    /// The role to assign
    pub role_id: Uuid,
    /// The store context for this role assignment
    pub store_id: Uuid,
}

/// Command to remove a role from a user in a specific store
/// Requirements: 3.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRoleCommand {
    /// The user to remove the role from
    pub user_id: Uuid,
    /// The role to remove
    pub role_id: Uuid,
    /// The store context for this role removal
    pub store_id: Uuid,
}

// =============================================================================
// User-Store Membership Commands
// =============================================================================

/// Command to add a user to a store
/// Requirements: 8.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddUserToStoreCommand {
    /// The user to add to the store
    pub user_id: Uuid,
    /// The store to add the user to
    pub store_id: Uuid,
}

/// Command to remove a user from a store
/// Requirements: 8.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveUserFromStoreCommand {
    /// The user to remove from the store
    pub user_id: Uuid,
    /// The store to remove the user from
    pub store_id: Uuid,
}

// =============================================================================
// Permission Management Commands
// =============================================================================

/// Command to add a permission to a role
/// Requirements: 2.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPermissionToRoleCommand {
    /// The role to add the permission to
    pub role_id: Uuid,
    /// The permission to add
    pub permission_id: Uuid,
}

/// Command to remove a permission from a role
/// Requirements: 2.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovePermissionFromRoleCommand {
    /// The role to remove the permission from
    pub role_id: Uuid,
    /// The permission to remove
    pub permission_id: Uuid,
}

// =============================================================================
// User Status Commands
// =============================================================================

/// Command to set a user's active status
/// Requirements: 6.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetUserActiveCommand {
    /// The user to update
    pub user_id: Uuid,
    /// Whether the user should be active
    pub is_active: bool,
}

/// Command to set a store's active status
/// Requirements: 7.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStoreActiveCommand {
    /// The store to update
    pub store_id: Uuid,
    /// Whether the store should be active
    pub is_active: bool,
}
