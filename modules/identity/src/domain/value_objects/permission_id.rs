// PermissionId value object - typed wrapper around Uuid for permission identification

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Typed ID for permissions, wrapping a UUID v7 for type safety and temporal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionId(Uuid);

impl PermissionId {
    /// Creates a new PermissionId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a PermissionId from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID value
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Consumes self and returns the inner UUID
    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for PermissionId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for PermissionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<PermissionId> for Uuid {
    fn from(id: PermissionId) -> Self {
        id.0
    }
}

impl std::fmt::Display for PermissionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_id_new_generates_unique_ids() {
        let id1 = PermissionId::new();
        let id2 = PermissionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_permission_id_from_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let permission_id = PermissionId::from_uuid(uuid);
        assert_eq!(*permission_id.as_uuid(), uuid);
    }

    #[test]
    fn test_permission_id_into_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let permission_id = PermissionId::from_uuid(uuid);
        assert_eq!(permission_id.into_uuid(), uuid);
    }

    #[test]
    fn test_permission_id_equality() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id1 = PermissionId::from_uuid(uuid);
        let id2 = PermissionId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }
}
