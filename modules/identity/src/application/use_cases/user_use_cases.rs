// User use cases - Application layer business logic for user management
//
// Requirements: 3.1, 3.5, 6.1, 6.2, 6.3, 6.5, 6.6

use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use crate::application::dtos::{CreateUserCommand, UpdateUserCommand};
use crate::domain::entities::{AuditAction, AuditEntry, User};
use crate::domain::repositories::{AuditRepository, RoleRepository, StoreRepository, UserRepository};
use crate::domain::value_objects::{Email, RoleId, StoreId, UserId, Username};
use crate::error::IdentityError;

// =============================================================================
// CreateUserUseCase
// =============================================================================

/// Use case for creating a new user
///
/// Validates username/email uniqueness, hashes the password using Argon2,
/// generates a unique UUID, and creates an audit entry.
///
/// Requirements: 6.1, 6.2
pub struct CreateUserUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    audit_repo: Arc<A>,
}

impl<U, A> CreateUserUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    /// Creates a new instance of CreateUserUseCase
    pub fn new(user_repo: Arc<U>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            audit_repo,
        }
    }

    /// Executes the use case to create a new user
    ///
    /// # Arguments
    /// * `command` - The create user command containing user data
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The created User on success
    ///
    /// # Errors
    /// * `IdentityError::DuplicateUsername` - If username already exists
    /// * `IdentityError::DuplicateEmail` - If email already exists
    /// * `IdentityError::InvalidUsernameFormat` - If username format is invalid
    /// * `IdentityError::InvalidEmailFormat` - If email format is invalid
    pub async fn execute(
        &self,
        command: CreateUserCommand,
        actor_id: UserId,
    ) -> Result<User, IdentityError> {
        // Validate and create value objects
        let username = Username::new(&command.username)?;
        let email = Email::new(&command.email)?;

        // Check username uniqueness (Requirement 6.1)
        if self.user_repo.find_by_username(&username).await?.is_some() {
            return Err(IdentityError::DuplicateUsername(command.username));
        }

        // Check email uniqueness (Requirement 6.1)
        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(IdentityError::DuplicateEmail(command.email));
        }

        // Hash the password using Argon2 (Requirement 6.2)
        let password_hash = hash_password(&command.password)?;

        // Create user with generated UUID (Requirement 6.2)
        let user = User::create(
            username,
            email,
            command.first_name,
            command.last_name,
            password_hash,
        );

        // Save to repository
        self.user_repo.save(&user).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_create(
            "user",
            user.id().into_uuid(),
            &user,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(user)
    }
}

/// Hashes a password using Argon2
fn hash_password(password: &str) -> Result<String, IdentityError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| IdentityError::NotImplemented) // Using NotImplemented as a placeholder for password hash error
}


// =============================================================================
// UpdateUserUseCase
// =============================================================================

/// Use case for updating an existing user's profile
///
/// Validates email uniqueness if changed, updates the user,
/// and creates an audit entry.
///
/// Requirements: 6.5, 6.6
pub struct UpdateUserUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    audit_repo: Arc<A>,
}

impl<U, A> UpdateUserUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    /// Creates a new instance of UpdateUserUseCase
    pub fn new(user_repo: Arc<U>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            audit_repo,
        }
    }

    /// Executes the use case to update a user's profile
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to update
    /// * `command` - The update user command containing fields to update
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The updated User on success
    ///
    /// # Errors
    /// * `IdentityError::UserNotFound` - If user doesn't exist
    /// * `IdentityError::DuplicateEmail` - If new email conflicts with another user
    /// * `IdentityError::InvalidEmailFormat` - If new email format is invalid
    pub async fn execute(
        &self,
        user_id: UserId,
        command: UpdateUserCommand,
        actor_id: UserId,
    ) -> Result<User, IdentityError> {
        // Find the user
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IdentityError::UserNotFound(user_id.into_uuid()))?;

        // Store old state for audit
        let old_user = user.clone();

        // Update first name if provided (Requirement 6.5)
        if let Some(first_name) = command.first_name {
            user.set_first_name(first_name);
        }

        // Update last name if provided (Requirement 6.5)
        if let Some(last_name) = command.last_name {
            user.set_last_name(last_name);
        }

        // Update email if provided (Requirement 6.5, 6.6)
        if let Some(email_str) = command.email {
            let new_email = Email::new(&email_str)?;
            
            // Check uniqueness only if email is actually changing
            if new_email.as_str() != user.email().as_str() {
                // Check if new email is already in use by another user (Requirement 6.6)
                if let Some(existing_user) = self.user_repo.find_by_email(&new_email).await? {
                    if existing_user.id() != &user_id {
                        return Err(IdentityError::DuplicateEmail(email_str));
                    }
                }
                user.set_email(new_email);
            }
        }

        // Save updated user
        self.user_repo.update(&user).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_update(
            "user",
            user_id.into_uuid(),
            &old_user,
            &user,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(user)
    }
}


