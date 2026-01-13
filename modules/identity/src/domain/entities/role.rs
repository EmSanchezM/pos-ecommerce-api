// Role entity - represents a named collection of permissions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::RoleId;

/// Role entity representing a named collection of permissions
///
/// Roles group permissions together for easier assignment to users.
/// System-protected roles cannot be deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    id: RoleId,
    name: String,
    description: Option<String>,
    is_system_protected: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Role {
    /// Creates a new Role with all fields specified
    pub fn new(
        id: RoleId,
        name: String,
        description: Option<String>,
        is_system_protected: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            is_system_protected,
            created_at,
            updated_at,
        }
    }

    /// Creates a new non-protected Role with current timestamps
    pub fn create(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: RoleId::new(),
            name,
            description,
            is_system_protected: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new system-protected Role with current timestamps
    pub fn create_protected(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: RoleId::new(),
            name,
            description,
            is_system_protected: true,
            created_at: now,
            updated_at: now,
        }
    }

    // Getters

    pub fn id(&self) -> &RoleId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn is_system_protected(&self) -> bool {
        self.is_system_protected
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Setters / Mutators

    /// Updates the role name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Updates the role description
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Checks if this role can be deleted
    pub fn can_delete(&self) -> bool {
        !self.is_system_protected
    }
}

impl PartialEq for Role {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Role {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_create() {
        let role = Role::create("Admin".to_string(), Some("Administrator role".to_string()));

        assert_eq!(role.name(), "Admin");
        assert_eq!(role.description(), Some("Administrator role"));
        assert!(!role.is_system_protected());
        assert!(role.can_delete());
    }

    #[test]
    fn test_role_create_without_description() {
        let role = Role::create("Viewer".to_string(), None);

        assert_eq!(role.name(), "Viewer");
        assert_eq!(role.description(), None);
    }

    #[test]
    fn test_role_create_protected() {
        let role = Role::create_protected("SuperAdmin".to_string(), Some("System admin".to_string()));

        assert_eq!(role.name(), "SuperAdmin");
        assert!(role.is_system_protected());
        assert!(!role.can_delete());
    }

    #[test]
    fn test_role_set_name() {
        let mut role = Role::create("Old Name".to_string(), None);
        let original_updated = role.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(10));
        role.set_name("New Name".to_string());

        assert_eq!(role.name(), "New Name");
        assert!(role.updated_at() > original_updated);
    }

    #[test]
    fn test_role_set_description() {
        let mut role = Role::create("Role".to_string(), None);
        assert_eq!(role.description(), None);

        role.set_description(Some("New description".to_string()));
        assert_eq!(role.description(), Some("New description"));

        role.set_description(None);
        assert_eq!(role.description(), None);
    }

    #[test]
    fn test_role_equality_by_id() {
        let role1 = Role::create("Role 1".to_string(), None);
        let role2 = Role::new(
            *role1.id(),
            "Different Name".to_string(),
            Some("Different description".to_string()),
            true,
            Utc::now(),
            Utc::now(),
        );

        // Roles are equal if they have the same ID
        assert_eq!(role1, role2);
    }

    #[test]
    fn test_role_inequality_different_ids() {
        let role1 = Role::create("Role".to_string(), None);
        let role2 = Role::create("Role".to_string(), None);

        // Different IDs mean different roles
        assert_ne!(role1, role2);
    }
}
