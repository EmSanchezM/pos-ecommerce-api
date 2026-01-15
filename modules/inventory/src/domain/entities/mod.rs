// Domain entities for inventory management

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

// Re-exports for Recipe/BOM, Adjustments, and Transfers will be added as entities are implemented
