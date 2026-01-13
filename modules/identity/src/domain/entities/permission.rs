// Permission entity - represents a system permission with module:action format

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{PermissionCode, PermissionId};

/// Permission entity representing a specific action that can be performed
///
/// Permissions follow the format `module:action` (e.g., `sales:create_invoice`).
/// They are assigned to roles, which are then assigned to users per store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    id: PermissionId,
    code: PermissionCode,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

impl Permission {
    /// Creates a new Permission with all fields specified
    pub fn new(
        id: PermissionId,
        code: PermissionCode,
        description: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            code,
            description,
            created_at,
        }
    }

    /// Creates a new Permission with current timestamp
    pub fn create(code: PermissionCode, description: Option<String>) -> Self {
        Self {
            id: PermissionId::new(),
            code,
            description,
            created_at: Utc::now(),
        }
    }

    // Getters

    pub fn id(&self) -> &PermissionId {
        &self.id
    }

    pub fn code(&self) -> &PermissionCode {
        &self.code
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the module part of the permission code
    pub fn module(&self) -> &str {
        self.code.module()
    }

    /// Returns the action part of the permission code
    pub fn action(&self) -> &str {
        self.code.action()
    }
}

impl PartialEq for Permission {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Permission {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_permission() -> Permission {
        let code = PermissionCode::new("sales:create_invoice").unwrap();
        Permission::create(code, Some("Create sales invoices".to_string()))
    }

    #[test]
    fn test_permission_create() {
        let permission = create_test_permission();

        assert_eq!(permission.code().as_str(), "sales:create_invoice");
        assert_eq!(permission.description(), Some("Create sales invoices"));
        assert_eq!(permission.module(), "sales");
        assert_eq!(permission.action(), "create_invoice");
    }

    #[test]
    fn test_permission_create_without_description() {
        let code = PermissionCode::new("inventory:view_stock").unwrap();
        let permission = Permission::create(code, None);

        assert_eq!(permission.code().as_str(), "inventory:view_stock");
        assert_eq!(permission.description(), None);
    }

    #[test]
    fn test_permission_module_and_action() {
        let code = PermissionCode::new("purchasing:approve_order").unwrap();
        let permission = Permission::create(code, None);

        assert_eq!(permission.module(), "purchasing");
        assert_eq!(permission.action(), "approve_order");
    }

    #[test]
    fn test_permission_equality_by_id() {
        let permission1 = create_test_permission();
        let code2 = PermissionCode::new("different:action").unwrap();
        let permission2 = Permission::new(
            *permission1.id(),
            code2,
            Some("Different description".to_string()),
            Utc::now(),
        );

        // Permissions are equal if they have the same ID
        assert_eq!(permission1, permission2);
    }

    #[test]
    fn test_permission_inequality_different_ids() {
        let permission1 = create_test_permission();
        let permission2 = create_test_permission();

        // Different IDs mean different permissions
        assert_ne!(permission1, permission2);
    }
}
