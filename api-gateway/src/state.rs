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
use purchasing::{PgGoodsReceiptRepository, PgPurchaseOrderRepository, PgVendorRepository};
use sales::{PgCartRepository, PgCreditNoteRepository, PgCustomerRepository, PgSaleRepository, PgShiftRepository};
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
    /// Movement repository for inventory movement history (stock ledger)
    movement_repo: Arc<PgInventoryMovementRepository>,
    /// Recipe repository for recipe/BOM management
    recipe_repo: Arc<PgRecipeRepository>,
    /// Adjustment repository for stock adjustments
    adjustment_repo: Arc<PgAdjustmentRepository>,
    /// Transfer repository for inter-store transfers
    transfer_repo: Arc<PgTransferRepository>,
    // -------------------------------------------------------------------------
    // Purchasing repositories
    // -------------------------------------------------------------------------
    /// Vendor repository for vendor/supplier management
    vendor_repo: Arc<PgVendorRepository>,
    /// Purchase order repository for purchase order management
    purchase_order_repo: Arc<PgPurchaseOrderRepository>,
    /// Goods receipt repository for goods receipt management
    goods_receipt_repo: Arc<PgGoodsReceiptRepository>,
    // -------------------------------------------------------------------------
    // Sales repositories
    // -------------------------------------------------------------------------
    /// Customer repository for customer management
    customer_repo: Arc<PgCustomerRepository>,
    /// Sale repository for sale transaction operations
    sale_repo: Arc<PgSaleRepository>,
    /// Shift repository for cashier shift management
    shift_repo: Arc<PgShiftRepository>,
    /// Cart repository for e-commerce cart management
    cart_repo: Arc<PgCartRepository>,
    /// Credit note repository for returns management
    credit_note_repo: Arc<PgCreditNoteRepository>,
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
    /// * `vendor_repo` - Vendor repository implementation
    /// * `purchase_order_repo` - Purchase order repository implementation
    /// * `goods_receipt_repo` - Goods receipt repository implementation
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
        vendor_repo: Arc<PgVendorRepository>,
        purchase_order_repo: Arc<PgPurchaseOrderRepository>,
        goods_receipt_repo: Arc<PgGoodsReceiptRepository>,
        customer_repo: Arc<PgCustomerRepository>,
        sale_repo: Arc<PgSaleRepository>,
        shift_repo: Arc<PgShiftRepository>,
        cart_repo: Arc<PgCartRepository>,
        credit_note_repo: Arc<PgCreditNoteRepository>,
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
            vendor_repo,
            purchase_order_repo,
            goods_receipt_repo,
            customer_repo,
            sale_repo,
            shift_repo,
            cart_repo,
            credit_note_repo,
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

        // Purchasing repositories
        let vendor_repo = Arc::new(PgVendorRepository::new((*pool_arc).clone()));
        let purchase_order_repo = Arc::new(PgPurchaseOrderRepository::new((*pool_arc).clone()));
        let goods_receipt_repo = Arc::new(PgGoodsReceiptRepository::new((*pool_arc).clone()));

        // Sales repositories
        let customer_repo = Arc::new(PgCustomerRepository::new((*pool_arc).clone()));
        let sale_repo = Arc::new(PgSaleRepository::new((*pool_arc).clone()));
        let shift_repo = Arc::new(PgShiftRepository::new((*pool_arc).clone()));
        let cart_repo = Arc::new(PgCartRepository::new((*pool_arc).clone()));
        let credit_note_repo = Arc::new(PgCreditNoteRepository::new((*pool_arc).clone()));

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
            vendor_repo,
            purchase_order_repo,
            goods_receipt_repo,
            customer_repo,
            sale_repo,
            shift_repo,
            cart_repo,
            credit_note_repo,
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

    // -------------------------------------------------------------------------
    // Purchasing repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the vendor repository.
    pub fn vendor_repo(&self) -> Arc<PgVendorRepository> {
        self.vendor_repo.clone()
    }

    /// Returns a reference to the purchase order repository.
    pub fn purchase_order_repo(&self) -> Arc<PgPurchaseOrderRepository> {
        self.purchase_order_repo.clone()
    }

    /// Returns a reference to the goods receipt repository.
    pub fn goods_receipt_repo(&self) -> Arc<PgGoodsReceiptRepository> {
        self.goods_receipt_repo.clone()
    }

    // -------------------------------------------------------------------------
    // Sales repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the customer repository.
    pub fn customer_repo(&self) -> Arc<PgCustomerRepository> {
        self.customer_repo.clone()
    }

    /// Returns a reference to the sale repository.
    pub fn sale_repo(&self) -> Arc<PgSaleRepository> {
        self.sale_repo.clone()
    }

    /// Returns a reference to the shift repository.
    pub fn shift_repo(&self) -> Arc<PgShiftRepository> {
        self.shift_repo.clone()
    }

    /// Returns a reference to the cart repository.
    pub fn cart_repo(&self) -> Arc<PgCartRepository> {
        self.cart_repo.clone()
    }

    /// Returns a reference to the credit note repository.
    pub fn credit_note_repo(&self) -> Arc<PgCreditNoteRepository> {
        self.credit_note_repo.clone()
    }
}