// =============================================================================
// SetUserActiveUseCase
// =============================================================================

/// Use case for enabling or disabling a user account
///
/// Updates the user's is_active flag and creates an audit entry.
///
/// Requirements: 6.3
pub struct SetUserActiveUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    audit_repo: Arc<A>,
}

impl<U, A> SetUserActiveUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    /// Creates a new instance of SetUserActiveUseCase
    pub fn new(user_repo: Arc<U>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            audit_repo,
        }
    }

    /// Executes the use case to set a user's active status
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to update
    /// * `is_active` - Whether the user should be active
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// The updated User on success
    ///
    /// # Errors
    /// * `IdentityError::UserNotFound` - If user doesn't exist
    pub async fn execute(
        &self,
        user_id: UserId,
        is_active: bool,
        actor_id: UserId,
    ) -> Result<User, IdentityError> {
        // Find the user
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IdentityError::UserNotFound(user_id.into_uuid()))?;

        // Store old state for audit
        let old_user = user.clone();

        // Update active status (Requirement 6.3)
        if is_active {
            user.activate();
        } else {
            user.deactivate();
        }

        // Save updated user
        self.user_repo.update(&user).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_update(
            "user",
            user_id.into_uuid(),
            &old_user,
            &user,
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(user)
    }
}


// =============================================================================
// AssignRoleUseCase
// =============================================================================

/// Use case for assigning a role to a user in a specific store
///
/// Verifies the user exists, belongs to the store, and the role exists,
/// then assigns the role and creates an audit entry.
///
/// Requirements: 3.1, 3.5
pub struct AssignRoleUseCase<U, R, S, A>
where
    U: UserRepository,
    R: RoleRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    role_repo: Arc<R>,
    store_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<U, R, S, A> AssignRoleUseCase<U, R, S, A>
