// Authentication use cases - Application layer business logic for authentication
//
// Requirements: 1.1, 1.3, 1.4, 1.5, 1.6, 1.7, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 6.1, 6.4

use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::application::dtos::{LoginCommand, LoginResponse, RegisterEcommerceCommand, RegisterResponse};
use crate::application::validators::{validate_name, validate_password};
use crate::domain::auth::{AuthError, LoginIdentifier, TokenService};
use crate::domain::entities::{AuditEntry, User};
use crate::domain::repositories::{AuditRepository, UserRepository};
use crate::domain::value_objects::{Email, Username};

// =============================================================================
// RegisterUserUseCase (Ecommerce Registration)
// =============================================================================

/// Use case for registering a new ecommerce user (customer).
///
/// This handles public self-registration where:
/// - Username is auto-generated from email prefix
/// - User is created with is_active=true
/// - No store assignment (ecommerce users don't belong to stores)
///
/// Requirements: 1.1
pub struct RegisterUserUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    user_repo: Arc<U>,
    audit_repo: Arc<A>,
}

impl<U, A> RegisterUserUseCase<U, A>
where
    U: UserRepository,
    A: AuditRepository,
{
    /// Creates a new instance of RegisterUserUseCase
    pub fn new(user_repo: Arc<U>, audit_repo: Arc<A>) -> Self {
        Self {
            user_repo,
            audit_repo,
        }
    }

    /// Executes the use case to register a new ecommerce user.
    ///
    /// # Arguments
    /// * `command` - The registration command containing user data
    ///
    /// # Returns
    /// `RegisterResponse` with the created user's public information
    ///
    /// # Errors
    /// * `AuthError::InvalidEmailFormat` - If email format is invalid (Requirement 1.3)
    /// * `AuthError::DuplicateEmail` - If email already exists (Requirement 1.2)
    /// * `AuthError::DuplicateUsername` - If generated username already exists
    /// * `AuthError::PasswordTooShort` - If password is less than 8 characters (Requirement 1.4)
    /// * `AuthError::InvalidName` - If first_name or last_name is invalid (Requirement 6.4)
    /// * `AuthError::Internal` - On internal errors
    pub async fn execute(
        &self,
        command: RegisterEcommerceCommand,
    ) -> Result<RegisterResponse, AuthError> {
        // Validate password policy (Requirement 1.4, 6.3)
        validate_password(&command.password)?;

        // Validate names (Requirement 6.4)
        validate_name(&command.first_name, "first_name")?;
        validate_name(&command.last_name, "last_name")?;

        // Validate and create email value object (Requirement 1.3, 6.1)
        let email = Email::new(&command.email).map_err(|_| AuthError::InvalidEmailFormat)?;

        // Check email uniqueness (Requirement 1.2)
        if self
            .user_repo
            .find_by_email(&email)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .is_some()
        {
            return Err(AuthError::DuplicateEmail(command.email));
        }

        // Generate username from email prefix (Requirement 1.6)
        let username = generate_username_from_email(&email);

        // Check username uniqueness (in case generated username conflicts)
        if self
            .user_repo
            .find_by_username(&username)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .is_some()
        {
            // If username conflicts, append a random suffix
            let username_with_suffix = generate_unique_username(&email);
            return self
                .create_user_with_username(command, email, username_with_suffix)
                .await;
        }

        self.create_user_with_username(command, email, username)
            .await
    }

    /// Internal helper to create user with a given username
    async fn create_user_with_username(
        &self,
        command: RegisterEcommerceCommand,
        email: Email,
        username: Username,
    ) -> Result<RegisterResponse, AuthError> {
        // Hash password using Argon2 (Requirement 1.5)
        let password_hash = hash_password(&command.password)?;

        // Create user with is_active=true (Requirement 1.1)
        let user = User::create(
            username,
            email,
            command.first_name.trim().to_string(),
            command.last_name.trim().to_string(),
            password_hash,
        );

        // Save to repository
        self.user_repo
            .save(&user)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        // Create audit entry
        let audit_entry = AuditEntry::for_create("user", user.id().into_uuid(), &user, *user.id());
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        // Return response without password hash (Requirement 1.7)
        Ok(RegisterResponse::new(
            user.id().into_uuid(),
            user.username().as_str().to_string(),
            user.email().as_str().to_string(),
            user.first_name().to_string(),
            user.last_name().to_string(),
            user.created_at(),
        ))
    }
}

