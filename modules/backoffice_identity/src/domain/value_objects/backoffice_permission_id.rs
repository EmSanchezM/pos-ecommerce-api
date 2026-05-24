use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackofficePermissionId(Uuid);

impl BackofficePermissionId {
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

impl Default for BackofficePermissionId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for BackofficePermissionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<BackofficePermissionId> for Uuid {
    fn from(id: BackofficePermissionId) -> Self {
        id.0
    }
}

impl std::fmt::Display for BackofficePermissionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generates_unique_ids() {
        let id1 = BackofficePermissionId::new();
        let id2 = BackofficePermissionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_from_uuid_roundtrip() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id = BackofficePermissionId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
    }

    #[test]
    fn test_equality() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id1 = BackofficePermissionId::from_uuid(uuid);
        let id2 = BackofficePermissionId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }
}