where
    U: UserRepository,
    R: RoleRepository,
    S: StoreRepository,
    A: AuditRepository,
{
    /// Creates a new instance of AssignRoleUseCase
    pub fn new(
        user_repo: Arc<U>,
        role_repo: Arc<R>,
        store_repo: Arc<S>,
        audit_repo: Arc<A>,
    ) -> Self {
        Self {
            user_repo,
            role_repo,
            store_repo,
            audit_repo,
        }
    }

    /// Executes the use case to assign a role to a user in a store
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to assign the role to
    /// * `role_id` - ID of the role to assign
    /// * `store_id` - ID of the store context for this assignment
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::UserNotFound` - If user doesn't exist
    /// * `IdentityError::RoleNotFound` - If role doesn't exist (Requirement 3.5)
    /// * `IdentityError::StoreNotFound` - If store doesn't exist
    /// * `IdentityError::UserNotInStore` - If user is not a member of the store
    pub async fn execute(
        &self,
        user_id: UserId,
        role_id: RoleId,
        store_id: StoreId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IdentityError::UserNotFound(user_id.into_uuid()))?;

        // Verify role exists (Requirement 3.5)
        let role = self
            .role_repo
            .find_by_id(role_id)
            .await?
            .ok_or(IdentityError::RoleNotFound(role_id.into_uuid()))?;

        // Verify store exists
        self.store_repo
            .find_by_id(store_id)
            .await?
            .ok_or(IdentityError::StoreNotFound(store_id.into_uuid()))?;

        // Verify user is a member of the store
        if !self.user_repo.is_member_of_store(user_id, store_id).await? {
            return Err(IdentityError::UserNotInStore(store_id.into_uuid()));
        }

        // Assign role to user in store (Requirement 3.1)
        self.user_repo.assign_role(user_id, role_id, store_id).await?;

        // Create audit entry
        let audit_entry = AuditEntry::create(
            "user_store_role".to_string(),
            user_id.into_uuid(),
            AuditAction::RoleAssigned,
            None,
            Some(serde_json::json!({
                "user_id": user_id.into_uuid(),
                "username": user.username().as_str(),
                "role_id": role_id.into_uuid(),
                "role_name": role.name(),
                "store_id": store_id.into_uuid()
            })),
            actor_id,
        );
        self.audit_repo.save(&audit_entry).await?;

        Ok(())
    }
}


// =============================================================================
// RemoveRoleUseCase
// =============================================================================

/// Use case for removing a role from a user in a specific store
///
/// Verifies the user exists, then removes the role assignment
/// and creates an audit entry.
///
/// Requirements: 3.1
pub struct RemoveRoleUseCase<U, R, A>
where
    U: UserRepository,
    R: RoleRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    role_repo: Arc<R>,
    audit_repo: Arc<A>,
}

impl<U, R, A> RemoveRoleUseCase<U, R, A>
where
    U: UserRepository,
    R: RoleRepository,
    A: AuditRepository,
{
    /// Creates a new instance of RemoveRoleUseCase
    pub fn new(user_repo: Arc<U>, role_repo: Arc<R>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            role_repo,
            audit_repo,
        }
    }

    /// Executes the use case to remove a role from a user in a store
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to remove the role from
    /// * `role_id` - ID of the role to remove
    /// * `store_id` - ID of the store context for this removal
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Errors
    /// * `IdentityError::UserNotFound` - If user doesn't exist
    pub async fn execute(
        &self,
        user_id: UserId,
        role_id: RoleId,
        store_id: StoreId,
        actor_id: UserId,
    ) -> Result<(), IdentityError> {
        // Verify user exists
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(IdentityError::UserNotFound(user_id.into_uuid()))?;

        // Get role info for audit (if it exists)
        let role_info = self
            .role_repo
            .find_by_id(role_id)
            .await?
            .map(|r| r.name().to_string());

        // Remove role from user in store (Requirement 3.1)
        self.user_repo.remove_role(user_id, role_id, store_id).await?;

        // Create audit entry
        let audit_entry = AuditEntry::create(
            "user_store_role".to_string(),
            user_id.into_uuid(),
            AuditAction::RoleUnassigned,
            Some(serde_json::json!({
                "user_id": user_id.into_uuid(),
                "username": user.username().as_str(),
                "role_id": role_id.into_uuid(),
                "role_name": role_info,
                "store_id": store_id.into_uuid()
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

    use crate::domain::entities::{Permission, Role, Store};

    // Mock UserRepository for testing
    struct MockUserRepository {
        users: Mutex<HashMap<UserId, User>>,
        user_stores: Mutex<HashMap<(UserId, StoreId), bool>>,
        user_roles: Mutex<HashMap<(UserId, StoreId), Vec<RoleId>>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
                user_stores: Mutex::new(HashMap::new()),
                user_roles: Mutex::new(HashMap::new()),
            }
        }

        fn with_user(self, user: User) -> Self {
            self.users.lock().unwrap().insert(*user.id(), user);
            self
        }

        fn with_user_in_store(self, user_id: UserId, store_id: StoreId) -> Self {
            self.user_stores.lock().unwrap().insert((user_id, store_id), true);
            self
        }
    }


    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, user: &User) -> Result<(), IdentityError> {
            let mut users = self.users.lock().unwrap();
            users.insert(*user.id(), user.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(&id).cloned())
        }

        async fn find_by_email(&self, email: &Email) -> Result<Option<User>, IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.email() == email).cloned())
        }

        async fn find_by_username(&self, username: &Username) -> Result<Option<User>, IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.username() == username).cloned())
        }

        async fn update(&self, user: &User) -> Result<(), IdentityError> {
            let mut users = self.users.lock().unwrap();
            users.insert(*user.id(), user.clone());
            Ok(())
        }

        async fn assign_role(
            &self,
            user_id: UserId,
            role_id: RoleId,
            store_id: StoreId,
        ) -> Result<(), IdentityError> {
            let mut roles = self.user_roles.lock().unwrap();
            roles.entry((user_id, store_id)).or_default().push(role_id);
            Ok(())
        }

        async fn remove_role(
            &self,
            user_id: UserId,
            role_id: RoleId,
            store_id: StoreId,
        ) -> Result<(), IdentityError> {
            let mut roles = self.user_roles.lock().unwrap();
            if let Some(role_list) = roles.get_mut(&(user_id, store_id)) {
                role_list.retain(|r| *r != role_id);
            }
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

        async fn remove_role_from_all_users(&self, _role_id: RoleId) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn add_to_store(&self, user_id: UserId, store_id: StoreId) -> Result<(), IdentityError> {
            let mut stores = self.user_stores.lock().unwrap();
            stores.insert((user_id, store_id), true);
            Ok(())
        }

        async fn remove_from_store(&self, user_id: UserId, store_id: StoreId) -> Result<(), IdentityError> {
            let mut stores = self.user_stores.lock().unwrap();
            stores.remove(&(user_id, store_id));
            Ok(())
        }

        async fn get_stores(&self, _user_id: UserId) -> Result<Vec<Store>, IdentityError> {
            Ok(vec![])
        }

        async fn is_member_of_store(
            &self,
            user_id: UserId,
            store_id: StoreId,
        ) -> Result<bool, IdentityError> {
            let stores = self.user_stores.lock().unwrap();
            Ok(stores.get(&(user_id, store_id)).copied().unwrap_or(false))
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
    struct MockRoleRepository {
        roles: Mutex<HashMap<RoleId, Role>>,
    }

    impl MockRoleRepository {
        fn new() -> Self {
            Self {
                roles: Mutex::new(HashMap::new()),
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
            Ok(())
        }

        async fn update(&self, role: &Role) -> Result<(), IdentityError> {
            let mut roles = self.roles.lock().unwrap();
            roles.insert(*role.id(), role.clone());
            Ok(())
        }


        async fn add_permission(
            &self,
            _role_id: RoleId,
            _permission_id: crate::domain::value_objects::PermissionId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn remove_permission(
            &self,
            _role_id: RoleId,
            _permission_id: crate::domain::value_objects::PermissionId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }

        async fn get_permissions(&self, _role_id: RoleId) -> Result<Vec<Permission>, IdentityError> {
            Ok(vec![])
        }

        async fn remove_permission_from_all_roles(
            &self,
            _permission_id: crate::domain::value_objects::PermissionId,
        ) -> Result<(), IdentityError> {
            Ok(())
        }
    }

    // Mock StoreRepository for testing
    struct MockStoreRepository {
        stores: Mutex<HashMap<StoreId, Store>>,
    }

    impl MockStoreRepository {
        fn new() -> Self {
            Self {
                stores: Mutex::new(HashMap::new()),
            }
        }

        fn with_store(self, store: Store) -> Self {
            self.stores.lock().unwrap().insert(*store.id(), store);
            self
        }
    }


    #[async_trait]
    impl StoreRepository for MockStoreRepository {
        async fn save(&self, store: &Store) -> Result<(), IdentityError> {
            let mut stores = self.stores.lock().unwrap();
            stores.insert(*store.id(), store.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: StoreId) -> Result<Option<Store>, IdentityError> {
            let stores = self.stores.lock().unwrap();
            Ok(stores.get(&id).cloned())
        }

        async fn find_all(&self) -> Result<Vec<Store>, IdentityError> {
            let stores = self.stores.lock().unwrap();
            Ok(stores.values().cloned().collect())
        }

        async fn find_active(&self) -> Result<Vec<Store>, IdentityError> {
            let stores = self.stores.lock().unwrap();
            Ok(stores.values().filter(|s| s.is_active()).cloned().collect())
        }

        async fn update(&self, store: &Store) -> Result<(), IdentityError> {
            let mut stores = self.stores.lock().unwrap();
            stores.insert(*store.id(), store.clone());
            Ok(())
        }

        async fn get_users(&self, _store_id: StoreId) -> Result<Vec<User>, IdentityError> {
            Ok(vec![])
        }
    }


    // =============================================================================
    // CreateUserUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_create_user_success() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateUserUseCase::new(user_repo.clone(), audit_repo.clone());

        let actor_id = UserId::new();
        let command = CreateUserCommand {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "securepassword123".to_string(),
        };

        let result = use_case.execute(command, actor_id).await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username().as_str(), "testuser");
        assert_eq!(user.email().as_str(), "test@example.com");
        assert_eq!(user.first_name(), "John");
        assert_eq!(user.last_name(), "Doe");
        assert!(user.is_active());
        // Password should be hashed, not plain text
        assert_ne!(user.password_hash(), "securepassword123");

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "user");
        assert_eq!(entries[0].action(), &AuditAction::Created);
    }


    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        let existing_user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("existing@example.com").unwrap(),
            "Existing".to_string(),
            "User".to_string(),
            "hash".to_string(),
        );
        let user_repo = Arc::new(MockUserRepository::new().with_user(existing_user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateUserUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let command = CreateUserCommand {
            username: "testuser".to_string(),
            email: "new@example.com".to_string(),
            first_name: "New".to_string(),
            last_name: "User".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(IdentityError::DuplicateUsername(_))));
    }

    #[tokio::test]
    async fn test_create_user_duplicate_email() {
        let existing_user = User::create(
            Username::new("existinguser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "Existing".to_string(),
            "User".to_string(),
            "hash".to_string(),
        );
        let user_repo = Arc::new(MockUserRepository::new().with_user(existing_user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateUserUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let command = CreateUserCommand {
            username: "newuser".to_string(),
            email: "test@example.com".to_string(),
            first_name: "New".to_string(),
            last_name: "User".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(IdentityError::DuplicateEmail(_))));
    }


    #[tokio::test]
    async fn test_create_user_invalid_email() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateUserUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let command = CreateUserCommand {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(IdentityError::InvalidEmailFormat)));
    }

    // =============================================================================
    // UpdateUserUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_update_user_success() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = UpdateUserUseCase::new(user_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let command = UpdateUserCommand {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            email: None,
        };

        let result = use_case.execute(user_id, command, actor_id).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.first_name(), "Jane");
        assert_eq!(updated_user.last_name(), "Smith");
        assert_eq!(updated_user.email().as_str(), "test@example.com");

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action(), &AuditAction::Updated);
    }


    #[tokio::test]
    async fn test_update_user_email_success() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("old@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = UpdateUserUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let command = UpdateUserCommand {
            first_name: None,
            last_name: None,
            email: Some("new@example.com".to_string()),
        };

        let result = use_case.execute(user_id, command, actor_id).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.email().as_str(), "new@example.com");
    }

    #[tokio::test]
    async fn test_update_user_duplicate_email() {
        let user1 = User::create(
            Username::new("user1").unwrap(),
            Email::new("user1@example.com").unwrap(),
            "User".to_string(),
            "One".to_string(),
            "hash".to_string(),
        );
        let user2 = User::create(
            Username::new("user2").unwrap(),
            Email::new("user2@example.com").unwrap(),
            "User".to_string(),
            "Two".to_string(),
            "hash".to_string(),
        );
        let user1_id = *user1.id();
        let user_repo = Arc::new(MockUserRepository::new().with_user(user1).with_user(user2));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = UpdateUserUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let command = UpdateUserCommand {
            first_name: None,
            last_name: None,
            email: Some("user2@example.com".to_string()), // Try to use user2's email
        };

        let result = use_case.execute(user1_id, command, actor_id).await;
        assert!(matches!(result, Err(IdentityError::DuplicateEmail(_))));
    }


    #[tokio::test]
    async fn test_update_user_not_found() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = UpdateUserUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_id = UserId::new();
        let command = UpdateUserCommand {
            first_name: Some("Test".to_string()),
            last_name: None,
            email: None,
        };

        let result = use_case.execute(non_existent_id, command, actor_id).await;
        assert!(matches!(result, Err(IdentityError::UserNotFound(_))));
    }

    // =============================================================================
    // SetUserActiveUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_set_user_active_deactivate() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        assert!(user.is_active()); // Initially active

        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = SetUserActiveUseCase::new(user_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(user_id, false, actor_id).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert!(!updated_user.is_active());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action(), &AuditAction::Updated);
    }


    #[tokio::test]
    async fn test_set_user_active_activate() {
        let mut user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        user.deactivate();
        let user_id = *user.id();
        assert!(!user.is_active()); // Initially inactive

        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = SetUserActiveUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let result = use_case.execute(user_id, true, actor_id).await;

        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert!(updated_user.is_active());
    }

    #[tokio::test]
    async fn test_set_user_active_not_found() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = SetUserActiveUseCase::new(user_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_id = UserId::new();
        let result = use_case.execute(non_existent_id, true, actor_id).await;

        assert!(matches!(result, Err(IdentityError::UserNotFound(_))));
    }


    // =============================================================================
    // AssignRoleUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_assign_role_success() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let store = Store::create("Test Store".to_string(), "123 Main St".to_string());
        let store_id = *store.id();

        let user_repo = Arc::new(
            MockUserRepository::new()
                .with_user(user)
                .with_user_in_store(user_id, store_id)
        );
        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let store_repo = Arc::new(MockStoreRepository::new().with_store(store));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AssignRoleUseCase::new(user_repo, role_repo, store_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(user_id, role_id, store_id, actor_id).await;

        assert!(result.is_ok());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action(), &AuditAction::RoleAssigned);
    }


    #[tokio::test]
    async fn test_assign_role_user_not_found() {
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let store = Store::create("Test Store".to_string(), "123 Main St".to_string());
        let store_id = *store.id();

        let user_repo = Arc::new(MockUserRepository::new());
        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let store_repo = Arc::new(MockStoreRepository::new().with_store(store));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AssignRoleUseCase::new(user_repo, role_repo, store_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_user_id = UserId::new();
        let result = use_case.execute(non_existent_user_id, role_id, store_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::UserNotFound(_))));
    }

    #[tokio::test]
    async fn test_assign_role_role_not_found() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        let store = Store::create("Test Store".to_string(), "123 Main St".to_string());
        let store_id = *store.id();

        let user_repo = Arc::new(
            MockUserRepository::new()
                .with_user(user)
                .with_user_in_store(user_id, store_id)
        );
        let role_repo = Arc::new(MockRoleRepository::new());
        let store_repo = Arc::new(MockStoreRepository::new().with_store(store));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AssignRoleUseCase::new(user_repo, role_repo, store_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_role_id = RoleId::new();
        let result = use_case.execute(user_id, non_existent_role_id, store_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::RoleNotFound(_))));
    }


    #[tokio::test]
    async fn test_assign_role_user_not_in_store() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let store = Store::create("Test Store".to_string(), "123 Main St".to_string());
        let store_id = *store.id();

        // User exists but is NOT in the store
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let store_repo = Arc::new(MockStoreRepository::new().with_store(store));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = AssignRoleUseCase::new(user_repo, role_repo, store_repo, audit_repo);

        let actor_id = UserId::new();
        let result = use_case.execute(user_id, role_id, store_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::UserNotInStore(_))));
    }

    // =============================================================================
    // RemoveRoleUseCase Tests
    // =============================================================================

    #[tokio::test]
    async fn test_remove_role_success() {
        let user = User::create(
            Username::new("testuser").unwrap(),
            Email::new("test@example.com").unwrap(),
            "John".to_string(),
            "Doe".to_string(),
            "hash".to_string(),
        );
        let user_id = *user.id();
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let store_id = StoreId::new();

        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RemoveRoleUseCase::new(user_repo, role_repo, audit_repo.clone());

        let actor_id = UserId::new();
        let result = use_case.execute(user_id, role_id, store_id, actor_id).await;

        assert!(result.is_ok());

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action(), &AuditAction::RoleUnassigned);
    }


    #[tokio::test]
    async fn test_remove_role_user_not_found() {
        let role = Role::create("Admin".to_string(), None);
        let role_id = *role.id();
        let store_id = StoreId::new();

        let user_repo = Arc::new(MockUserRepository::new());
        let role_repo = Arc::new(MockRoleRepository::new().with_role(role));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RemoveRoleUseCase::new(user_repo, role_repo, audit_repo);

        let actor_id = UserId::new();
        let non_existent_user_id = UserId::new();
        let result = use_case.execute(non_existent_user_id, role_id, store_id, actor_id).await;

        assert!(matches!(result, Err(IdentityError::UserNotFound(_))));
    }
}
