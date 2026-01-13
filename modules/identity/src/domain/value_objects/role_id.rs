// RoleId value object - typed wrapper around Uuid for role identification

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Typed ID for roles, wrapping a UUID v7 for type safety and temporal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(Uuid);

impl RoleId {
    /// Creates a new RoleId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a RoleId from an existing UUID
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

impl Default for RoleId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for RoleId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<RoleId> for Uuid {
    fn from(id: RoleId) -> Self {
        id.0
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_id_new_generates_unique_ids() {
        let id1 = RoleId::new();
        let id2 = RoleId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_role_id_from_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let role_id = RoleId::from_uuid(uuid);
        assert_eq!(*role_id.as_uuid(), uuid);
    }

    #[test]
    fn test_role_id_into_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let role_id = RoleId::from_uuid(uuid);
        assert_eq!(role_id.into_uuid(), uuid);
    }

    #[test]
    fn test_role_id_equality() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id1 = RoleId::from_uuid(uuid);
        let id2 = RoleId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }
}
