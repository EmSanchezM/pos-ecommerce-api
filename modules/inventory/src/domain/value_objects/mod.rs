// Domain value objects for inventory management
// These will be implemented in later tasks (Task 2)

// ID value objects
mod product_id;
mod variant_id;
mod stock_id;
mod movement_id;
mod reservation_id;
mod recipe_id;
mod ingredient_id;
mod substitute_id;
mod adjustment_id;
mod transfer_id;
mod category_id;

// Validated value objects
mod sku;
mod barcode;
mod currency;
mod unit_of_measure;

// Enum value objects
mod movement_type;
mod reservation_status;
mod adjustment_status;
mod adjustment_type;
mod adjustment_reason;
mod transfer_status;

// Re-exports will be added as value objects are implemented
