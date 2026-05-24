use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackofficeUserId(Uuid);

impl BackofficeUserId {
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

impl Default for BackofficeUserId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for BackofficeUserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<BackofficeUserId> for Uuid {
    fn from(id: BackofficeUserId) -> Self {
        id.0
    }
}

impl std::fmt::Display for BackofficeUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generates_unique_ids() {
        let id1 = BackofficeUserId::new();
        let id2 = BackofficeUserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_from_uuid_roundtrip() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id = BackofficeUserId::from_uuid(uuid);
        assert_eq!(*id.as_uuid(), uuid);
        assert_eq!(id.into_uuid(), uuid);
    }

    #[test]
    fn test_equality() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id1 = BackofficeUserId::from_uuid(uuid);
        let id2 = BackofficeUserId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_display() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id = BackofficeUserId::from_uuid(uuid);
        assert_eq!(format!("{}", id), format!("{}", uuid));
    }

    #[test]
    fn test_from_and_into_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id: BackofficeUserId = uuid.into();
        let back: Uuid = id.into();
        assert_eq!(back, uuid);
    }
}