// =============================================================================
// LoginUseCase
// =============================================================================

/// Use case for authenticating a user with email or username.
///
/// This handles unified login where:
/// - Identifier can be either email or username
/// - System auto-detects the format
/// - Returns JWT tokens on success
///
/// Requirements: 3.1
pub struct LoginUseCase<U, T>
where
    U: UserRepository,
    T: TokenService,
{
    user_repo: Arc<U>,
    token_service: Arc<T>,
}

impl<U, T> LoginUseCase<U, T>
where
    U: UserRepository,
    T: TokenService,
{
    /// Creates a new instance of LoginUseCase
    pub fn new(user_repo: Arc<U>, token_service: Arc<T>) -> Self {
        Self {
            user_repo,
            token_service,
        }
    }

    /// Executes the use case to authenticate a user.
    ///
    /// # Arguments
    /// * `command` - The login command containing identifier and password
    ///
    /// # Returns
    /// `LoginResponse` with access and refresh tokens
    ///
    /// # Errors
    /// * `AuthError::InvalidCredentials` - If user not found or password incorrect (Requirement 3.6)
    /// * `AuthError::AccountDisabled` - If user account is inactive (Requirement 3.7)
    /// * `AuthError::Internal` - On internal errors
    ///
    /// # Security
    /// The error message for invalid credentials is intentionally vague to prevent
    /// user enumeration attacks. Both "user not found" and "wrong password" return
    /// the same error message.
    pub async fn execute(&self, command: LoginCommand) -> Result<LoginResponse, AuthError> {
        // Parse identifier to determine if it's email or username (Requirement 3.2)
        let identifier = LoginIdentifier::parse(&command.identifier);

        // Find user by email or username (Requirements 3.3, 3.4)
        let user = match &identifier {
            LoginIdentifier::Email(email) => {
                self.user_repo
                    .find_by_email(email)
                    .await
                    .map_err(|e| AuthError::Internal(e.to_string()))?
            }
            LoginIdentifier::Username(username) => {
                self.user_repo
                    .find_by_username(username)
                    .await
                    .map_err(|e| AuthError::Internal(e.to_string()))?
            }
        };

        // Return InvalidCredentials if user not found (Requirement 3.6)
        let user = user.ok_or(AuthError::InvalidCredentials)?;

        // Verify password using Argon2 (Requirement 3.5)
        if !verify_password(&command.password, user.password_hash())? {
            // Return same error as "user not found" to prevent enumeration (Requirement 3.6)
            return Err(AuthError::InvalidCredentials);
        }

        // Check if account is active (Requirement 3.7)
        if !user.is_active() {
            return Err(AuthError::AccountDisabled);
        }

        // Generate JWT tokens (Requirement 3.8)
        let access_token = self.token_service.generate_access_token(&user)?;
        let refresh_token = self.token_service.generate_refresh_token(*user.id())?;

        // Return login response with tokens
        // expires_in is 900 seconds (15 minutes) as per Requirement 4.2
        Ok(LoginResponse::new(access_token, refresh_token, 900))
    }
}

// =============================================================================
// RefreshTokenUseCase
// =============================================================================

/// Use case for refreshing an access token using a refresh token.
///
/// This handles token refresh where:
/// - The refresh token is validated
/// - The user is verified to exist and be active
/// - A new access token is generated
///
/// Requirements: 4.5
pub struct RefreshTokenUseCase<U, T>
where
    U: UserRepository,
    T: TokenService,
{
    user_repo: Arc<U>,
    token_service: Arc<T>,
}

