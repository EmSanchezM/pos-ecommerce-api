// BuildUserContextUseCase - builds an immutable UserContext for a user session
//
// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
// - Verify user is active
// - Get roles for the user in the specified store
// - Merge permissions from all roles (deduplicated)
// - Return UserContext with user_id, store_id, and permissions

use std::collections::HashSet;
use std::sync::Arc;

use crate::domain::repositories::UserRepository;
use crate::domain::services::UserContext;
use crate::domain::value_objects::{PermissionCode, StoreId, UserId};
use crate::error::IdentityError;

/// Use case for building a UserContext with effective permissions
///
/// This use case:
/// 1. Verifies the user exists and is active
/// 2. Retrieves all roles assigned to the user in the specified store
/// 3. Merges permissions from all roles into a deduplicated set
/// 4. Returns an immutable UserContext for the request lifecycle
pub struct BuildUserContextUseCase<U>
where
    U: UserRepository,
{
    user_repo: Arc<U>,
}

impl<U> BuildUserContextUseCase<U>
where
    U: UserRepository,
{
    /// Creates a new BuildUserContextUseCase with the required repository
    pub fn new(user_repo: Arc<U>) -> Self {
        Self { user_repo }
    }

    /// Builds a UserContext for the given user and store
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user
    /// * `store_id` - The ID of the store/tenant context
    ///
    /// # Returns
    ///
    /// A `UserContext` containing the user_id, store_id, and deduplicated
    /// permissions from all roles assigned to the user in that store.
    ///
    /// # Errors
    ///
    /// * `IdentityError::UserNotFound` - If the user doesn't exist
    /// * `IdentityError::UserInactive` - If the user account is inactive
    /// * `IdentityError::Database` - On database errors
    ///
    /// # Example
    ///
    /// ```ignore
    /// let use_case = BuildUserContextUseCase::new(user_repo);
    /// let ctx = use_case.execute(user_id, store_id).await?;
    ///
    /// if ctx.has_permission("sales:create_invoice") {
    ///     // User can create invoices in this store
    /// }
    /// ```
    pub async fn execute(
        &self,
        user_id: UserId,
        store_id: StoreId,
    ) -> Result<UserContext, IdentityError> {
        // 1. Verify user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| IdentityError::UserNotFound(*user_id.as_uuid()))?;

        // 2. Verify user is active (Requirement 6.4)
        if !user.is_active() {
            return Err(IdentityError::UserInactive);
        }

        // 3. Get all permissions for the user in this store
        // This already merges permissions from all roles (handled by repository)
        let permissions = self
            .user_repo
            .get_permissions_for_store(user_id, store_id)
            .await?;

        // 4. Convert to HashSet of PermissionCode for deduplication
        // (Requirement 4.2, 4.3 - deduplicated list, merge from all roles)
        let permission_set: HashSet<PermissionCode> = permissions
            .into_iter()
            .map(|p| p.code().clone())
            .collect();

        // 5. Build and return the UserContext
        // If user has no roles in the store, permission_set will be empty (Requirement 4.4)
        Ok(UserContext::new(user_id, store_id, permission_set))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Permission, Role, Store, User};
    use crate::domain::value_objects::{Email, RoleId, Username};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // Mock UserRepository for testing
    struct MockUserRepository {
        users: Mutex<Vec<User>>,
        permissions: Mutex<Vec<Permission>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
                permissions: Mutex::new(Vec::new()),
            }
        }

        fn with_user(user: User) -> Self {
            let repo = Self::new();
            repo.users.lock().unwrap().push(user);
            repo
        }

        fn with_user_and_permissions(user: User, permissions: Vec<Permission>) -> Self {
            let repo = Self::new();
            repo.users.lock().unwrap().push(user);
            *repo.permissions.lock().unwrap() = permissions;
            repo
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, _user: &User) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| *u.id() == id).cloned())
        }

        async fn find_by_email(&self, _email: &Email) -> Result<Option<User>, IdentityError> {
            Ok(None)
        }

        async fn find_by_username(
            &self,
            _username: &Username,
        ) -> Result<Option<User>, IdentityError> {
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
            Ok(Vec::new())
        }

        async fn get_permissions_for_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<Vec<Permission>, IdentityError> {
            let permissions = self.permissions.lock().unwrap();
            Ok(permissions.clone())
        }

        async fn remove_role_from_all_users(&self, _role_id: RoleId) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn add_to_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn remove_from_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn get_stores(&self, _user_id: UserId) -> Result<Vec<Store>, IdentityError> {
            Ok(Vec::new())
        }

        async fn is_member_of_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<bool, IdentityError> {
            Ok(true)
        }
    }

    fn create_active_user() -> User {
        User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hashed_password".to_string(),
        )
    }

    fn create_inactive_user() -> User {
        let mut user = create_active_user();
        user.deactivate();
        user
    }

    fn create_permission(code: &str) -> Permission {
        Permission::create(PermissionCode::new(code).unwrap(), None)
    }

    #[tokio::test]
    async fn test_build_user_context_success() {
        let user = create_active_user();
        let user_id = *user.id();
        let store_id = StoreId::new();

        let permissions = vec![
            create_permission("sales:create"),
            create_permission("sales:view"),
        ];

        let repo = Arc::new(MockUserRepository::with_user_and_permissions(
            user,
            permissions,
        ));
        let use_case = BuildUserContextUseCase::new(repo);

        let result = use_case.execute(user_id, store_id).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(*ctx.user_id(), user_id);
        assert_eq!(*ctx.store_id(), store_id);
        assert_eq!(ctx.permissions().len(), 2);
        assert!(ctx.has_permission("sales:create"));
        assert!(ctx.has_permission("sales:view"));
    }

    #[tokio::test]
    async fn test_build_user_context_user_not_found() {
        let repo = Arc::new(MockUserRepository::new());
        let use_case = BuildUserContextUseCase::new(repo);

        let result = use_case.execute(UserId::new(), StoreId::new()).await;

        assert!(matches!(result, Err(IdentityError::UserNotFound(_))));
    }

    #[tokio::test]
    async fn test_build_user_context_user_inactive() {
        let user = create_inactive_user();
        let user_id = *user.id();
        let store_id = StoreId::new();

        let repo = Arc::new(MockUserRepository::with_user(user));
        let use_case = BuildUserContextUseCase::new(repo);

        let result = use_case.execute(user_id, store_id).await;

        assert!(matches!(result, Err(IdentityError::UserInactive)));
    }

    #[tokio::test]
    async fn test_build_user_context_no_permissions() {
        let user = create_active_user();
        let user_id = *user.id();
        let store_id = StoreId::new();

        let repo = Arc::new(MockUserRepository::with_user(user));
        let use_case = BuildUserContextUseCase::new(repo);

        let result = use_case.execute(user_id, store_id).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert!(ctx.permissions().is_empty());
    }

    #[tokio::test]
    async fn test_build_user_context_deduplicates_permissions() {
        let user = create_active_user();
        let user_id = *user.id();
        let store_id = StoreId::new();

        // Simulate duplicate permissions from multiple roles
        let permissions = vec![
            create_permission("sales:create"),
            create_permission("sales:create"), // duplicate
            create_permission("sales:view"),
        ];

        let repo = Arc::new(MockUserRepository::with_user_and_permissions(
            user,
            permissions,
        ));
        let use_case = BuildUserContextUseCase::new(repo);

        let result = use_case.execute(user_id, store_id).await;

        assert!(result.is_ok());
        let ctx = result.unwrap();
        // Should be deduplicated to 2 unique permissions
        assert_eq!(ctx.permissions().len(), 2);
    }
}
