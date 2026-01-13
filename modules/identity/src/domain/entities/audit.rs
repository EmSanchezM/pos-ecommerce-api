// AuditEntry entity - records changes to permissions, roles, and memberships

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::value_objects::UserId;

/// Actions that can be audited in the identity module
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    /// Entity was created
    Created,
    /// Entity was updated
    Updated,
    /// Entity was deleted
    Deleted,
    /// Permission was added to a role
    PermissionAdded,
    /// Permission was removed from a role
    PermissionRemoved,
    /// Role was assigned to a user in a store
    RoleAssigned,
    /// Role was unassigned from a user in a store
    RoleUnassigned,
    /// User was added to a store
    UserAddedToStore,
    /// User was removed from a store
    UserRemovedFromStore,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Created => write!(f, "created"),
            AuditAction::Updated => write!(f, "updated"),
            AuditAction::Deleted => write!(f, "deleted"),
            AuditAction::PermissionAdded => write!(f, "permission_added"),
            AuditAction::PermissionRemoved => write!(f, "permission_removed"),
            AuditAction::RoleAssigned => write!(f, "role_assigned"),
            AuditAction::RoleUnassigned => write!(f, "role_unassigned"),
            AuditAction::UserAddedToStore => write!(f, "user_added_to_store"),
            AuditAction::UserRemovedFromStore => write!(f, "user_removed_from_store"),
        }
    }
}

/// Audit entry recording a change in the identity module
///
/// Captures who made what change, when, and the before/after state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    id: Uuid,
    entity_type: String,
    entity_id: Uuid,
    action: AuditAction,
    old_value: Option<serde_json::Value>,
    new_value: Option<serde_json::Value>,
    actor_id: UserId,
    created_at: DateTime<Utc>,
}

