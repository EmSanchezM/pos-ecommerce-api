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

pub use create_category_use_case::CreateCategoryUseCase;
pub use create_product_use_case::CreateProductUseCase;
pub use create_variant_use_case::CreateVariantUseCase;

// Stock use cases exports
pub use update_stock_use_case::UpdateStockUseCase;
pub use create_reservation_use_case::CreateReservationUseCase;
pub use confirm_reservation_use_case::ConfirmReservationUseCase;
pub use cancel_reservation_use_case::CancelReservationUseCase;
pub use expire_reservations_use_case::{ExpireReservationsUseCase, ExpireReservationsResult};
