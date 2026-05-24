use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{BackofficePermissionId, PlatformPermissionCode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackofficePermission {
    id: BackofficePermissionId,
    code: PlatformPermissionCode,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

impl BackofficePermission {
    pub fn new(
        id: BackofficePermissionId,
        code: PlatformPermissionCode,
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

    pub fn create(code: PlatformPermissionCode, description: Option<String>) -> Self {
        Self {
            id: BackofficePermissionId::new(),
            code,
            description,
            created_at: Utc::now(),
        }
    }

    pub fn id(&self) -> &BackofficePermissionId {
        &self.id
    }

    pub fn code(&self) -> &PlatformPermissionCode {
        &self.code
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl PartialEq for BackofficePermission {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BackofficePermission {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_code(s: &str) -> PlatformPermissionCode {
        PlatformPermissionCode::new(s).unwrap()
    }

    #[test]
    fn test_create_permission() {
        let perm = BackofficePermission::create(
            make_code("platform:org.suspend"),
            Some("Suspend an organization".to_string()),
        );
        assert_eq!(perm.code().as_str(), "platform:org.suspend");
        assert_eq!(perm.description(), Some("Suspend an organization"));
    }

    #[test]
    fn test_permission_code_must_match_format() {
        let result = PlatformPermissionCode::new("system:admin");
        assert!(result.is_err());
    }

    #[test]
    fn test_description_optional() {
        let perm = BackofficePermission::create(make_code("platform:audit.read"), None);
        assert!(perm.description().is_none());
    }

    #[test]
    fn test_equality_by_id() {
        let perm1 = BackofficePermission::create(make_code("platform:org.list"), None);
        let perm2 = BackofficePermission::new(
            *perm1.id(),
            make_code("platform:org.suspend"),
            None,
            Utc::now(),
        );
        assert_eq!(perm1, perm2);
    }

    #[test]
    fn test_inequality_different_ids() {
        let perm1 = BackofficePermission::create(make_code("platform:org.list"), None);
        let perm2 = BackofficePermission::create(make_code("platform:org.list"), None);
        assert_ne!(perm1, perm2);
    }
}
