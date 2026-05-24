use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::application::dtos::{AuthenticateBackofficeCommand, BackofficeAuthResponse};
use crate::domain::auth::BackofficeTokenService;
use crate::domain::repositories::BackofficeUserRepository;
use crate::domain::value_objects::BackofficeEmail;
use crate::error::BackofficeIdentityError;

/// Backoffice access token lifetime in seconds (7 days).
const BACKOFFICE_ACCESS_TOKEN_EXPIRES_IN: i64 = 7 * 24 * 3600;

pub struct AuthenticateBackofficeUserUseCase {
    user_repo: Arc<dyn BackofficeUserRepository>,
    token_service: Arc<dyn BackofficeTokenService>,
}

impl AuthenticateBackofficeUserUseCase {
    pub fn new(
        user_repo: Arc<dyn BackofficeUserRepository>,
        token_service: Arc<dyn BackofficeTokenService>,
    ) -> Self {
        Self {
            user_repo,
            token_service,
        }
    }

    pub async fn execute(
        &self,
        cmd: AuthenticateBackofficeCommand,
    ) -> Result<BackofficeAuthResponse, BackofficeIdentityError> {
        // Parse email into the domain value object
        let email = BackofficeEmail::new(&cmd.email)
            .map_err(|_| BackofficeIdentityError::InvalidCredentials)?;

        // find_by_email — treat missing user as invalid credentials (no info leak)
        let mut user = self
            .user_repo
            .find_by_email(&email)
            .await?
            .ok_or(BackofficeIdentityError::InvalidCredentials)?;

        // Verify password — treat wrong password as invalid credentials (no info leak)
        let parsed_hash = PasswordHash::new(user.password_hash())
            .map_err(|e| BackofficeIdentityError::PasswordHashError(e.to_string()))?;

        Argon2::default()
            .verify_password(cmd.password.as_bytes(), &parsed_hash)
            .map_err(|_| BackofficeIdentityError::InvalidCredentials)?;

        if !user.is_active() {
            return Err(BackofficeIdentityError::InvalidCredentials);
        }

        // Load permissions via repo (pass id by value — trait takes owned copy)
        let permissions = self
            .user_repo
            .list_permissions_for_user(*user.id())
            .await?
            .into_iter()
            .map(|p| p.code().as_str().to_string())
            .collect::<Vec<_>>();

        // Issue token
        let access_token = self
            .token_service
            .issue_backoffice_token(&user, &permissions)?;

        // Record login and persist
        user.record_login();
        self.user_repo.save(&user).await?;

        Ok(BackofficeAuthResponse {
            access_token,
            expires_in: BACKOFFICE_ACCESS_TOKEN_EXPIRES_IN,
            user_id: *user.id().as_uuid(),
            email: user.email().as_str().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{BackofficePermission, BackofficeRole, BackofficeUser};
    use crate::domain::repositories::BackofficeUserRepository;
    use crate::domain::value_objects::{
        BackofficeEmail, BackofficePermissionId, BackofficeRoleId, BackofficeUserId,
        PlatformPermissionCode,
    };
    use crate::error::BackofficeIdentityError;
    use async_trait::async_trait;
    use chrono::Utc;
    use common::{BackofficeClaims, TokenAudience};
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp};

    // -------------------------------------------------------------------------
    // Mock repo
    // -------------------------------------------------------------------------

    struct MockUserRepo {
        user: Option<BackofficeUser>,
        saved: Mutex<Vec<BackofficeUser>>,
    }

    impl MockUserRepo {
        fn with_user(user: BackofficeUser) -> Self {
            Self {
                user: Some(user),
                saved: Mutex::new(vec![]),
            }
        }

        fn empty() -> Self {
            Self {
                user: None,
                saved: Mutex::new(vec![]),
            }
        }

        fn last_saved(&self) -> Option<BackofficeUser> {
            self.saved.lock().unwrap().last().cloned()
        }
    }

    #[async_trait]
    impl BackofficeUserRepository for MockUserRepo {
        async fn save(&self, user: &BackofficeUser) -> Result<(), BackofficeIdentityError> {
            self.saved.lock().unwrap().push(user.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            _id: BackofficeUserId,
        ) -> Result<Option<BackofficeUser>, BackofficeIdentityError> {
            Ok(self.user.clone())
        }

        async fn find_by_email(
            &self,
            _email: &BackofficeEmail,
        ) -> Result<Option<BackofficeUser>, BackofficeIdentityError> {
            Ok(self.user.clone())
        }

        async fn update(&self, _user: &BackofficeUser) -> Result<(), BackofficeIdentityError> {
            Ok(())
        }

        async fn list(&self) -> Result<Vec<BackofficeUser>, BackofficeIdentityError> {
            Ok(vec![])
        }

        async fn list_roles_for_user(
            &self,
            _user_id: BackofficeUserId,
        ) -> Result<Vec<BackofficeRole>, BackofficeIdentityError> {
            Ok(vec![])
        }

        async fn list_permissions_for_user(
            &self,
            _user_id: BackofficeUserId,
        ) -> Result<Vec<BackofficePermission>, BackofficeIdentityError> {
            Ok(vec![make_permission("platform:org.list")])
        }

        async fn assign_role(
            &self,
            _user_id: BackofficeUserId,
            _role_id: BackofficeRoleId,
        ) -> Result<(), BackofficeIdentityError> {
            Ok(())
        }

        async fn remove_role(
            &self,
            _user_id: BackofficeUserId,
            _role_id: BackofficeRoleId,
        ) -> Result<(), BackofficeIdentityError> {
            Ok(())
        }
    }

    // -------------------------------------------------------------------------
    // Mock token service
    // -------------------------------------------------------------------------

    struct MockTokenService;

    impl BackofficeTokenService for MockTokenService {
        fn issue_backoffice_token(
            &self,
            _user: &BackofficeUser,
            _permissions: &[String],
        ) -> Result<String, BackofficeIdentityError> {
            Ok("mock.jwt.token".to_string())
        }

        fn validate_backoffice_token(
            &self,
            _token: &str,
        ) -> Result<BackofficeClaims, BackofficeIdentityError> {
            Ok(BackofficeClaims {
                sub: uuid::Uuid::new_v7(Timestamp::now(NoContext)),
                aud: TokenAudience::Backoffice,
                iss: "backoffice-api:test".to_string(),
                exp: 9999999999,
                iat: 0,
                permissions: vec![],
            })
        }
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn make_permission(code: &str) -> BackofficePermission {
        BackofficePermission::new(
            BackofficePermissionId::new(),
            PlatformPermissionCode::new(code).unwrap(),
            Some("test permission".to_string()),
            Utc::now(),
        )
    }

    fn hash_password(password: &str) -> String {
        use argon2::{
            Argon2,
            password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
        };
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    fn make_active_user(password: &str) -> BackofficeUser {
        BackofficeUser::new(
            BackofficeUserId::new(),
            BackofficeEmail::new("admin@example.com").unwrap(),
            hash_password(password),
            None,
            true,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_inactive_user(password: &str) -> BackofficeUser {
        BackofficeUser::new(
            BackofficeUserId::new(),
            BackofficeEmail::new("admin@example.com").unwrap(),
            hash_password(password),
            None,
            false, // inactive
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_use_case(user: BackofficeUser) -> AuthenticateBackofficeUserUseCase {
        AuthenticateBackofficeUserUseCase::new(
            Arc::new(MockUserRepo::with_user(user)),
            Arc::new(MockTokenService),
        )
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn happy_path_returns_access_token() {
        let user = make_active_user("correct_password");
        let uc = make_use_case(user);

        let result = uc
            .execute(AuthenticateBackofficeCommand {
                email: "admin@example.com".to_string(),
                password: "correct_password".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert!(!resp.access_token.is_empty());
        assert_eq!(resp.expires_in, BACKOFFICE_ACCESS_TOKEN_EXPIRES_IN);
    }

    #[tokio::test]
    async fn invalid_email_returns_invalid_credentials() {
        let uc = AuthenticateBackofficeUserUseCase::new(
            Arc::new(MockUserRepo::empty()),
            Arc::new(MockTokenService),
        );

        let result = uc
            .execute(AuthenticateBackofficeCommand {
                email: "unknown@example.com".to_string(),
                password: "any_password".to_string(),
            })
            .await;

        assert!(matches!(
            result,
            Err(BackofficeIdentityError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn wrong_password_returns_invalid_credentials() {
        let user = make_active_user("correct_password");
        let uc = make_use_case(user);

        let result = uc
            .execute(AuthenticateBackofficeCommand {
                email: "admin@example.com".to_string(),
                password: "wrong_password".to_string(),
            })
            .await;

        assert!(matches!(
            result,
            Err(BackofficeIdentityError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn inactive_user_returns_invalid_credentials() {
        let user = make_inactive_user("correct_password");
        let uc = make_use_case(user);

        let result = uc
            .execute(AuthenticateBackofficeCommand {
                email: "admin@example.com".to_string(),
                password: "correct_password".to_string(),
            })
            .await;

        assert!(matches!(
            result,
            Err(BackofficeIdentityError::InvalidCredentials)
        ));
    }

    #[tokio::test]
    async fn happy_path_updates_last_login_at() {
        let user = make_active_user("correct_password");
        let repo = Arc::new(MockUserRepo::with_user(user));
        let uc = AuthenticateBackofficeUserUseCase::new(repo.clone(), Arc::new(MockTokenService));

        let _result = uc
            .execute(AuthenticateBackofficeCommand {
                email: "admin@example.com".to_string(),
                password: "correct_password".to_string(),
            })
            .await
            .unwrap();

        let saved = repo.last_saved().expect("save should have been called");
        assert!(
            saved.last_login_at().is_some(),
            "last_login_at must be set after successful login"
        );
    }
}