impl AuditEntry {
    /// Creates a new AuditEntry with all fields specified
    pub fn new(
        id: Uuid,
        entity_type: String,
        entity_id: Uuid,
        action: AuditAction,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        actor_id: UserId,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            entity_type,
            entity_id,
            action,
            old_value,
            new_value,
            actor_id,
            created_at,
        }
    }

    /// Creates a new AuditEntry with auto-generated ID and current timestamp
    pub fn create(
        entity_type: String,
        entity_id: Uuid,
        action: AuditAction,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        actor_id: UserId,
    ) -> Self {
        Self {
            id: Uuid::new_v7(Timestamp::now(NoContext)),
            entity_type,
            entity_id,
            action,
            old_value,
            new_value,
            actor_id,
            created_at: Utc::now(),
        }
    }

    /// Creates an audit entry for entity creation
    pub fn for_create<T: Serialize>(
        entity_type: &str,
        entity_id: Uuid,
        new_value: &T,
        actor_id: UserId,
    ) -> Self {
        Self::create(
            entity_type.to_string(),
            entity_id,
            AuditAction::Created,
            None,
            Some(serde_json::to_value(new_value).unwrap_or(serde_json::Value::Null)),
            actor_id,
        )
    }

    /// Creates an audit entry for entity update
    pub fn for_update<T: Serialize>(
        entity_type: &str,
        entity_id: Uuid,
        old_value: &T,
        new_value: &T,
        actor_id: UserId,
    ) -> Self {
        Self::create(
            entity_type.to_string(),
            entity_id,
            AuditAction::Updated,
            Some(serde_json::to_value(old_value).unwrap_or(serde_json::Value::Null)),
            Some(serde_json::to_value(new_value).unwrap_or(serde_json::Value::Null)),
            actor_id,
        )
    }

    /// Creates an audit entry for entity deletion
    pub fn for_delete<T: Serialize>(
        entity_type: &str,
        entity_id: Uuid,
        old_value: &T,
        actor_id: UserId,
    ) -> Self {
        Self::create(
            entity_type.to_string(),
            entity_id,
            AuditAction::Deleted,
            Some(serde_json::to_value(old_value).unwrap_or(serde_json::Value::Null)),
            None,
            actor_id,
        )
    }

    // Getters

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }

    pub fn entity_id(&self) -> Uuid {
        self.entity_id
    }

    pub fn action(&self) -> &AuditAction {
        &self.action
    }

    pub fn old_value(&self) -> Option<&serde_json::Value> {
        self.old_value.as_ref()
    }

    pub fn new_value(&self) -> Option<&serde_json::Value> {
        self.new_value.as_ref()
    }

    pub fn actor_id(&self) -> &UserId {
        &self.actor_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl PartialEq for AuditEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AuditEntry {}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    #[derive(Serialize, Deserialize)]
    struct TestEntity {
        name: String,
        value: i32,
    }

    #[test]
    fn test_audit_action_display() {
        assert_eq!(format!("{}", AuditAction::Created), "created");
        assert_eq!(format!("{}", AuditAction::Updated), "updated");
        assert_eq!(format!("{}", AuditAction::Deleted), "deleted");
        assert_eq!(format!("{}", AuditAction::PermissionAdded), "permission_added");
        assert_eq!(format!("{}", AuditAction::PermissionRemoved), "permission_removed");
        assert_eq!(format!("{}", AuditAction::RoleAssigned), "role_assigned");
        assert_eq!(format!("{}", AuditAction::RoleUnassigned), "role_unassigned");
        assert_eq!(format!("{}", AuditAction::UserAddedToStore), "user_added_to_store");
        assert_eq!(format!("{}", AuditAction::UserRemovedFromStore), "user_removed_from_store");
    }

    #[test]
    fn test_audit_entry_create() {
        let actor_id = UserId::new();
        let entity_id = new_uuid();

        let entry = AuditEntry::create(
            "role".to_string(),
            entity_id,
            AuditAction::Created,
            None,
            Some(serde_json::json!({"name": "Admin"})),
            actor_id,
        );

        assert_eq!(entry.entity_type(), "role");
        assert_eq!(entry.entity_id(), entity_id);
        assert_eq!(entry.action(), &AuditAction::Created);
        assert!(entry.old_value().is_none());
        assert!(entry.new_value().is_some());
        assert_eq!(entry.actor_id(), &actor_id);
    }

    #[test]
    fn test_audit_entry_for_create() {
        let actor_id = UserId::new();
        let entity_id = new_uuid();
        let entity = TestEntity {
            name: "Test".to_string(),
            value: 42,
        };

        let entry = AuditEntry::for_create("test_entity", entity_id, &entity, actor_id);

        assert_eq!(entry.entity_type(), "test_entity");
        assert_eq!(entry.action(), &AuditAction::Created);
        assert!(entry.old_value().is_none());
        assert!(entry.new_value().is_some());

        let new_val = entry.new_value().unwrap();
        assert_eq!(new_val["name"], "Test");
        assert_eq!(new_val["value"], 42);
    }

    #[test]
    fn test_audit_entry_for_update() {
        let actor_id = UserId::new();
        let entity_id = new_uuid();
        let old_entity = TestEntity {
            name: "Old".to_string(),
            value: 1,
        };
        let new_entity = TestEntity {
            name: "New".to_string(),
            value: 2,
        };

        let entry = AuditEntry::for_update("test_entity", entity_id, &old_entity, &new_entity, actor_id);

        assert_eq!(entry.action(), &AuditAction::Updated);
        assert!(entry.old_value().is_some());
        assert!(entry.new_value().is_some());

        let old_val = entry.old_value().unwrap();
        let new_val = entry.new_value().unwrap();
        assert_eq!(old_val["name"], "Old");
        assert_eq!(new_val["name"], "New");
    }

    #[test]
    fn test_audit_entry_for_delete() {
        let actor_id = UserId::new();
        let entity_id = new_uuid();
        let entity = TestEntity {
            name: "ToDelete".to_string(),
            value: 99,
        };

        let entry = AuditEntry::for_delete("test_entity", entity_id, &entity, actor_id);

        assert_eq!(entry.action(), &AuditAction::Deleted);
        assert!(entry.old_value().is_some());
        assert!(entry.new_value().is_none());
    }

    #[test]
    fn test_audit_entry_equality_by_id() {
        let actor_id = UserId::new();
        let id = new_uuid();
        let entity_id = new_uuid();

        let entry1 = AuditEntry::new(
            id,
            "role".to_string(),
            entity_id,
            AuditAction::Created,
            None,
            None,
            actor_id,
            Utc::now(),
        );

        let entry2 = AuditEntry::new(
            id,
            "permission".to_string(), // Different entity type
            new_uuid(),               // Different entity ID
            AuditAction::Deleted,     // Different action
            None,
            None,
            UserId::new(), // Different actor
            Utc::now(),
        );

        // Entries are equal if they have the same ID
        assert_eq!(entry1, entry2);
    }

    #[test]
    fn test_audit_entry_inequality_different_ids() {
        let actor_id = UserId::new();
        let entity_id = new_uuid();

        let entry1 = AuditEntry::create(
            "role".to_string(),
            entity_id,
            AuditAction::Created,
            None,
            None,
            actor_id,
        );

        let entry2 = AuditEntry::create(
            "role".to_string(),
            entity_id,
            AuditAction::Created,
            None,
            None,
            actor_id,
        );

        // Different IDs mean different entries
        assert_ne!(entry1, entry2);
    }
}
