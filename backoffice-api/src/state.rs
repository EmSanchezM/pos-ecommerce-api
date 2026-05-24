// BackofficeAppState — DI wiring for the backoffice binary
//
// Holds all shared dependencies for the backoffice API.
// Mirrors api-gateway/src/state.rs but scoped to backoffice concerns only.

use std::sync::Arc;

use backoffice_identity::{
    AuthenticateBackofficeUserUseCase, BackofficeTokenService, BackofficeUserRepository,
    JwtBackofficeTokenService, PgBackofficeUserRepository,
};
use sqlx::PgPool;

/// Application state shared across all backoffice HTTP handlers.
///
/// Follows the same pattern as `api-gateway/src/state.rs`.
#[derive(Clone)]
pub struct BackofficeAppState {
    /// Direct access to the PostgreSQL connection pool for transactional operations.
    pool: PgPool,
    /// Backoffice user repository.
    user_repo: Arc<dyn BackofficeUserRepository>,
    /// Token service for validating incoming backoffice JWTs in the auth middleware.
    token_service: Arc<dyn BackofficeTokenService>,
    /// Use case: authenticate a backoffice user and issue a JWT.
    authenticate_use_case: Arc<AuthenticateBackofficeUserUseCase>,
}

impl BackofficeAppState {
    /// Construct BackofficeAppState from a pool and JWT secrets.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `backoffice_secret` - Secret for signing/validating backoffice JWTs
    /// * `backoffice_issuer` - Issuer string embedded in backoffice tokens
    pub fn from_pool(pool: PgPool, backoffice_secret: String, backoffice_issuer: String) -> Self {
        let pool_arc = Arc::new(pool.clone());

        let user_repo: Arc<dyn BackofficeUserRepository> =
            Arc::new(PgBackofficeUserRepository::new((*pool_arc).clone()));

        let token_service = Arc::new(JwtBackofficeTokenService::with_issuer(
            backoffice_secret,
            backoffice_issuer,
        ));

        let authenticate_use_case = Arc::new(AuthenticateBackofficeUserUseCase::new(
            user_repo.clone(),
            token_service.clone(),
        ));

        Self {
            pool,
            user_repo,
            token_service,
            authenticate_use_case,
        }
    }

    /// Returns a reference to the PostgreSQL connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Returns the backoffice user repository.
    pub fn user_repo(&self) -> Arc<dyn BackofficeUserRepository> {
        self.user_repo.clone()
    }

    /// Returns the backoffice token service (used by auth middleware for validation).
    pub fn token_service(&self) -> Arc<dyn BackofficeTokenService> {
        self.token_service.clone()
    }

    /// Returns the authenticate backoffice user use case.
    pub fn authenticate_use_case(&self) -> Arc<AuthenticateBackofficeUserUseCase> {
        self.authenticate_use_case.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// P3-T01: BackofficeAppState::from_pool constructs without panic and
    /// exposes the expected use cases and pool.
    ///
    /// We create a PgPool pointing to a fictional URL — this test verifies
    /// that the state object is correctly wired at the type level without
    /// actually connecting to a database. We use `PgPool::connect_lazy` so
    /// no real connection is attempted.
    #[tokio::test]
    async fn backoffice_app_state_constructs_and_exposes_use_case() {
        // lazy connect — no real DB needed
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");

        let state = BackofficeAppState::from_pool(
            pool,
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
        );

        // authenticate_use_case is Arc-wrapped and clone-able
        let _uc = state.authenticate_use_case();
        // pool is accessible
        let _pool = state.pool();
        // user_repo is accessible
        let _repo = state.user_repo();
    }

    #[tokio::test]
    async fn backoffice_app_state_is_clone() {
        let pool = PgPool::connect_lazy("postgres://test:test@localhost/test")
            .expect("connect_lazy should not fail");

        let state = BackofficeAppState::from_pool(
            pool,
            "backoffice-secret-at-least-32-bytes-long".to_string(),
            "backoffice-api:test".to_string(),
        );

        // State must be Clone so Axum can share it across handlers.
        let _clone = state.clone();
    }
}
