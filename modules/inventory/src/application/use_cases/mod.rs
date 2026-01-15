// Use cases for inventory operations

mod create_category_use_case;
mod create_product_use_case;
mod create_variant_use_case;

// Stock use cases
mod update_stock_use_case;
mod create_reservation_use_case;
mod confirm_reservation_use_case;
mod cancel_reservation_use_case;
mod expire_reservations_use_case;

// Recipe use cases
mod create_recipe_use_case;
mod calculate_recipe_cost_use_case;

// Adjustment use cases
mod create_adjustment_use_case;
mod submit_adjustment_use_case;
mod approve_adjustment_use_case;
mod apply_adjustment_use_case;

// Transfer use cases
mod create_transfer_use_case;
mod ship_transfer_use_case;
mod receive_transfer_use_case;

pub use create_category_use_case::CreateCategoryUseCase;
pub use create_product_use_case::CreateProductUseCase;
pub use create_variant_use_case::CreateVariantUseCase;

// Stock use cases exports
pub use update_stock_use_case::UpdateStockUseCase;
pub use create_reservation_use_case::CreateReservationUseCase;
pub use confirm_reservation_use_case::ConfirmReservationUseCase;
pub use cancel_reservation_use_case::CancelReservationUseCase;
pub use expire_reservations_use_case::{ExpireReservationsUseCase, ExpireReservationsResult};

// Recipe use cases exports
pub use create_recipe_use_case::CreateRecipeUseCase;
pub use calculate_recipe_cost_use_case::{CalculateRecipeCostUseCase, RecipeCostResult};

// Adjustment use cases exports
pub use create_adjustment_use_case::CreateAdjustmentUseCase;
pub use submit_adjustment_use_case::SubmitAdjustmentUseCase;
pub use approve_adjustment_use_case::ApproveAdjustmentUseCase;
pub use apply_adjustment_use_case::ApplyAdjustmentUseCase;

// Transfer use cases exports
pub use create_transfer_use_case::CreateTransferUseCase;
pub use ship_transfer_use_case::ShipTransferUseCase;
pub use receive_transfer_use_case::ReceiveTransferUseCase;