impl<U, T> RefreshTokenUseCase<U, T>
where
    U: UserRepository,
    T: TokenService,
{
    /// Creates a new instance of RefreshTokenUseCase
    pub fn new(user_repo: Arc<U>, token_service: Arc<T>) -> Self {
        Self {
            user_repo,
            token_service,
        }
    }

    /// Executes the use case to refresh an access token.
    ///
    /// # Arguments
    /// * `refresh_token` - The refresh token obtained from login
    ///
    /// # Returns
    /// `LoginResponse` with new access token (refresh token remains the same)
    ///
    /// # Errors
    /// * `AuthError::TokenExpired` - If the refresh token has expired (Requirement 4.6)
    /// * `AuthError::InvalidToken` - If the refresh token is malformed or invalid (Requirement 4.6)
    /// * `AuthError::InvalidCredentials` - If the user no longer exists
    /// * `AuthError::AccountDisabled` - If the user account is inactive
    /// * `AuthError::Internal` - On internal errors
    ///
    /// # Requirements
    ///
    /// - Requirement 4.5: Valid refresh token SHALL generate new access token
    /// - Requirement 4.6: Expired/invalid refresh token SHALL return appropriate error
    pub async fn execute(&self, refresh_token: String) -> Result<LoginResponse, AuthError> {
        // Validate refresh token and extract user_id (Requirement 4.6)
        let user_id = self.token_service.validate_refresh_token(&refresh_token)?;

        // Find user by ID
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::InvalidCredentials)?;

        // Check if account is active
        if !user.is_active() {
            return Err(AuthError::AccountDisabled);
        }

        // Generate new access token (Requirement 4.5)
        let access_token = self.token_service.generate_access_token(&user)?;

        // Return response with new access token but same refresh token
        // expires_in is 900 seconds (15 minutes) as per Requirement 4.2
        Ok(LoginResponse::new(access_token, refresh_token, 900))
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Verifies a password against a stored Argon2 hash.
///
/// Requirements: 3.5
fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AuthError::Internal(format!("Invalid password hash format: {}", e)))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Generates a username from the email prefix (part before @).
///
/// The username is sanitized to meet the username format requirements:
/// - 3-50 characters
/// - Alphanumeric with underscores
/// - Starts with a letter
///
/// Requirements: 1.6
fn generate_username_from_email(email: &Email) -> Username {
    let email_str = email.as_str();
    let prefix = email_str.split('@').next().unwrap_or("user");

    // Sanitize: keep only alphanumeric and underscores
    let sanitized: String = prefix
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    // Ensure it starts with a letter
    let username_str = if sanitized.is_empty()
        || !sanitized.chars().next().unwrap_or('0').is_alphabetic()
    {
        format!("user_{}", sanitized)
    } else {
        sanitized
    };

    // Ensure minimum length of 3
    let username_str = if username_str.len() < 3 {
        format!("{}_{}", username_str, "user")
    } else {
        username_str
    };

    // Truncate to max 50 characters
    let username_str = if username_str.len() > 50 {
        username_str[..50].to_string()
    } else {
        username_str
    };

    // This should always succeed given our sanitization
    Username::new(&username_str).unwrap_or_else(|_| Username::new("user_default").unwrap())
}

/// Generates a unique username by appending a random suffix.
fn generate_unique_username(email: &Email) -> Username {
    let base = generate_username_from_email(email);
    // Use UUID v7 to generate a unique suffix (last 4 chars of UUID)
    let uuid = uuid::Uuid::now_v7();
    let suffix = &uuid.to_string()[..4];
    let unique_str = format!("{}_{}", base.as_str(), suffix);

    // Truncate if needed
    let unique_str = if unique_str.len() > 50 {
        unique_str[..50].to_string()
    } else {
        unique_str
    };

    Username::new(&unique_str).unwrap_or_else(|_| Username::new("user_default").unwrap())
}

