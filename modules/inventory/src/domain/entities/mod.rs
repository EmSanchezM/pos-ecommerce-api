// Domain entities for inventory management
// These will be implemented in later tasks (Task 3+)

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

// Re-exports will be added as entities are implemented
