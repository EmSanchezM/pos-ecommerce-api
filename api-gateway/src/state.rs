// Application state for sharing dependencies across handlers
//
// This module defines the AppState struct that holds all shared dependencies
// for the API Gateway, following hexagonal architecture principles.

use std::sync::Arc;

use identity::{JwtTokenService, PgAuditRepository, PgStoreRepository, PgUserRepository};
use inventory::{
    PgAdjustmentRepository, PgCategoryRepository, PgInventoryMovementRepository,
    PgInventoryStockRepository, PgProductRepository, PgRecipeRepository, PgReservationRepository,
    PgTransferRepository,
};
use pos_core::PgTerminalRepository;
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
    /// Store repository for store persistence operations
    store_repo: Arc<PgStoreRepository>,
    /// Terminal repository for terminal persistence operations
    terminal_repo: Arc<PgTerminalRepository>,
    /// Audit repository for audit logging
    audit_repo: Arc<PgAuditRepository>,
    /// Token service for JWT generation and validation
    token_service: Arc<JwtTokenService>,
    // -------------------------------------------------------------------------
    // Inventory repositories
    // -------------------------------------------------------------------------
    /// Product repository for product catalog operations
    product_repo: Arc<PgProductRepository>,
    /// Category repository for product category operations
    category_repo: Arc<PgCategoryRepository>,
    /// Inventory stock repository for stock management
    stock_repo: Arc<PgInventoryStockRepository>,
    /// Reservation repository for stock reservations
    reservation_repo: Arc<PgReservationRepository>,
    /// Movement repository for inventory movement history (kardex)
    movement_repo: Arc<PgInventoryMovementRepository>,
    /// Recipe repository for recipe/BOM management
    recipe_repo: Arc<PgRecipeRepository>,
    /// Adjustment repository for stock adjustments
    adjustment_repo: Arc<PgAdjustmentRepository>,
    /// Transfer repository for inter-store transfers
    transfer_repo: Arc<PgTransferRepository>,
}

impl AppState {
    /// Creates a new AppState with the given dependencies.
    ///
    /// # Arguments
    ///
    /// * `user_repo` - User repository implementation
    /// * `store_repo` - Store repository implementation
    /// * `terminal_repo` - Terminal repository implementation
    /// * `audit_repo` - Audit repository implementation
    /// * `token_service` - Token service implementation
    /// * `product_repo` - Product repository implementation
    /// * `category_repo` - Category repository implementation
    /// * `stock_repo` - Inventory stock repository implementation
    /// * `reservation_repo` - Reservation repository implementation
    /// * `movement_repo` - Inventory movement repository implementation
    /// * `recipe_repo` - Recipe repository implementation
    /// * `adjustment_repo` - Adjustment repository implementation
    /// * `transfer_repo` - Transfer repository implementation
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_repo: Arc<PgUserRepository>,
        store_repo: Arc<PgStoreRepository>,
        terminal_repo: Arc<PgTerminalRepository>,
        audit_repo: Arc<PgAuditRepository>,
        token_service: Arc<JwtTokenService>,
        product_repo: Arc<PgProductRepository>,
        category_repo: Arc<PgCategoryRepository>,
        stock_repo: Arc<PgInventoryStockRepository>,
        reservation_repo: Arc<PgReservationRepository>,
        movement_repo: Arc<PgInventoryMovementRepository>,
        recipe_repo: Arc<PgRecipeRepository>,
        adjustment_repo: Arc<PgAdjustmentRepository>,
        transfer_repo: Arc<PgTransferRepository>,
    ) -> Self {
        Self {
            user_repo,
            store_repo,
            terminal_repo,
            audit_repo,
            token_service,
            product_repo,
            category_repo,
            stock_repo,
            reservation_repo,
            movement_repo,
            recipe_repo,
            adjustment_repo,
            transfer_repo,
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
        let pool_arc = Arc::new(pool);

        // Identity repositories
        let user_repo = Arc::new(PgUserRepository::new((*pool_arc).clone()));
        let store_repo = Arc::new(PgStoreRepository::new((*pool_arc).clone()));
        let audit_repo = Arc::new(PgAuditRepository::new((*pool_arc).clone()));

        // Core repositories
        let terminal_repo = Arc::new(PgTerminalRepository::new(pool_arc.clone()));

        // Inventory repositories
        let product_repo = Arc::new(PgProductRepository::new((*pool_arc).clone()));
        let category_repo = Arc::new(PgCategoryRepository::new((*pool_arc).clone()));
        let stock_repo = Arc::new(PgInventoryStockRepository::new((*pool_arc).clone()));
        let reservation_repo = Arc::new(PgReservationRepository::new((*pool_arc).clone()));
        let movement_repo = Arc::new(PgInventoryMovementRepository::new((*pool_arc).clone()));
        let recipe_repo = Arc::new(PgRecipeRepository::new((*pool_arc).clone()));
        let adjustment_repo = Arc::new(PgAdjustmentRepository::new((*pool_arc).clone()));
        let transfer_repo = Arc::new(PgTransferRepository::new((*pool_arc).clone()));

        // Services
        let token_service = Arc::new(JwtTokenService::new(jwt_secret));

        Self {
            user_repo,
            store_repo,
            terminal_repo,
            audit_repo,
            token_service,
            product_repo,
            category_repo,
            stock_repo,
            reservation_repo,
            movement_repo,
            recipe_repo,
            adjustment_repo,
            transfer_repo,
        }
    }

    /// Returns a reference to the user repository.
    pub fn user_repo(&self) -> Arc<PgUserRepository> {
        self.user_repo.clone()
    }

    /// Returns a reference to the store repository.
    pub fn store_repo(&self) -> Arc<PgStoreRepository> {
        self.store_repo.clone()
    }

    /// Returns a reference to the terminal repository.
    pub fn terminal_repo(&self) -> Arc<PgTerminalRepository> {
        self.terminal_repo.clone()
    }

    /// Returns a reference to the audit repository.
    pub fn audit_repo(&self) -> Arc<PgAuditRepository> {
        self.audit_repo.clone()
    }

    /// Returns a reference to the token service.
    pub fn token_service(&self) -> Arc<JwtTokenService> {
        self.token_service.clone()
    }

    // -------------------------------------------------------------------------
    // Inventory repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the product repository.
    pub fn product_repo(&self) -> Arc<PgProductRepository> {
        self.product_repo.clone()
    }

    /// Returns a reference to the category repository.
    pub fn category_repo(&self) -> Arc<PgCategoryRepository> {
        self.category_repo.clone()
    }

    /// Returns a reference to the inventory stock repository.
    pub fn stock_repo(&self) -> Arc<PgInventoryStockRepository> {
        self.stock_repo.clone()
    }

    /// Returns a reference to the reservation repository.
    pub fn reservation_repo(&self) -> Arc<PgReservationRepository> {
        self.reservation_repo.clone()
    }

    /// Returns a reference to the inventory movement repository.
    pub fn movement_repo(&self) -> Arc<PgInventoryMovementRepository> {
        self.movement_repo.clone()
    }

    /// Returns a reference to the recipe repository.
    pub fn recipe_repo(&self) -> Arc<PgRecipeRepository> {
        self.recipe_repo.clone()
    }

    /// Returns a reference to the adjustment repository.
    pub fn adjustment_repo(&self) -> Arc<PgAdjustmentRepository> {
        self.adjustment_repo.clone()
    }

    /// Returns a reference to the transfer repository.
    pub fn transfer_repo(&self) -> Arc<PgTransferRepository> {
        self.transfer_repo.clone()
    }
}
