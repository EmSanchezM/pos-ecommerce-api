// Inventory handlers module
//
// This module organizes all inventory-related HTTP handlers by domain:
// - products: Product CRUD operations
// - variants: Product variant operations
// - recipes: Recipe management and cost calculation
// - stock: Inventory stock operations

pub mod products;
pub mod recipes;
pub mod stock;
pub mod variants;

// Re-export all handlers for easy access
pub use products::*;
pub use recipes::*;
pub use stock::*;
pub use variants::*;