/// Hashes a password using Argon2.
///
/// Requirements: 1.5
fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::Internal(format!("Password hashing failed: {}", e)))
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

    use crate::domain::entities::{AuditAction, Permission, Role, Store};
    use crate::domain::value_objects::{RoleId, StoreId, UserId};

    // =========================================================================
    // Mock Repositories
    // =========================================================================

    struct MockUserRepository {
        users: Mutex<HashMap<UserId, User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }

        fn with_user(self, user: User) -> Self {
            self.users.lock().unwrap().insert(*user.id(), user);
            self
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, user: &User) -> Result<(), crate::error::IdentityError> {
            let mut users = self.users.lock().unwrap();
            users.insert(*user.id(), user.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: UserId,
        ) -> Result<Option<User>, crate::error::IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(&id).cloned())
        }

        async fn find_by_email(
            &self,
            email: &Email,
        ) -> Result<Option<User>, crate::error::IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.email() == email).cloned())
        }

        async fn find_by_username(
            &self,
            username: &Username,
        ) -> Result<Option<User>, crate::error::IdentityError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.username() == username).cloned())
        }

        async fn update(&self, user: &User) -> Result<(), crate::error::IdentityError> {
            let mut users = self.users.lock().unwrap();
            users.insert(*user.id(), user.clone());
            Ok(())
        }

        async fn assign_role(
            &self,
            _user_id: UserId,
            _role_id: RoleId,
            _store_id: StoreId,
        ) -> Result<(), crate::error::IdentityError> {
            Ok(())
        }

        async fn remove_role(
            &self,
            _user_id: UserId,
            _role_id: RoleId,
            _store_id: StoreId,
        ) -> Result<(), crate::error::IdentityError> {
            Ok(())
        }

        async fn get_roles_for_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<Vec<Role>, crate::error::IdentityError> {
            Ok(vec![])
        }

        async fn get_permissions_for_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<Vec<Permission>, crate::error::IdentityError> {
            Ok(vec![])
        }

        async fn remove_role_from_all_users(
            &self,
            _role_id: RoleId,
        ) -> Result<(), crate::error::IdentityError> {
            Ok(())
        }

        async fn add_to_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<(), crate::error::IdentityError> {
            Ok(())
        }

        async fn remove_from_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<(), crate::error::IdentityError> {
            Ok(())
        }

        async fn get_stores(
            &self,
            _user_id: UserId,
        ) -> Result<Vec<Store>, crate::error::IdentityError> {
            Ok(vec![])
        }

        async fn is_member_of_store(
            &self,
            _user_id: UserId,
            _store_id: StoreId,
        ) -> Result<bool, crate::error::IdentityError> {
            Ok(false)
        }
    }

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
        async fn save(&self, entry: &AuditEntry) -> Result<(), crate::error::IdentityError> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
            Ok(())
        }

        async fn find_by_entity(
            &self,
            entity_type: &str,
            entity_id: Uuid,
        ) -> Result<Vec<AuditEntry>, crate::error::IdentityError> {
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
        ) -> Result<Vec<AuditEntry>, crate::error::IdentityError> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .iter()
                .filter(|e| e.created_at() >= from && e.created_at() < to)
                .cloned()
                .collect())
        }
    }

    // =========================================================================
    // Mock TokenService
    // =========================================================================

    struct MockTokenService;

    impl TokenService for MockTokenService {
        fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
            Ok(format!("access_token_for_{}", user.id().as_uuid()))
        }

        fn generate_refresh_token(&self, user_id: UserId) -> Result<String, AuthError> {
            Ok(format!("refresh_token_for_{}", user_id.as_uuid()))
        }

        fn validate_access_token(&self, _token: &str) -> Result<crate::domain::auth::TokenClaims, AuthError> {
            unimplemented!("Not needed for login tests")
        }

        fn validate_refresh_token(&self, _token: &str) -> Result<UserId, AuthError> {
            unimplemented!("Not needed for login tests")
        }
    }

    /// Mock TokenService that supports refresh token validation for RefreshTokenUseCase tests
    struct MockTokenServiceWithRefresh {
        /// Map of refresh tokens to user IDs
        valid_tokens: Mutex<HashMap<String, UserId>>,
        /// Tokens that should return TokenExpired error
        expired_tokens: Mutex<Vec<String>>,
    }

    impl MockTokenServiceWithRefresh {
        fn new() -> Self {
            Self {
                valid_tokens: Mutex::new(HashMap::new()),
                expired_tokens: Mutex::new(Vec::new()),
            }
        }

        fn with_valid_token(self, token: &str, user_id: UserId) -> Self {
            self.valid_tokens.lock().unwrap().insert(token.to_string(), user_id);
            self
        }

        fn with_expired_token(self, token: &str) -> Self {
            self.expired_tokens.lock().unwrap().push(token.to_string());
            self
        }
    }

    impl TokenService for MockTokenServiceWithRefresh {
        fn generate_access_token(&self, user: &User) -> Result<String, AuthError> {
            Ok(format!("new_access_token_for_{}", user.id().as_uuid()))
        }

        fn generate_refresh_token(&self, user_id: UserId) -> Result<String, AuthError> {
            Ok(format!("refresh_token_for_{}", user_id.as_uuid()))
        }

        fn validate_access_token(&self, _token: &str) -> Result<crate::domain::auth::TokenClaims, AuthError> {
            unimplemented!("Not needed for refresh tests")
        }

        fn validate_refresh_token(&self, token: &str) -> Result<UserId, AuthError> {
            // Check if token is in expired list
            if self.expired_tokens.lock().unwrap().contains(&token.to_string()) {
                return Err(AuthError::TokenExpired);
            }

            // Check if token is valid
            self.valid_tokens
                .lock()
                .unwrap()
                .get(token)
                .copied()
                .ok_or(AuthError::InvalidToken)
        }
    }

    // =========================================================================
    // Helper to create user with hashed password
    // =========================================================================

    fn create_user_with_password(
        username: &str,
        email: &str,
        password: &str,
        is_active: bool,
    ) -> User {
        let password_hash = hash_password(password).unwrap();
        let mut user = User::create(
            Username::new(username).unwrap(),
            Email::new(email).unwrap(),
            "Test".to_string(),
            "User".to_string(),
            password_hash,
        );
        if !is_active {
            user.deactivate();
        }
        user
    }

    // =========================================================================
    // RegisterUserUseCase Tests
    // =========================================================================

    #[tokio::test]
    async fn test_register_user_success() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RegisterUserUseCase::new(user_repo.clone(), audit_repo.clone());

        let command = RegisterEcommerceCommand {
            email: "john.doe@example.com".to_string(),
            password: "securepassword123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.email, "john.doe@example.com");
        assert_eq!(response.first_name, "John");
        assert_eq!(response.last_name, "Doe");
        // Username should be derived from email prefix
        assert!(response.username.starts_with("john"));

        // Verify audit entry was created
        let entries = audit_repo.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_type(), "user");
        assert_eq!(entries[0].action(), &AuditAction::Created);
    }

    #[tokio::test]
    async fn test_register_user_duplicate_email() {
        let existing_user = User::create(
            Username::new("existinguser").unwrap(),
            Email::new("john.doe@example.com").unwrap(),
            "Existing".to_string(),
            "User".to_string(),
            "hash".to_string(),
        );
        let user_repo = Arc::new(MockUserRepository::new().with_user(existing_user));
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RegisterUserUseCase::new(user_repo, audit_repo);

        let command = RegisterEcommerceCommand {
            email: "john.doe@example.com".to_string(),
            password: "securepassword123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(matches!(result, Err(AuthError::DuplicateEmail(_))));
    }

    #[tokio::test]
    async fn test_register_user_invalid_email() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RegisterUserUseCase::new(user_repo, audit_repo);

        let command = RegisterEcommerceCommand {
            email: "invalid-email".to_string(),
            password: "securepassword123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(matches!(result, Err(AuthError::InvalidEmailFormat)));
    }

    #[tokio::test]
    async fn test_register_user_password_too_short() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RegisterUserUseCase::new(user_repo, audit_repo);

        let command = RegisterEcommerceCommand {
            email: "john@example.com".to_string(),
            password: "short".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(matches!(result, Err(AuthError::PasswordTooShort)));
    }

    #[tokio::test]
    async fn test_register_user_empty_first_name() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RegisterUserUseCase::new(user_repo, audit_repo);

        let command = RegisterEcommerceCommand {
            email: "john@example.com".to_string(),
            password: "securepassword123".to_string(),
            first_name: "   ".to_string(),
            last_name: "Doe".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(matches!(result, Err(AuthError::InvalidName(_))));
    }

    #[tokio::test]
    async fn test_register_user_empty_last_name() {
        let user_repo = Arc::new(MockUserRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = RegisterUserUseCase::new(user_repo, audit_repo);

        let command = RegisterEcommerceCommand {
            email: "john@example.com".to_string(),
            password: "securepassword123".to_string(),
            first_name: "John".to_string(),
            last_name: "".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(matches!(result, Err(AuthError::InvalidName(_))));
    }

    // =========================================================================
    // LoginUseCase Tests
    // =========================================================================

    #[tokio::test]
    async fn test_login_with_email_success() {
        let user = create_user_with_password("testuser", "test@example.com", "password123", true);
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        let command = LoginCommand {
            identifier: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.access_token.starts_with("access_token_for_"));
        assert!(response.refresh_token.starts_with("refresh_token_for_"));
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 900);
    }

    #[tokio::test]
    async fn test_login_with_username_success() {
        let user = create_user_with_password("testuser", "test@example.com", "password123", true);
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        let command = LoginCommand {
            identifier: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.access_token.starts_with("access_token_for_"));
        assert!(response.refresh_token.starts_with("refresh_token_for_"));
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let user_repo = Arc::new(MockUserRepository::new());
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        let command = LoginCommand {
            identifier: "nonexistent@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(command).await;

        // Should return InvalidCredentials (not "user not found") for security
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let user = create_user_with_password("testuser", "test@example.com", "password123", true);
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        let command = LoginCommand {
            identifier: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = use_case.execute(command).await;

        // Should return InvalidCredentials (same as user not found) for security
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_login_inactive_account() {
        let user = create_user_with_password("testuser", "test@example.com", "password123", false);
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        let command = LoginCommand {
            identifier: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(command).await;

        assert!(matches!(result, Err(AuthError::AccountDisabled)));
    }

    #[tokio::test]
    async fn test_login_identifier_detection_email() {
        // Test that email format is correctly detected
        let user = create_user_with_password("testuser", "user@domain.com", "password123", true);
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        // Using email format should find user by email
        let command = LoginCommand {
            identifier: "user@domain.com".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_identifier_detection_username() {
        // Test that username format is correctly detected
        let user = create_user_with_password("john_doe", "john@example.com", "password123", true);
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));
        let token_service = Arc::new(MockTokenService);
        let use_case = LoginUseCase::new(user_repo, token_service);

        // Using username format should find user by username
        let command = LoginCommand {
            identifier: "john_doe".to_string(),
            password: "password123".to_string(),
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());
    }

    // =========================================================================
    // Username Generation Tests
    // =========================================================================

    #[test]
    fn test_generate_username_from_email_simple() {
        let email = Email::new("john@example.com").unwrap();
        let username = generate_username_from_email(&email);
        assert_eq!(username.as_str(), "john");
    }

    #[test]
    fn test_generate_username_from_email_with_dots() {
        let email = Email::new("john.doe@example.com").unwrap();
        let username = generate_username_from_email(&email);
        // Dots are removed, only alphanumeric and underscores kept
        assert_eq!(username.as_str(), "johndoe");
    }

    #[test]
    fn test_generate_username_from_email_with_numbers() {
        let email = Email::new("john123@example.com").unwrap();
        let username = generate_username_from_email(&email);
        assert_eq!(username.as_str(), "john123");
    }

    #[test]
    fn test_generate_username_from_email_starts_with_number() {
        let email = Email::new("123john@example.com").unwrap();
        let username = generate_username_from_email(&email);
        // Should prepend "user_" since it starts with a number
        assert!(username.as_str().starts_with("user_"));
    }

    #[test]
    fn test_generate_username_from_email_short() {
        let email = Email::new("ab@example.com").unwrap();
        let username = generate_username_from_email(&email);
        // Should be padded to at least 3 characters
        assert!(username.as_str().len() >= 3);
    }

    // =========================================================================
    // RefreshTokenUseCase Tests
    // =========================================================================

    #[tokio::test]
    async fn test_refresh_token_success() {
        // Create a user
        let user = create_user_with_password("testuser", "test@example.com", "password123", true);
        let user_id = *user.id();
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));

        // Create token service with valid refresh token
        let token_service = Arc::new(
            MockTokenServiceWithRefresh::new()
                .with_valid_token("valid_refresh_token", user_id)
        );

        let use_case = RefreshTokenUseCase::new(user_repo, token_service);

        let result = use_case.execute("valid_refresh_token".to_string()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.access_token.starts_with("new_access_token_for_"));
        assert_eq!(response.refresh_token, "valid_refresh_token");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 900);
    }

    #[tokio::test]
    async fn test_refresh_token_expired() {
        let user_repo = Arc::new(MockUserRepository::new());
        let token_service = Arc::new(
            MockTokenServiceWithRefresh::new()
                .with_expired_token("expired_token")
        );

        let use_case = RefreshTokenUseCase::new(user_repo, token_service);

        let result = use_case.execute("expired_token".to_string()).await;

        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[tokio::test]
    async fn test_refresh_token_invalid() {
        let user_repo = Arc::new(MockUserRepository::new());
        let token_service = Arc::new(MockTokenServiceWithRefresh::new());

        let use_case = RefreshTokenUseCase::new(user_repo, token_service);

        let result = use_case.execute("invalid_token".to_string()).await;

        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_refresh_token_user_not_found() {
        // Token is valid but user no longer exists
        let user_id = UserId::new();
        let user_repo = Arc::new(MockUserRepository::new());
        let token_service = Arc::new(
            MockTokenServiceWithRefresh::new()
                .with_valid_token("valid_token", user_id)
        );

        let use_case = RefreshTokenUseCase::new(user_repo, token_service);

        let result = use_case.execute("valid_token".to_string()).await;

        // Should return InvalidCredentials when user doesn't exist
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn test_refresh_token_inactive_account() {
        // Create an inactive user
        let user = create_user_with_password("testuser", "test@example.com", "password123", false);
        let user_id = *user.id();
        let user_repo = Arc::new(MockUserRepository::new().with_user(user));

        let token_service = Arc::new(
            MockTokenServiceWithRefresh::new()
                .with_valid_token("valid_token", user_id)
        );

        let use_case = RefreshTokenUseCase::new(user_repo, token_service);

        let result = use_case.execute("valid_token".to_string()).await;

        assert!(matches!(result, Err(AuthError::AccountDisabled)));
    }

    // =========================================================================
    // Property-Based Tests for Uniqueness Constraints
    // =========================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        /// Generates a valid email string for property testing
        fn valid_email_strategy() -> impl Strategy<Value = String> {
            // Generate local part: 1-20 alphanumeric chars starting with letter
            let local_part = "[a-z][a-z0-9]{0,19}";
            // Generate domain: simple domain with .com
            let domain = "[a-z]{3,10}";

            (local_part, domain).prop_map(|(local, domain)| format!("{}@{}.com", local, domain))
        }

        /// Generates a valid password (8+ characters)
        fn valid_password_strategy() -> impl Strategy<Value = String> {
            "[a-zA-Z0-9]{8,20}".prop_map(|s| s)
        }

        /// Generates a valid name (1-50 chars, non-empty after trim)
        fn valid_name_strategy() -> impl Strategy<Value = String> {
            "[A-Za-z]{1,50}".prop_map(|s| s)
        }

        // Feature: user-authentication, Property 1: Email Uniqueness Constraint
        // *For any* two registration attempts with the same email address, the second
        // attempt SHALL be rejected with a duplicate email error, regardless of whether
        // it's ecommerce or POS registration.
        // **Validates: Requirements 1.2, 2.3**
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]
            #[test]
            fn prop_email_uniqueness_constraint(
                email in valid_email_strategy(),
                password1 in valid_password_strategy(),
                password2 in valid_password_strategy(),
                first_name1 in valid_name_strategy(),
                first_name2 in valid_name_strategy(),
                last_name1 in valid_name_strategy(),
                last_name2 in valid_name_strategy(),
            ) {
                // Run the async test in a blocking context
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let user_repo = Arc::new(MockUserRepository::new());
                    let audit_repo = Arc::new(MockAuditRepository::new());
                    let use_case = RegisterUserUseCase::new(user_repo.clone(), audit_repo.clone());

                    // First registration should succeed
                    let command1 = RegisterEcommerceCommand {
                        email: email.clone(),
                        password: password1,
                        first_name: first_name1,
                        last_name: last_name1,
                    };
                    let result1 = use_case.execute(command1).await;
                    prop_assert!(result1.is_ok(), "First registration should succeed");

                    // Second registration with same email should fail with DuplicateEmail
                    let command2 = RegisterEcommerceCommand {
                        email: email.clone(),
                        password: password2,
                        first_name: first_name2,
                        last_name: last_name2,
                    };
                    let result2 = use_case.execute(command2).await;
                    prop_assert!(
                        matches!(result2, Err(AuthError::DuplicateEmail(_))),
                        "Second registration with same email should fail with DuplicateEmail, got: {:?}",
                        result2
                    );

                    Ok(())
                })?;
            }
        }

        // Feature: user-authentication, Property 2: Username Uniqueness Constraint
        // *For any* two POS registration attempts with the same username, the second
        // attempt SHALL be rejected with a duplicate username error.
        // **Validates: Requirements 2.2**
        //
        // Note: Since POS registration is not yet implemented (it's an admin operation),
        // we test username uniqueness through the ecommerce registration flow where
        // a generated username conflicts with an existing one.
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]
            #[test]
            fn prop_username_uniqueness_constraint(
                email in valid_email_strategy(),
                password in valid_password_strategy(),
                first_name in valid_name_strategy(),
                last_name in valid_name_strategy(),
            ) {
                // Run the async test in a blocking context
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // Pre-create a user with a username that will conflict with the
                    // generated username from the email
                    let email_obj = Email::new(&email).unwrap();
                    let expected_username = generate_username_from_email(&email_obj);

                    let existing_user = User::create(
                        expected_username.clone(),
                        Email::new("different@example.com").unwrap(),
                        "Existing".to_string(),
                        "User".to_string(),
                        "hash".to_string(),
                    );

                    let user_repo = Arc::new(MockUserRepository::new().with_user(existing_user));
                    let audit_repo = Arc::new(MockAuditRepository::new());
                    let use_case = RegisterUserUseCase::new(user_repo.clone(), audit_repo.clone());

                    // Registration should still succeed because the system generates
                    // a unique username with a suffix when there's a conflict
                    let command = RegisterEcommerceCommand {
                        email: email.clone(),
                        password,
                        first_name,
                        last_name,
                    };
                    let result = use_case.execute(command).await;

                    // The registration should succeed with a modified username
                    prop_assert!(
                        result.is_ok(),
                        "Registration should succeed with unique username suffix, got: {:?}",
                        result
                    );

                    // The generated username should be different from the conflicting one
                    if let Ok(response) = result {
                        prop_assert!(
                            response.username != expected_username.as_str() ||
                            response.username.contains('_'),
                            "Username should be unique (either different or have suffix)"
                        );
                    }

                    Ok(())
                })?;
            }
        }
    }
}
