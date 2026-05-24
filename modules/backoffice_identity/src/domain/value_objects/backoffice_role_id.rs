use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackofficeRoleId(Uuid);

impl BackofficeRoleId {
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn into_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for BackofficeRoleId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for BackofficeRoleId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<BackofficeRoleId> for Uuid {
    fn from(id: BackofficeRoleId) -> Self {
        id.0
    }
}

impl std::fmt::Display for BackofficeRoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generates_unique_ids() {
        let id1 = BackofficeRoleId::new();
        let id2 = BackofficeRoleId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_from_uuid_roundtrip() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id = BackofficeRoleId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn test_equality() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id1 = BackofficeRoleId::from_uuid(uuid);
        let id2 = BackofficeRoleId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }
}
