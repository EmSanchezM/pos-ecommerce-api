// UserContext service - immutable context for the current user session
//
// Contains the user_id, store_id, and a deduplicated set of permissions
// from all roles assigned to the user in that store.

use std::collections::HashSet;

use crate::domain::value_objects::{PermissionCode, StoreId, UserId};

/// Result of a permission check operation
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionCheckResult {
    /// All requested permissions are granted
    Granted,
    /// Some permissions are missing
    Denied { missing: Vec<String> },
}

/// Immutable context for the current user session.
///
/// Contains the user_id, store_id, and a deduplicated set of permissions
/// from all roles assigned to the user in that store.
///
/// Once constructed, the context does not change during the request lifecycle.
#[derive(Debug, Clone)]
pub struct UserContext {
    user_id: UserId,
    store_id: StoreId,
    permissions: HashSet<PermissionCode>,
}

impl UserContext {
    /// Creates a new UserContext with the given user_id, store_id, and permissions.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user
    /// * `store_id` - The ID of the store/tenant context
    /// * `permissions` - A deduplicated set of permission codes
    pub fn new(user_id: UserId, store_id: StoreId, permissions: HashSet<PermissionCode>) -> Self {
        Self {
            user_id,
            store_id,
            permissions,
        }
    }

    /// Returns a reference to the user ID
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Returns a reference to the store ID
    pub fn store_id(&self) -> &StoreId {
        &self.store_id
    }

    /// Returns a reference to the permissions set
    pub fn permissions(&self) -> &HashSet<PermissionCode> {
        &self.permissions
    }


    /// Checks if the user has a specific permission.
    ///
    /// Returns `true` if the permission exists in the context's permission set,
    /// `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `permission` - The permission code string to check (format: "module:action")
    ///
    /// # Example
    ///
    /// ```ignore
    /// if ctx.has_permission("sales:create_invoice") {
    ///     // User can create invoices
    /// }
    /// ```
    pub fn has_permission(&self, permission: &str) -> bool {
        // Try to parse the permission string and check if it exists
        match PermissionCode::new(permission) {
            Ok(code) => self.permissions.contains(&code),
            Err(_) => false, // Invalid format means permission doesn't exist
        }
    }


