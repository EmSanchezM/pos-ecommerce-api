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
mod product_category;
mod product;
mod product_variant;

// Stock management
mod inventory_stock;
mod inventory_reservation;
mod inventory_movement;

// Recipe/BOM
mod recipe;
mod recipe_ingredient;
mod ingredient_substitute;

// Adjustments and transfers
mod stock_adjustment;
mod adjustment_item;
mod stock_transfer;
mod transfer_item;

// Re-exports - Product catalog
pub use product_category::ProductCategory;
pub use product::Product;
pub use product_variant::ProductVariant;

// Re-exports - Stock management
pub use inventory_stock::InventoryStock;
pub use inventory_reservation::InventoryReservation;
pub use inventory_movement::InventoryMovement;

// Re-exports - Recipe/BOM
pub use recipe::Recipe;
pub use recipe_ingredient::RecipeIngredient;
pub use ingredient_substitute::IngredientSubstitute;

// Re-exports - Adjustments and transfers
pub use stock_adjustment::StockAdjustment;
pub use adjustment_item::AdjustmentItem;
pub use stock_transfer::StockTransfer;
pub use transfer_item::TransferItem;
