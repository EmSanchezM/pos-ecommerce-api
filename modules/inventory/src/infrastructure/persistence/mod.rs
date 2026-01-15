//! PostgreSQL repository implementations.
//!
//! This module provides PostgreSQL-based implementations of all repository traits
//! defined in the domain layer.
//!
//! ## Available Repositories
//!
//! - [`PgCategoryRepository`]: Product category persistence
//! - [`PgProductRepository`]: Product catalog with cascade delete for variants
//! - [`PgInventoryStockRepository`]: Stock records with optimistic locking
//! - [`PgReservationRepository`]: Stock reservations with expiration queries
//! - [`PgInventoryMovementRepository`]: Kardex with weighted average cost calculation
//! - [`PgRecipeRepository`]: Recipe/BOM persistence
//! - [`PgAdjustmentRepository`]: Stock adjustments with number generation
//! - [`PgTransferRepository`]: Inter-store transfers with number generation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use inventory::{PgProductRepository, ProductRepository};
//! use sqlx::PgPool;
//! use std::sync::Arc;
//!
//! let pool = Arc::new(PgPool::connect("postgres://...").await?);
//! let repo = PgProductRepository::new(pool);
//!
//! // Use the repository
//! let product = repo.find_by_id(product_id).await?;
//! ```

mod pg_category_repository;
mod pg_product_repository;
mod pg_inventory_stock_repository;
mod pg_reservation_repository;
mod pg_inventory_movement_repository;
mod pg_recipe_repository;
mod pg_adjustment_repository;
mod pg_transfer_repository;

// Re-exports
pub use pg_category_repository::PgCategoryRepository;
pub use pg_product_repository::PgProductRepository;
pub use pg_inventory_stock_repository::PgInventoryStockRepository;
pub use pg_reservation_repository::PgReservationRepository;
pub use pg_inventory_movement_repository::PgInventoryMovementRepository;
pub use pg_recipe_repository::PgRecipeRepository;
pub use pg_adjustment_repository::PgAdjustmentRepository;
pub use pg_transfer_repository::PgTransferRepository;
