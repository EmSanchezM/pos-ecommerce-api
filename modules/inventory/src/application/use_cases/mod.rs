//! Use cases for inventory operations.
//!
//! Use cases implement specific business operations by coordinating domain entities
//! and repositories. Each use case follows the single responsibility principle.
//!
//! ## Product and Category Use Cases
//!
//! - [`CreateCategoryUseCase`]: Create hierarchical product categories
//! - [`CreateProductUseCase`]: Create products with auto-generated SKUs
//! - [`CreateVariantUseCase`]: Create product variants
//!
//! ## Stock Management Use Cases
//!
//! - [`UpdateStockUseCase`]: Update stock with optimistic locking
//! - [`CreateReservationUseCase`]: Reserve stock for carts/orders
//! - [`ConfirmReservationUseCase`]: Confirm and consume reserved stock
//! - [`CancelReservationUseCase`]: Cancel and release reserved stock
//! - [`ExpireReservationsUseCase`]: Batch expire old reservations
//!
//! ## Recipe Use Cases
//!
//! - [`CreateRecipeUseCase`]: Create recipes/BOMs for composite products
//! - [`CalculateRecipeCostUseCase`]: Calculate recipe cost from ingredients
//!
//! ## Adjustment Use Cases
//!
//! - [`CreateAdjustmentUseCase`]: Create stock adjustment documents
//! - [`SubmitAdjustmentUseCase`]: Submit adjustments for approval
//! - [`ApproveAdjustmentUseCase`]: Approve or reject adjustments
//! - [`ApplyAdjustmentUseCase`]: Apply approved adjustments to stock
//!
//! ## Transfer Use Cases
//!
//! - [`CreateTransferUseCase`]: Create inter-store transfer documents
//! - [`ShipTransferUseCase`]: Ship transfers and reduce source stock
//! - [`ReceiveTransferUseCase`]: Receive transfers and increase destination stock

mod create_category_use_case;
mod create_product_use_case;
mod create_variant_use_case;
mod list_products_use_case;
mod get_product_use_case;
mod update_product_use_case;
mod delete_product_use_case;
mod list_variants_use_case;
mod get_variant_use_case;
mod update_variant_use_case;
mod delete_variant_use_case;

// Stock use cases
mod initialize_stock_use_case;
mod bulk_initialize_stock_use_case;
mod update_stock_use_case;
mod update_stock_levels_use_case;
mod get_low_stock_alerts_use_case;
mod list_stock_use_case;
mod get_stock_use_case;
mod get_store_inventory_use_case;
mod get_product_stock_use_case;
mod create_reservation_use_case;
mod confirm_reservation_use_case;
mod cancel_reservation_use_case;
mod expire_reservations_use_case;
mod list_reservations_use_case;

// Stock history and report use cases
mod get_stock_history_use_case;
mod get_valuation_report_use_case;
mod get_low_stock_report_use_case;
mod get_movements_report_use_case;

// Recipe use cases
mod create_recipe_use_case;
mod calculate_recipe_cost_use_case;
mod list_recipes_use_case;
mod get_recipe_use_case;
mod get_product_recipe_use_case;
mod update_recipe_use_case;

// Adjustment use cases
mod create_adjustment_use_case;
mod submit_adjustment_use_case;
mod approve_adjustment_use_case;
mod apply_adjustment_use_case;
mod list_adjustments_use_case;
mod get_adjustment_use_case;

// Transfer use cases
mod create_transfer_use_case;
mod ship_transfer_use_case;
mod receive_transfer_use_case;

pub use create_category_use_case::CreateCategoryUseCase;
pub use create_product_use_case::CreateProductUseCase;
pub use create_variant_use_case::CreateVariantUseCase;
pub use list_products_use_case::{ListProductsUseCase, ListProductsQuery};
pub use get_product_use_case::GetProductUseCase;
pub use update_product_use_case::UpdateProductUseCase;
pub use delete_product_use_case::DeleteProductUseCase;
pub use list_variants_use_case::ListVariantsUseCase;
pub use get_variant_use_case::GetVariantUseCase;
pub use update_variant_use_case::UpdateVariantUseCase;
pub use delete_variant_use_case::DeleteVariantUseCase;

// Stock use cases exports
pub use initialize_stock_use_case::InitializeStockUseCase;
pub use bulk_initialize_stock_use_case::{BulkInitializeStockUseCase, BulkInitializeStockResult, BulkInitializeStockError};
pub use update_stock_use_case::UpdateStockUseCase;
pub use update_stock_levels_use_case::UpdateStockLevelsUseCase;
pub use get_low_stock_alerts_use_case::GetLowStockAlertsUseCase;
pub use list_stock_use_case::{ListStockUseCase, ListStockQuery};
pub use get_stock_use_case::GetStockUseCase;
pub use get_store_inventory_use_case::GetStoreInventoryUseCase;
pub use get_product_stock_use_case::GetProductStockUseCase;
pub use create_reservation_use_case::CreateReservationUseCase;
pub use confirm_reservation_use_case::ConfirmReservationUseCase;
pub use cancel_reservation_use_case::CancelReservationUseCase;
pub use expire_reservations_use_case::{ExpireReservationsUseCase, ExpireReservationsResult};
pub use list_reservations_use_case::{ListReservationsUseCase, ListReservationsQuery};

// Stock history and report use cases exports
pub use get_stock_history_use_case::{GetStockHistoryUseCase, StockHistoryQuery};
pub use get_valuation_report_use_case::{GetValuationReportUseCase, ValuationReportQuery};
pub use get_low_stock_report_use_case::{GetLowStockReportUseCase, LowStockReportQuery};
pub use get_movements_report_use_case::{GetMovementsReportUseCase, MovementsReportQuery};

// Recipe use cases exports
pub use create_recipe_use_case::CreateRecipeUseCase;
pub use calculate_recipe_cost_use_case::{CalculateRecipeCostUseCase, RecipeCostResult};
pub use list_recipes_use_case::{ListRecipesUseCase, ListRecipesQuery};
pub use get_recipe_use_case::GetRecipeUseCase;
pub use get_product_recipe_use_case::GetProductRecipeUseCase;
pub use update_recipe_use_case::UpdateRecipeUseCase;

// Adjustment use cases exports
pub use create_adjustment_use_case::CreateAdjustmentUseCase;
pub use submit_adjustment_use_case::SubmitAdjustmentUseCase;
pub use approve_adjustment_use_case::ApproveAdjustmentUseCase;
pub use apply_adjustment_use_case::ApplyAdjustmentUseCase;
pub use list_adjustments_use_case::{ListAdjustmentsUseCase, ListAdjustmentsQuery};
pub use get_adjustment_use_case::GetAdjustmentUseCase;

// Transfer use cases exports
pub use create_transfer_use_case::CreateTransferUseCase;
pub use ship_transfer_use_case::ShipTransferUseCase;
pub use receive_transfer_use_case::ReceiveTransferUseCase;
