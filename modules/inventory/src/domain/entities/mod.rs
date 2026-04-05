//! Domain entities for inventory management.
//!
//! Entities are domain objects with identity and lifecycle. They encapsulate
//! business rules and invariants.
//!
//! ## Product Catalog
//!
//! - [`ProductCategory`]: Hierarchical product categorization
//! - [`Product`]: Main product with SKU, pricing, and attributes
//! - [`ProductVariant`]: Product variations (size, color, etc.)
//!
//! ## Stock Management
//!
//! - [`InventoryStock`]: Stock levels per store with optimistic locking
//! - [`InventoryReservation`]: Temporary stock holds for carts/orders
//! - [`InventoryMovement`]: Kardex entries tracking all stock changes
//!
//! ## Recipe/BOM
//!
//! - [`Recipe`]: Bill of materials for composite products
//! - [`RecipeIngredient`]: Components required by a recipe
//! - [`IngredientSubstitute`]: Alternative ingredients with conversion ratios
//!
//! ## Adjustments and Transfers
//!
//! - [`StockAdjustment`]: Inventory corrections with approval workflow
//! - [`AdjustmentItem`]: Line items within an adjustment
//! - [`StockTransfer`]: Inter-store inventory movement
//! - [`TransferItem`]: Line items within a transfer

// Product catalog
mod product;
mod product_category;
mod product_variant;

// Stock management
mod inventory_movement;
mod inventory_reservation;
mod inventory_stock;

// Recipe/BOM
mod ingredient_substitute;
mod recipe;
mod recipe_ingredient;

// Adjustments and transfers
mod adjustment_item;
mod stock_adjustment;
mod stock_transfer;
mod transfer_item;

// Re-exports - Product catalog
pub use product::Product;
pub use product_category::ProductCategory;
pub use product_variant::ProductVariant;

// Re-exports - Stock management
pub use inventory_movement::InventoryMovement;
pub use inventory_reservation::InventoryReservation;
pub use inventory_stock::InventoryStock;

// Re-exports - Recipe/BOM
pub use ingredient_substitute::IngredientSubstitute;
pub use recipe::Recipe;
pub use recipe_ingredient::RecipeIngredient;

// Re-exports - Adjustments and transfers
pub use adjustment_item::AdjustmentItem;
pub use stock_adjustment::StockAdjustment;
pub use stock_transfer::StockTransfer;
pub use transfer_item::TransferItem;
