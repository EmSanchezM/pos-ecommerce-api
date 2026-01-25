// Inventory handlers module
//
// This module organizes all inventory-related HTTP handlers by domain:
// - products: Product CRUD operations
// - variants: Product variant operations
// - recipes: Recipe management and cost calculation
// - stock: Inventory stock operations
// - reservations: Inventory reservation operations
// - adjustments: Stock adjustment operations
// - reports: Stock history and inventory reports

pub mod adjustments;
pub mod products;
pub mod recipes;
pub mod reports;
pub mod reservations;
pub mod stock;
pub mod variants;

// Re-export all handlers for easy access
pub use adjustments::*;
pub use products::*;
pub use recipes::*;
pub use reports::*;
pub use reservations::*;
pub use stock::*;
pub use variants::*;
