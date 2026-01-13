// Permission use cases - Application layer business logic for permission management
//
// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5

use std::sync::Arc;

use crate::domain::entities::{AuditEntry, Permission};
use crate::domain::repositories::{AuditRepository, PermissionRepository, RoleRepository};
use crate::domain::value_objects::{PermissionCode, PermissionId, UserId};
use crate::error::IdentityError;

// =============================================================================
// CreatePermissionUseCase
// =============================================================================

/// Use case for creating a new permission
///
/// Validates the permission format, checks for uniqueness, saves to repository,
/// and creates an audit entry.
///
/// Requirements: 1.1, 1.2, 1.3
pub struct CreatePermissionUseCase<P, A>
where
    P: PermissionRepository,
    A: AuditRepository,
{
    permission_repo: Arc<P>,
    audit_repo: Arc<A>,
}

impl<P, A> CreatePermissionUseCase<P, A>
where
    P: PermissionRepository,
    A: AuditRepository,
{
    /// Creates a new instance of CreatePermissionUseCase
    pub fn new(permission_repo: Arc<P>, audit_repo: Arc<A>) -> Self {
        Self {
            permission_repo,
            audit_repo,
        }
    }

    /// Executes the use case to create a new permission
    ///
    /// # Arguments
    /// * `code` - Permission code in format `module:action`
    /// * `description` - Optional description of the permission
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The created Permission on success
    ///
    /// # Errors
    /// * `IdentityError::InvalidPermissionFormat` - If code format is invalid
    /// * `IdentityError::DuplicatePermission` - If permission already exists
    pub async fn execute(
        &self,
        code: &str,
        description: Option<String>,
        actor_id: UserId,
    ) -> Result<Permission, IdentityError> {
        // Validate permission format (Requirements 1.1, 1.2)
        let permission_code = PermissionCode::new(code)?;

        // Check for uniqueness (Requirement 1.3)
        if self.permission_repo.exists(&permission_code).await? {
            return Err(IdentityError::DuplicatePermission(code.to_string()));
        }

        // Create and save the permission
        let permission = Permission::create(permission_code, description);
        self.permission_repo.save(&permission).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_create(
            "permission",
            permission.id().into_uuid(),
            &permission,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(permission)
    }
}


// =============================================================================
// DeletePermissionUseCase
// =============================================================================

/// Use case for deleting a permission
///
/// Removes the permission from all roles before deleting it, and creates
/// an audit entry.
///
/// Requirements: 1.5
pub struct DeletePermissionUseCase<P, R, A>
where
    P: PermissionRepository,
    R: RoleRepository,
    A: AuditRepository,
{
    permission_repo: Arc<P>,
    role_repo: Arc<R>,
    audit_repo: Arc<A>,
}

impl<P, R, A> DeletePermissionUseCase<P, R, A>
where
    P: PermissionRepository,
    R: RoleRepository,
    A: AuditRepository,
{
    /// Creates a new instance of DeletePermissionUseCase
    pub fn new(permission_repo: Arc<P>, role_repo: Arc<R>, audit_repo: Arc<A>) -> Self {
        Self {
            permission_repo,
            role_repo,
            audit_repo,
        }
    }

    /// Executes the use case to delete a permission
    ///
    /// # Arguments
    /// * `permission_id` - ID of the permission to delete
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::PermissionNotFound` - If permission doesn't exist
    pub async fn execute(
        &self,
        permission_id: PermissionId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Find the permission first (to get data for audit)
        let permission = self
            .permission_repo
            .find_by_id(permission_id)
            .await?
            .ok_or(IdentityError::PermissionNotFound(permission_id.into_uuid()))?;

        // Remove permission from all roles first (Requirement 1.5)
        self.role_repo
            .remove_permission_from_all_roles(permission_id)
            .await?;

        // Delete the permission
        self.permission_repo.delete(permission_id).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_delete(
            "permission",
            permission_id.into_uuid(),
            &permission,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}


// =============================================================================
// ListPermissionsUseCase
// =============================================================================

/// Use case for listing permissions with optional module filtering
///
/// Returns all permissions or filters by module prefix.
///
/// Requirements: 1.4
pub struct ListPermissionsUseCase<P>
where
    P: PermissionRepository,
{
    permission_repo: Arc<P>,
}

impl<P> ListPermissionsUseCase<P>
where
    P: PermissionRepository,
{
    /// Creates a new instance of ListPermissionsUseCase
    pub fn new(permission_repo: Arc<P>) -> Self {
        Self { permission_repo }
    }

    /// Executes the use case to list permissions
    ///
    /// # Arguments
    /// * `module_filter` - Optional module prefix to filter by (e.g., "sales")
    ///
    /// # Returns
    /// List of permissions, optionally filtered by module
    pub async fn execute(
        &self,
        module_filter: Option<&str>,
    ) -> Result<Vec<Permission>, IdentityError> {
        match module_filter {
            Some(module) => self.permission_repo.find_by_module(module).await,
            None => self.permission_repo.find_all().await,
        }
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

    use crate::domain::entities::Role;
    use crate::domain::value_objects::RoleId;

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

        async fn find_by_code(
            &self,
            code: &PermissionCode,
        ) -> Result<Option<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.values().find(|p| p.code() == code).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms.values().cloned().collect())
        }

        async fn find_by_module(&self, module: &str) -> Result<Vec<Permission>, IdentityError> {
            let perms = self.permissions.lock().unwrap();
            Ok(perms
                .values()
                .filter(|p| p.module() == module)
                .cloned()
                .collect())
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

    // Mock RoleRepository for testing
    struct MockRoleRepository;

    #[async_trait]
    impl RoleRepository for MockRoleRepository {
        async fn save(&self, _role: &Role) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn find_by_id(&self, _id: RoleId) -> Result<Option<Role>, IdentityError> {
            Ok(None)
        }

        async fn find_by_name(&self, _name: &str) -> Result<Option<Role>, IdentityError> {
            Ok(None)
        }

        async fn find_all(&self) -> Result<Vec<Role>, IdentityError> {
            Ok(vec![])
        }

        async fn delete(&self, _id: RoleId) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn update(&self, _role: &Role) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn add_permission(
            &self,
            _role_id: RoleId,
            _permission_id: PermissionId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn remove_permission(
            &self,
            _role_id: RoleId,
            _permission_id: PermissionId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn get_permissions(&self, _role_id: RoleId) -> Result<Vec<Permission>, IdentityError> {
            Ok(vec![])
        }

        async fn remove_permission_from_all_roles(
            &self,
            _permission_id: PermissionId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }
    }

    // =============================================================================
    // CreatePermissionUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_permission_success() {
        let perm_repo = Arc::new(MockPermissionRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreatePermissionUseCase::new(perm_repo.clone(), audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case
            .execute("sales:create_invoice", Some("Create invoices".to_string()), actor_id)
            .await;

        assert!(result.is_ok());
        let permission = result.unwrap();
        assert_eq!(permission.code().as_str(), "sales:create_invoice");
        assert_eq!(permission.description(), Some("Create invoices"));

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "permission");
    }

    #[tokio::test]
    async fn test_create_permission_invalid_format() {
        let perm_repo = Arc::new(MockPermissionRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreatePermissionUseCase::new(perm_repo, audit_repo);

        let actor_id = UserId::new();
        let result = use_case.execute("invalid_format", None, actor_id).await;

        assert!(matches!(result, Err(IdentityError::InvalidPermissionFormat)));
    }

    #[tokio::test]
    async fn test_create_permission_duplicate() {
        let perm_repo = Arc::new(MockPermissionRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreatePermissionUseCase::new(perm_repo.clone(), audit_repo.clone());

        let actor_id = UserId::new();

        // Create first permission
        let _ = use_case
            .execute("sales:create_invoice", None, actor_id)
            .await
            .unwrap();

        // Try to create duplicate
        let result = use_case
            .execute("sales:create_invoice", None, actor_id)
            .await;

        assert!(matches!(result, Err(IdentityError::DuplicatePermission(_))));
    }

    // =============================================================================
    // DeletePermissionUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_delete_permission_success() {
        let perm_repo = Arc::new(MockPermissionRepository::new());
        let role_repo = Arc::new(MockRoleRepository);
        let audit_repo = Arc::new(MockAuditRepository::new());

        // First create a permission
        let code = PermissionCode::new("sales:delete_invoice").unwrap();
        let permission = Permission::create(code, None);
        let permission_id = *permission.id();
        perm_repo.save(&permission).await.unwrap();

        let use_case =
            DeletePermissionUseCase::new(perm_repo.clone(), role_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(permission_id, actor_id).await;

        assert!(result.is_ok());

        // Verify permission was deleted
        let found = perm_repo.find_by_id(permission_id).await.unwrap();
        assert!(found.is_none());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "permission");
    }

    #[tokio::test]
    async fn test_delete_permission_not_found() {
        let perm_repo = Arc::new(MockPermissionRepository::new());
        let role_repo = Arc::new(MockRoleRepository);
        let audit_repo = Arc::new(MockAuditRepository::new());

        let use_case = DeletePermissionUseCase::new(perm_repo, role_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_id = PermissionId::new();
        let result = use_case.execute(non_existent_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::PermissionNotFound(_))));
    }

    // =============================================================================
    // ListPermissionsUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_list_permissions_all() {
        let perm_repo = Arc::new(MockPermissionRepository::new());

        // Create some permissions
        let code1 = PermissionCode::new("sales:create").unwrap();
        let code2 = PermissionCode::new("inventory:view").unwrap();
        perm_repo.save(&Permission::create(code1, None)).await.unwrap();
        perm_repo.save(&Permission::create(code2, None)).await.unwrap();

        let use_case = ListPermissionsUseCase::new(perm_repo);

        let result = use_case.execute(None).await;

        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 2);
    }

    #[tokio::test]
    async fn test_list_permissions_by_module() {
        let perm_repo = Arc::new(MockPermissionRepository::new());

        // Create permissions in different modules
        let code1 = PermissionCode::new("sales:create").unwrap();
        let code2 = PermissionCode::new("sales:delete").unwrap();
        let code3 = PermissionCode::new("inventory:view").unwrap();
        perm_repo.save(&Permission::create(code1, None)).await.unwrap();
        perm_repo.save(&Permission::create(code2, None)).await.unwrap();
        perm_repo.save(&Permission::create(code3, None)).await.unwrap();

        let use_case = ListPermissionsUseCase::new(perm_repo);

        let result = use_case.execute(Some("sales")).await;

        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 2);
        assert!(permissions.iter().all(|p| p.module() == "sales"));
    }

    #[tokio::test]
    async fn test_list_permissions_empty_module() {
        let perm_repo = Arc::new(MockPermissionRepository::new());

        // Create permissions in different modules
        let code1 = PermissionCode::new("sales:create").unwrap();
        perm_repo.save(&Permission::create(code1, None)).await.unwrap();

        let use_case = ListPermissionsUseCase::new(perm_repo);

        let result = use_case.execute(Some("nonexistent")).await;

        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert!(permissions.is_empty());
    }
}
