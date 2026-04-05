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
//! - [`InventoryMovementRepository`]: Stock history movement records
//! - [`ReservationRepository`]: Stock reservation management
//! - [`RecipeRepository`]: Recipe/BOM persistence
//! - [`AdjustmentRepository`]: Stock adjustment documents
//! - [`TransferRepository`]: Inter-store transfer documents
//!
//! ## Optimistic Locking
//!
//! The [`InventoryStockRepository`] supports optimistic locking via the
//! `update_with_version` method, which prevents concurrent update conflicts.

mod adjustment_repository;
mod category_repository;
mod inventory_movement_repository;
mod inventory_stock_repository;
mod product_repository;
mod recipe_repository;
mod reservation_repository;
mod transfer_repository;

// Re-exports
pub use adjustment_repository::AdjustmentRepository;
pub use category_repository::CategoryRepository;
pub use inventory_movement_repository::{InventoryMovementRepository, MovementQuery};
pub use inventory_stock_repository::InventoryStockRepository;
pub use product_repository::ProductRepository;
pub use recipe_repository::RecipeRepository;
pub use reservation_repository::ReservationRepository;
pub use transfer_repository::TransferRepository;
