// Repository traits for inventory management

mod category_repository;
mod product_repository;
mod inventory_stock_repository;
mod inventory_movement_repository;
mod reservation_repository;
mod recipe_repository;
mod adjustment_repository;
mod transfer_repository;

// Re-exports
pub use category_repository::CategoryRepository;
pub use product_repository::ProductRepository;
pub use inventory_stock_repository::InventoryStockRepository;
pub use inventory_movement_repository::InventoryMovementRepository;
pub use reservation_repository::ReservationRepository;
pub use recipe_repository::RecipeRepository;
pub use adjustment_repository::AdjustmentRepository;
pub use transfer_repository::TransferRepository;