    /// Checks if the user has ALL of the specified permissions (AND logic).
    ///
    /// Returns `true` if every permission in the list exists in the context's
    /// permission set, `false` if any permission is missing.
    ///
    /// # Arguments
    ///
    /// * `permissions` - A slice of permission code strings to check
    ///
    /// # Example
    ///
    /// ```ignore
    /// // User must have both permissions
    /// if ctx.has_all_permissions(&["sales:create_invoice", "sales:view_invoice"]) {
    ///     // User can create and view invoices
    /// }
    /// ```
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }


    /// Checks if the user has AT LEAST ONE of the specified permissions (OR logic).
    ///
    /// Returns `true` if at least one permission in the list exists in the context's
    /// permission set, `false` if none of the permissions are present.
    ///
    /// For an empty list, returns `false` (no permissions to match).
    ///
    /// # Arguments
    ///
    /// * `permissions` - A slice of permission code strings to check
    ///
    /// # Example
    ///
    /// ```ignore
    /// // User needs at least one of these permissions
    /// if ctx.has_any_permission(&["admin:manage_users", "sales:manage_team"]) {
    ///     // User can manage something
    /// }
    /// ```
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        if permissions.is_empty() {
            return false;
        }
        permissions.iter().any(|p| self.has_permission(p))
    }


    /// Checks permissions and returns detailed information about missing permissions.
    ///
    /// Returns `PermissionCheckResult::Granted` if all permissions are present,
    /// or `PermissionCheckResult::Denied` with a list of missing permissions.
    ///
    /// # Arguments
    ///
    /// * `permissions` - A slice of permission code strings to check
    ///
    /// # Example
    ///
    /// ```ignore
    /// match ctx.check_permissions(&["sales:create", "sales:delete"]) {
    ///     PermissionCheckResult::Granted => {
    ///         // Proceed with operation
    ///     }
    ///     PermissionCheckResult::Denied { missing } => {
    ///         // Log or report missing permissions
    ///         println!("Missing permissions: {:?}", missing);
    ///     }
    /// }
    /// ```
    pub fn check_permissions(&self, permissions: &[&str]) -> PermissionCheckResult {
        let missing: Vec<String> = permissions
            .iter()
            .filter(|p| !self.has_permission(p))
            .map(|p| p.to_string())
            .collect();

        if missing.is_empty() {
            PermissionCheckResult::Granted
        } else {
            PermissionCheckResult::Denied { missing }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(permissions: &[&str]) -> UserContext {
        let perms: HashSet<PermissionCode> = permissions
            .iter()
            .filter_map(|p| PermissionCode::new(p).ok())
            .collect();
        UserContext::new(UserId::new(), StoreId::new(), perms)
    }

    #[test]
    fn test_user_context_new() {
        let user_id = UserId::new();
        let store_id = StoreId::new();
        let permissions = HashSet::new();

        let ctx = UserContext::new(user_id, store_id, permissions);

        assert_eq!(*ctx.user_id(), user_id);
        assert_eq!(*ctx.store_id(), store_id);
        assert!(ctx.permissions().is_empty());
    }

    #[test]
    fn test_user_context_with_permissions() {
        let ctx = create_test_context(&["sales:create", "sales:view"]);

        assert_eq!(ctx.permissions().len(), 2);
    }

    #[test]
    fn test_has_permission_returns_true_when_present() {
        let ctx = create_test_context(&["sales:create", "inventory:view"]);

        assert!(ctx.has_permission("sales:create"));
        assert!(ctx.has_permission("inventory:view"));
    }

    #[test]
    fn test_has_permission_returns_false_when_absent() {
        let ctx = create_test_context(&["sales:create"]);

        assert!(!ctx.has_permission("sales:delete"));
        assert!(!ctx.has_permission("inventory:view"));
    }

    #[test]
    fn test_has_permission_returns_false_for_invalid_format() {
        let ctx = create_test_context(&["sales:create"]);

        assert!(!ctx.has_permission("invalid"));
        assert!(!ctx.has_permission(""));
        assert!(!ctx.has_permission(":"));
    }

    #[test]
    fn test_has_all_permissions_returns_true_when_all_present() {
        let ctx = create_test_context(&["sales:create", "sales:view", "sales:delete"]);

        assert!(ctx.has_all_permissions(&["sales:create", "sales:view"]));
        assert!(ctx.has_all_permissions(&["sales:create"]));
        assert!(ctx.has_all_permissions(&[])); // Empty list = all present
    }

    #[test]
    fn test_has_all_permissions_returns_false_when_any_missing() {
        let ctx = create_test_context(&["sales:create", "sales:view"]);

        assert!(!ctx.has_all_permissions(&["sales:create", "sales:delete"]));
        assert!(!ctx.has_all_permissions(&["inventory:view"]));
    }

    #[test]
    fn test_has_any_permission_returns_true_when_any_present() {
        let ctx = create_test_context(&["sales:create", "sales:view"]);

        assert!(ctx.has_any_permission(&["sales:create", "inventory:view"]));
        assert!(ctx.has_any_permission(&["sales:create"]));
    }

    #[test]
    fn test_has_any_permission_returns_false_when_none_present() {
        let ctx = create_test_context(&["sales:create"]);

        assert!(!ctx.has_any_permission(&["sales:delete", "inventory:view"]));
    }

    #[test]
    fn test_has_any_permission_returns_false_for_empty_list() {
        let ctx = create_test_context(&["sales:create"]);

        assert!(!ctx.has_any_permission(&[]));
    }

    #[test]
    fn test_check_permissions_returns_granted_when_all_present() {
        let ctx = create_test_context(&["sales:create", "sales:view"]);

        let result = ctx.check_permissions(&["sales:create", "sales:view"]);
        assert_eq!(result, PermissionCheckResult::Granted);
    }

    #[test]
    fn test_check_permissions_returns_granted_for_empty_list() {
        let ctx = create_test_context(&["sales:create"]);

        let result = ctx.check_permissions(&[]);
        assert_eq!(result, PermissionCheckResult::Granted);
    }

    #[test]
    fn test_check_permissions_returns_denied_with_missing() {
        let ctx = create_test_context(&["sales:create"]);

        let result = ctx.check_permissions(&["sales:create", "sales:delete", "inventory:view"]);

        match result {
            PermissionCheckResult::Denied { missing } => {
                assert_eq!(missing.len(), 2);
                assert!(missing.contains(&"sales:delete".to_string()));
                assert!(missing.contains(&"inventory:view".to_string()));
            }
            PermissionCheckResult::Granted => panic!("Expected Denied result"),
        }
    }

    #[test]
    fn test_check_permissions_includes_invalid_format_as_missing() {
        let ctx = create_test_context(&["sales:create"]);

        let result = ctx.check_permissions(&["sales:create", "invalid"]);

        match result {
            PermissionCheckResult::Denied { missing } => {
                assert_eq!(missing.len(), 1);
                assert!(missing.contains(&"invalid".to_string()));
            }
            PermissionCheckResult::Granted => panic!("Expected Denied result"),
        }
    }
}
