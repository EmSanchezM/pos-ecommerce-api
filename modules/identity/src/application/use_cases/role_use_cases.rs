// Role use cases - Application layer business logic for role management
//
// Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6

use std::sync::Arc;

use crate::domain::entities::{AuditAction, AuditEntry, Role};
use crate::domain::repositories::{AuditRepository, PermissionRepository, RoleRepository, UserRepository};
use crate::domain::value_objects::{PermissionId, RoleId, UserId};
use crate::error::IdentityError;

// =============================================================================
// CreateRoleUseCase
// =============================================================================

/// Use case for creating a new role
///
/// Validates the role name uniqueness, saves to repository,
/// and creates an audit entry.
///
/// Requirements: 2.1, 2.2
pub struct CreateRoleUseCase<R, A>
where
    R: RoleRepository,
    A: AuditRepository,
{
    role_repo: Arc<R>,
    audit_repo: Arc<A>,
}

impl<R, A> CreateRoleUseCase<R, A>
where
    R: RoleRepository,
    A: AuditRepository,
{
    /// Creates a new instance of CreateRoleUseCase
    pub fn new(role_repo: Arc<R>, audit_repo: Arc<A>) -> Self {
        Self {
            role_repo,
            audit_repo,
        }
    }

    /// Executes the use case to create a new role
    ///
    /// # Arguments
    /// * `name` - Unique name for the role
    /// * `description` - Optional description of the role
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The created Role on success
    ///
    /// # Errors
    /// * `IdentityError::DuplicateRole` - If a role with the same name exists
    pub async fn execute(
        &self,
        name: &str,
        description: Option<String>,
        actor_id: UserId,
    ) -> Result<Role, IdentityError> {
        // Check for uniqueness (Requirement 2.1)
        if self.role_repo.find_by_name(name).await?.is_some() {
            return Err(IdentityError::DuplicateRole(name.to_string()));
        }

        // Create and save the role (Requirement 2.2 - stored in database)
        let role = Role::create(name.to_string(), description);
        self.role_repo.save(&role).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_create(
            "role",
            role.id().into_uuid(),
            &role,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(role)
    }
}


// =============================================================================
// DeleteRoleUseCase
// =============================================================================

/// Use case for deleting a role
///
/// Verifies the role is not system-protected, removes all user-role assignments,
/// and creates an audit entry.
///
/// Requirements: 2.5, 2.6
pub struct DeleteRoleUseCase<R, U, A>
where
    R: RoleRepository,
    U: UserRepository,
    A: AuditRepository,
{
    role_repo: Arc<R>,
    user_repo: Arc<U>,
    audit_repo: Arc<A>,
}

impl<R, U, A> DeleteRoleUseCase<R, U, A>
where
    R: RoleRepository,
    U: UserRepository,
    A: AuditRepository,
{
    /// Creates a new instance of DeleteRoleUseCase
    pub fn new(role_repo: Arc<R>, user_repo: Arc<U>, audit_repo: Arc<A>) -> Self {
        Self {
            role_repo,
            user_repo,
            audit_repo,
        }
    }

    /// Executes the use case to delete a role
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to delete
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::RoleNotFound` - If role doesn't exist
    /// * `IdentityError::ProtectedRoleCannotBeDeleted` - If role is system-protected
    pub async fn execute(
        &self,
        role_id: RoleId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Find the role first
        let role = self
            .role_repo
            .find_by_id(role_id)
            .await?
            .ok_or(IdentityError::RoleNotFound(role_id.into_uuid()))?;

        // Check if role is system-protected (Requirement 2.6)
        if role.is_system_protected() {
            return Err(IdentityError::ProtectedRoleCannotBeDeleted);
        }

        // Remove role from all users first (Requirement 2.5)
        self.user_repo.remove_role_from_all_users(role_id).await?;

        // Delete the role
        self.role_repo.delete(role_id).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_delete(
            "role",
            role_id.into_uuid(),
            &role,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}


// =============================================================================
// AddPermissionToRoleUseCase
// =============================================================================

/// Use case for adding a permission to a role
///
/// Validates that both role and permission exist, then adds the permission
/// to the role and creates an audit entry.
///
/// Requirements: 2.3
pub struct AddPermissionToRoleUseCase<R, P, A>
where
    R: RoleRepository,
    P: PermissionRepository,
    A: AuditRepository,
{
    role_repo: Arc<R>,
    permission_repo: Arc<P>,
    audit_repo: Arc<A>,
}

impl<R, P, A> AddPermissionToRoleUseCase<R, P, A>
where
    R: RoleRepository,
    P: PermissionRepository,
    A: AuditRepository,
{
    /// Creates a new instance of AddPermissionToRoleUseCase
    pub fn new(role_repo: Arc<R>, permission_repo: Arc<P>, audit_repo: Arc<A>) -> Self {
        Self {
            role_repo,
            permission_repo,
            audit_repo,
        }
    }

    /// Executes the use case to add a permission to a role
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to add permission to
    /// * `permission_id` - ID of the permission to add
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::RoleNotFound` - If role doesn't exist
    /// * `IdentityError::PermissionNotFound` - If permission doesn't exist
    pub async fn execute(
        &self,
        role_id: RoleId,
        permission_id: PermissionId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Verify role exists
        let role = self
            .role_repo
            .find_by_id(role_id)
            .await?
            .ok_or(IdentityError::RoleNotFound(role_id.into_uuid()))?;

        // Verify permission exists
        let permission = self
            .permission_repo
            .find_by_id(permission_id)
            .await?
            .ok_or(IdentityError::PermissionNotFound(permission_id.into_uuid()))?;

        // Add permission to role (Requirement 2.3)
        self.role_repo.add_permission(role_id, permission_id).await?;

        // Create audit entry for permission addition
        let audit_entry = AuditEntry::create(
            "role".to_string(),
            role_id.into_uuid(),
            AuditAction::PermissionAdded,
            None,
            Some(serde_json::json!({
                "role_id": role_id.into_uuid(),
                "role_name": role.name(),
                "permission_id": permission_id.into_uuid(),
                "permission_code": permission.code().as_str()
            })),
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}


// =============================================================================
// RemovePermissionFromRoleUseCase
// =============================================================================

/// Use case for removing a permission from a role
///
/// Validates that the role exists, then removes the permission
/// and creates an audit entry.
///
/// Requirements: 2.4
pub struct RemovePermissionFromRoleUseCase<R, P, A>
where
    R: RoleRepository,
    P: PermissionRepository,
    A: AuditRepository,
{
    role_repo: Arc<R>,
    permission_repo: Arc<P>,
    audit_repo: Arc<A>,
}

impl<R, P, A> RemovePermissionFromRoleUseCase<R, P, A>
where
    R: RoleRepository,
    P: PermissionRepository,
    A: AuditRepository,
{
    /// Creates a new instance of RemovePermissionFromRoleUseCase
    pub fn new(role_repo: Arc<R>, permission_repo: Arc<P>, audit_repo: Arc<A>) -> Self {
        Self {
            role_repo,
            permission_repo,
            audit_repo,
        }
    }

    /// Executes the use case to remove a permission from a role
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to remove permission from
    /// * `permission_id` - ID of the permission to remove
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::RoleNotFound` - If role doesn't exist
    pub async fn execute(
        &self,
        role_id: RoleId,
        permission_id: PermissionId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Verify role exists
        let role = self
            .role_repo
            .find_by_id(role_id)
            .await?
            .ok_or(IdentityError::RoleNotFound(role_id.into_uuid()))?;

        // Get permission info for audit (if it exists)
        let permission_info = self
            .permission_repo
            .find_by_id(permission_id)
            .await?
            .map(|p| p.code().as_str().to_string());

        // Remove permission from role (Requirement 2.4)
        self.role_repo.remove_permission(role_id, permission_id).await?;

        // Create audit entry for permission removal
        let audit_entry = AuditEntry::create(
            "role".to_string(),
            role_id.into_uuid(),
            AuditAction::PermissionRemoved,
            Some(serde_json::json!({
                "role_id": role_id.into_uuid(),
                "role_name": role.name(),
                "permission_id": permission_id.into_uuid(),
                "permission_code": permission_info
            })),
            None,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}


// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::entities::{Permission, Store, User};
    use crate::domain::value_objects::{PermissionCode, StoreId};

    // Mock RoleRepository for testing
    struct MockRoleRepository {
        roles: Mutex<HashMap<RoleId, Role>>,
        role_permissions: Mutex<HashMap<RoleId, Vec<PermissionId>>>,
    }

    impl MockRoleRepository {
        fn new() -> Self {
            Self {
                roles: Mutex::new(HashMap::new()),
                role_permissions: Mutex::new(HashMap::new()),
            }
        }

        fn with_role(self, role: Role) -> Self {
            self.roles.lock().unwrap().insert(*role.id(), role);
            self
        }
    }

    #[async_trait]
    impl RoleRepository for MockRoleRepository {
        async fn save(&self, role: &Role) -> Result<(), IdentityError> {
            let mut roles = self.roles.lock().unwrap();
            roles.insert(*role.id(), role.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: RoleId) -> Result<Option<Role>, IdentityError> {
            let roles = self.roles.lock().unwrap();
            Ok(roles.get(&id).cloned())
        }

        async fn find_by_name(&self, name: &str) -> Result<Option<Role>, IdentityError> {
            let roles = self.roles.lock().unwrap();
            Ok(roles.values().find(|r| r.name() == name).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Role>, IdentityError> {
            let roles = self.roles.lock().unwrap();
            Ok(roles.values().cloned().collect())
        }

        async fn delete(&self, id: RoleId) -> Result<(), IdentityError> {
            let mut roles = self.roles.lock().unwrap();
            roles.remove(&id);
            let mut perms = self.role_permissions.lock().unwrap();
            perms.remove(&id);
            Ok(())
        }

        async fn update(&self, role: &Role) -> Result<(), IdentityError> {
            let mut roles = self.roles.lock().unwrap();
            roles.insert(*role.id(), role.clone());
            Ok(())
        }

        async fn add_permission(
            &self,
            role_id: RoleId,
            permission_id: PermissionId,
        ) -> Result<(), IdentityError> {
            let mut perms = self.role_permissions.lock().unwrap();
            perms.entry(role_id).or_default().push(permission_id);
            Ok(())
        }

        async fn remove_permission(
            &self,
            role_id: RoleId,
            permission_id: PermissionId,
        ) -> Result<(), IdentityError> {
            let mut perms = self.role_permissions.lock().unwrap();
            if let Some(perm_list) = perms.get_mut(&role_id) {
                perm_list.retain(|p| *p != permission_id);
            }
            Ok(())
        }

        async fn get_permissions(&self, _role_id: RoleId) -> Result<Vec<Permission>, IdentityError> {
            Ok(vec![])
        }

        async fn remove_permission_from_all_roles(
            &self,
            permission_id: PermissionId,
        ) -> Result<(), IdentityError> {
            let mut perms = self.role_permissions.lock().unwrap();
            for perm_list in perms.values_mut() {
                perm_list.retain(|p| *p != permission_id);
            }
            Ok(())
        }
    }

    // Mock AuditRepository for testing
    struct MockAuditRepository {
        entries: Mutex<Vec<AuditEntry>>,
    }

    impl MockAuditRepository {
        fn new() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
            }
        }

        fn get_entries(&self) -> Vec<AuditEntry> {
            self.entries.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl AuditRepository for MockAuditRepository {
        async fn save(&self, entry: &AuditEntry) -> Result<(), IdentityError> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
            Ok(())
        }

        async fn find_by_entity(
            &self,
            entity_type: &str,
            entity_id: Uuid,
        ) -> Result<Vec<AuditEntry>, IdentityError> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .iter()
                .filter(|e| e.entity_type() == entity_type && e.entity_id() == entity_id)
                .cloned()
                .collect())
        }

        async fn find_by_date_range(
            &self,
            from: DateTime<Utc>,
            to: DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, IdentityError> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .iter()
                .filter(|e| e.created_at() >= from && e.created_at() < to)
                .cloned()
                .collect())
        }
    }

    // Mock UserRepository for testing
    struct MockUserRepository {
        removed_roles: Mutex<Vec<RoleId>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                removed_roles: Mutex::new(Vec::new()),
            }
        }

        fn get_removed_roles(&self) -> Vec<RoleId> {
            self.removed_roles.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, _user: &User) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn find_by_id(&self, _id: UserId) -> Result<Option<User>, IdentityError> {
            Ok(None)
        }

        async fn find_by_email(&self, _email: &crate::domain::value_objects::Email) -> Result<Option<User>, IdentityError> {
            Ok(None)
        }

        async fn find_by_username(&self, _username: &crate::domain::value_objects::Username) -> Result<Option<User>, IdentityError> {
            Ok(None)
        }

        async fn update(&self, _user: &User) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn assign_role(
            &self,
            _user_id: UserId,
            _role_id: RoleId,
            _store_id: StoreId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn remove_role(
            &self,
            _user_id: UserId,
            _role_id: RoleId,
            _store_id: StoreId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn get_roles_for_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<Vec<Role>, IdentityError> {
            Ok(vec![])
        }

        async fn get_permissions_for_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<Vec<Permission>, IdentityError> {
            Ok(vec![])
        }

        async fn remove_role_from_all_users(&self, role_id: RoleId) -> Result<(), IdentityError> {
            let mut removed = self.removed_roles.lock().unwrap();
            removed.push(role_id);
            Ok(())
        }

        async fn add_to_store(&self, _user_id: UserId, _store_id: StoreId) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn remove_from_store(&self, _user_id: UserId, _store_id: StoreId) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn get_stores(&self, _user_id: UserId) -> Result<Vec<Store>, IdentityError> {
            Ok(vec![])
        }

        async fn is_member_of_store(&self, _user_id: UserId, _store_id: StoreId) -> Result<bool, IdentityError> {
            Ok(false)
        }
    }

    // Mock PermissionRepository for testing
    struct MockPermissionRepository {
        permissions: Mutex<HashMap<PermissionId, Permission>>,
    }

    impl MockPermissionRepository {
        fn new() -> Self {
            Self {
                permissions: Mutex::new(HashMap::new()),
            }
        }

        fn with_permission(self, permission: Permission) -> Self {
            self.permissions.lock().unwrap().insert(*permission.id(), permission);
            self
        }
    }

    #[async_trait]
    impl PermissionRepository for MockPermissionRepository {
        async fn save(&self, permission: &Permission) -> Result<(), IdentityError> {
            let mut perms = self.permissions.lock().unwrap();
            perms.insert(*permission.id(), permission.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: PermissionId) -> Result<Option<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.get(&id).cloned())
        }

        async fn find_by_code(&self, code: &PermissionCode) -> Result<Option<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.values().find(|p| p.code() == code).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.values().cloned().collect())
        }

        async fn find_by_module(&self, module: &str) -> Result<Vec<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.values().filter(|p| p.module() == module).cloned().collect())
        }

        async fn delete(&self, id: PermissionId) -> Result<(), IdentityError> {
            let mut perms = self.permissions.lock().unwrap();
            perms.remove(&id);
            Ok(())
        }

        async fn exists(&self, code: &PermissionCode) -> Result<bool, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.values().any(|p| p.code() == code))
        }
    }

    // =============================================================================
    // CreateRoleUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_role_success() {
        let role_repo = Arc::new(MockRoleRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateRoleUseCase::new(role_repo.clone(), audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case
            .execute("Admin", Some("Administrator role".to_string()), actor_id)
            .await;

        assert!(result.is_ok());
        let role = result.unwrap();
        assert_eq!(role.name(), "Admin");
        assert_eq!(role.description(), Some("Administrator role"));
        assert!(!role.is_system_protected());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "role");
        assert_eq!(entries[0].action(), &AuditAction::Created);
    }

    #[tokio::test]
    async fn test_create_role_without_description() {
        let role_repo = Arc::new(MockRoleRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateRoleUseCase::new(role_repo, audit_repo);

        let actor_id = UserId::new();
        let result = use_case.execute("Viewer", None, actor_id).await;

        assert!(result.is_ok());
        let role = result.unwrap();
        assert_eq!(role.name(), "Viewer");
        assert_eq!(role.description(), None);
    }

    #[tokio::test]
    async fn test_create_role_duplicate_name() {
        let existing_role = Role::create("Admin".to_string(), None);
        let role_repo = Arc::new(MockRoleRepository::new().with_role(existing_role));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateRoleUseCase::new(role_repo, audit_repo);

        let actor_id = UserId::new();
        let result = use_case.execute("Admin", None, actor_id).await;

        assert!(matches!(result, Err(IdentityError::DuplicateRole(_))));
    }

    // =============================================================================
    // DeleteRoleUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_role_success() {
        let role = Role::create("ToDelete".to_string(), None);
        let role_id = *role.id();
        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = DeleteRoleUseCase::new(role_repo.clone(), user_repo.clone(), audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(role_id, actor_id).await;

        assert!(result.is_ok());

        // Verify role was deleted
        let found = role_repo.find_by_id(role_id).await.unwrap();
        assert!(found.is_none());

        // Verify role was removed from all users
        let removed_roles = user_repo.get_removed_roles();
        assert!(removed_roles.contains(&role_id));

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "role");
        assert_eq!(entries[0].action(), &AuditAction::Deleted);
    }

    #[tokio::test]
    async fn test_delete_role_not_found() {
        let role_repo = Arc::new(MockRoleRepository::new());
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = DeleteRoleUseCase::new(role_repo, user_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_id = RoleId::new();
        let result = use_case.execute(non_existent_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::RoleNotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_role_protected() {
        let protected_role = Role::create_protected("SuperAdmin".to_string(), None);
        let role_id = *protected_role.id();
        let role_repo = Arc::new(MockRoleRepository::new().with_role(protected_role));
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = DeleteRoleUseCase::new(role_repo, user_repo, audit_repo);

        let actor_id = UserId::new();
        let result = use_case.execute(role_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::ProtectedRoleCannotBeDeleted)));
    }

    // =============================================================================
    // AddPermissionToRoleUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_add_permission_to_role_success() {
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let permission_code = PermissionCode::new("sales:create").unwrap();
        let permission = Permission::create(permission_code, None);
        let permission_id = *permission.id();

        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let permission_repo = Arc::new(MockPermissionRepository::new().with_permission(permission));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AddPermissionToRoleUseCase::new(role_repo, permission_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(role_id, permission_id, actor_id).await;

        assert!(result.is_ok());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "role");
        assert_eq!(entries[0].action(), &AuditAction::PermissionAdded);
    }

    #[tokio::test]
    async fn test_add_permission_to_role_role_not_found() {
        let permission_code = PermissionCode::new("sales:create").unwrap();
        let permission = Permission::create(permission_code, None);
        let permission_id = *permission.id();

        let role_repo = Arc::new(MockRoleRepository::new());
        let permission_repo = Arc::new(MockPermissionRepository::new().with_permission(permission));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AddPermissionToRoleUseCase::new(role_repo, permission_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_role_id = RoleId::new();
        let result = use_case.execute(non_existent_role_id, permission_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::RoleNotFound(_))));
    }

    #[tokio::test]
    async fn test_add_permission_to_role_permission_not_found() {
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();

        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let permission_repo = Arc::new(MockPermissionRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AddPermissionToRoleUseCase::new(role_repo, permission_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_permission_id = PermissionId::new();
        let result = use_case.execute(role_id, non_existent_permission_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::PermissionNotFound(_))));
    }

    // =============================================================================
    // RemovePermissionFromRoleUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_remove_permission_from_role_success() {
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let permission_code = PermissionCode::new("sales:create").unwrap();
        let permission = Permission::create(permission_code, None);
        let permission_id = *permission.id();

        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let permission_repo = Arc::new(MockPermissionRepository::new().with_permission(permission));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RemovePermissionFromRoleUseCase::new(role_repo, permission_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(role_id, permission_id, actor_id).await;

        assert!(result.is_ok());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "role");
        assert_eq!(entries[0].action(), &AuditAction::PermissionRemoved);
    }

    #[tokio::test]
    async fn test_remove_permission_from_role_role_not_found() {
        let role_repo = Arc::new(MockRoleRepository::new());
        let permission_repo = Arc::new(MockPermissionRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RemovePermissionFromRoleUseCase::new(role_repo, permission_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_role_id = RoleId::new();
        let permission_id = PermissionId::new();
        let result = use_case.execute(non_existent_role_id, permission_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::RoleNotFound(_))));
    }

    #[tokio::test]
    async fn test_remove_permission_from_role_permission_not_in_repo() {
        // Even if permission doesn't exist in repo, we should still be able to remove it
        // (it might have been deleted already)
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();

        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let permission_repo = Arc::new(MockPermissionRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RemovePermissionFromRoleUseCase::new(role_repo, permission_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let permission_id = PermissionId::new();
        let result = use_case.execute(role_id, permission_id, actor_id).await;

        assert!(result.is_ok());

        // Verify audit entry was created (with null permission_code)
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
    }
}
