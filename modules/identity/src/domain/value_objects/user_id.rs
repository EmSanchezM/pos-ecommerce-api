// UserId value object - typed wrapper around Uuid for user identification

use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

/// Typed ID for users, wrapping a UUID v7 for type safety and temporal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// Creates a new UserId with a UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::new_v7(Timestamp::now(NoContext)))
    }

    /// Creates a UserId from an existing UUID
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

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<UserId> for Uuid {
    fn from(id: UserId) -> Self {
        id.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_new_generates_unique_ids() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_user_id_from_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let user_id = UserId::from_uuid(uuid);
        assert_eq!(*user_id.as_uuid(), uuid);
    }

    #[test]
    fn test_user_id_into_uuid() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let user_id = UserId::from_uuid(uuid);
        assert_eq!(user_id.into_uuid(), uuid);
    }

    #[test]
    fn test_user_id_equality() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let id1 = UserId::from_uuid(uuid);
        let id2 = UserId::from_uuid(uuid);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_user_id_display() {
        let uuid = Uuid::new_v7(Timestamp::now(NoContext));
        let user_id = UserId::from_uuid(uuid);
        assert_eq!(format!("{}", user_id), format!("{}", uuid));
    }
}
