// Application state for sharing dependencies across handlers
//
// This module defines the AppState struct that holds all shared dependencies
// for the API Gateway, following hexagonal architecture principles.
//
// Requirements: 5.1, 5.3, 5.4

use std::sync::Arc;

use identity::{JwtTokenService, PgAuditRepository, PgUserRepository};
use sqlx::PgPool;

/// Application state shared across all HTTP handlers.
///
/// This struct holds Arc-wrapped instances of repositories and services
/// that are needed by the authentication handlers. Using Arc allows
/// efficient sharing across async tasks without cloning the underlying data.
///
/// # Architecture
///
/// The AppState uses concrete PostgreSQL implementations for production.
/// For testing, handlers can be tested with mock implementations directly.
#[derive(Clone)]
pub struct AppState {
    /// User repository for user persistence operations
    user_repo: Arc<PgUserRepository>,
    /// Audit repository for audit logging
    audit_repo: Arc<PgAuditRepository>,
    /// Token service for JWT generation and validation
    token_service: Arc<JwtTokenService>,
}

impl AppState {
    /// Creates a new AppState with the given dependencies.
    ///
    /// # Arguments
    ///
    /// * `user_repo` - User repository implementation
    /// * `audit_repo` - Audit repository implementation
    /// * `token_service` - Token service implementation
    pub fn new(
        user_repo: Arc<PgUserRepository>,
        audit_repo: Arc<PgAuditRepository>,
        token_service: Arc<JwtTokenService>,
    ) -> Self {
        Self {
            user_repo,
            audit_repo,
            token_service,
        }
    }

    /// Creates an AppState from a PostgreSQL connection pool and JWT secret.
    ///
    /// This is a convenience constructor for production use that creates
    /// the concrete PostgreSQL repository implementations.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `jwt_secret` - Secret key for JWT signing (should be at least 32 bytes)
    pub fn from_pool(pool: PgPool, jwt_secret: String) -> Self {
        let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
        let audit_repo = Arc::new(PgAuditRepository::new(pool));
        let token_service = Arc::new(JwtTokenService::new(jwt_secret));

        Self {
            user_repo,
            audit_repo,
            token_service,
        }
    }

    /// Returns a reference to the user repository.
    pub fn user_repo(&self) -> Arc<PgUserRepository> {
        self.user_repo.clone()
    }

    /// Returns a reference to the audit repository.
    pub fn audit_repo(&self) -> Arc<PgAuditRepository> {
        self.audit_repo.clone()
    }

    /// Returns a reference to the token service.
    pub fn token_service(&self) -> Arc<JwtTokenService> {
        self.token_service.clone()
    }
}
