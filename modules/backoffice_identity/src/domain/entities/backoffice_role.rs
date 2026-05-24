use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::BackofficeRoleId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackofficeRole {
    id: BackofficeRoleId,
    name: String,
    description: Option<String>,
    is_system_protected: bool,
    created_at: DateTime<Utc>,
}

impl BackofficeRole {
    pub fn new(
        id: BackofficeRoleId,
        name: String,
        description: Option<String>,
        is_system_protected: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            is_system_protected,
            created_at,
        }
    }

    pub fn create(name: String, description: Option<String>, is_system_protected: bool) -> Self {
        Self {
            id: BackofficeRoleId::new(),
            name,
            description,
            is_system_protected,
            created_at: Utc::now(),
        }
    }

    pub fn id(&self) -> &BackofficeRoleId {
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
}

impl PartialEq for BackofficeRole {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BackofficeRole {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_role() {
        let role = BackofficeRole::create(
            "super_admin".to_string(),
            Some("Full platform access".to_string()),
            true,
        );
        assert_eq!(role.name(), "super_admin");
        assert_eq!(role.description(), Some("Full platform access"));
        assert!(role.is_system_protected());
    }

    #[test]
    fn test_is_system_protected_field() {
        let protected = BackofficeRole::create("super_admin".to_string(), None, true);
        let not_protected = BackofficeRole::create("support_readonly".to_string(), None, false);
        assert!(protected.is_system_protected());
        assert!(!not_protected.is_system_protected());
    }

    #[test]
    fn test_description_optional() {
        let role = BackofficeRole::create("billing_admin".to_string(), None, false);
        assert!(role.description().is_none());
    }

    #[test]
    fn test_equality_by_id() {
        let role1 = BackofficeRole::create("super_admin".to_string(), None, true);
        let role2 = BackofficeRole::new(
            *role1.id(),
            "different_name".to_string(),
            None,
            false,
            Utc::now(),
        );
        assert_eq!(role1, role2);
    }

    #[test]
    fn test_inequality_different_ids() {
        let role1 = BackofficeRole::create("super_admin".to_string(), None, true);
        let role2 = BackofficeRole::create("super_admin".to_string(), None, true);
        assert_ne!(role1, role2);
    }
}
