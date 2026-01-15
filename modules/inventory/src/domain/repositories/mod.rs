//! Repository traits for inventory management.
//!
//! Repository traits define the contract for data persistence operations.
//! They are implemented by infrastructure layer components (e.g., PostgreSQL repositories).
//!
//! ## Available Repositories
//!
//! - [`CategoryRepository`]: CRUD operations for product categories
//! - [`ProductRepository`]: Product catalog persistence
//! - [`InventoryStockRepository`]: Stock records with optimistic locking support
//! - [`InventoryMovementRepository`]: Kardex movement history
//! - [`ReservationRepository`]: Stock reservation management
//! - [`RecipeRepository`]: Recipe/BOM persistence
//! - [`AdjustmentRepository`]: Stock adjustment documents
//! - [`TransferRepository`]: Inter-store transfer documents
//!
//! ## Optimistic Locking
//!
//! The [`InventoryStockRepository`] supports optimistic locking via the
//! `update_with_version` method, which prevents concurrent update conflicts.

mod category_repository;
mod product_repository;
mod inventory_stock_repository;
mod inventory_movement_repository;
mod reservation_repository;
mod recipe_repository;
mod adjustment_repository;
mod transfer_repository;

// Re-exports
pub use category_repository::CategoryRepository;
pub use product_repository::ProductRepository;
pub use inventory_stock_repository::InventoryStockRepository;
pub use inventory_movement_repository::InventoryMovementRepository;
pub use reservation_repository::ReservationRepository;
pub use recipe_repository::RecipeRepository;
pub use adjustment_repository::AdjustmentRepository;
pub use transfer_repository::TransferRepository;
