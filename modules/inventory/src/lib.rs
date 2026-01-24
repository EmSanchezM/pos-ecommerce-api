//! # Inventory Module
//!
//! Comprehensive inventory management for a multi-store POS and e-commerce system.
//!
//! This module provides:
//! - **Product Catalog Management**: Products with SKUs, barcodes, pricing, and hierarchical categories
//! - **Product Variants**: Size, color, flavor variations with independent pricing
//! - **Stock Control**: Per-store inventory tracking with optimistic locking
//! - **Inventory Reservations**: Temporary holds for shopping carts with automatic expiration
//! - **Movement History (Kardex)**: Complete audit trail of all stock changes
//! - **Recipe/BOM Management**: Bill of materials for composite products
//! - **Stock Adjustments**: Inventory corrections with approval workflow
//! - **Inter-store Transfers**: Stock movement between locations with shipping workflow
//!
//! ## Architecture
//!
//! The module follows hexagonal/clean architecture with three layers:
//!
//! - **Domain Layer**: Core business logic, entities, value objects, repository traits
//! - **Application Layer**: Use cases, DTOs, orchestration
//! - **Infrastructure Layer**: PostgreSQL repository implementations
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use inventory::{
//!     // Use cases
//!     CreateProductUseCase, CreateCategoryUseCase, UpdateStockUseCase,
//!     // DTOs
//!     CreateProductCommand, CreateCategoryCommand, UpdateStockCommand,
//!     // Repositories
//!     PgProductRepository, PgCategoryRepository, PgInventoryStockRepository,
//!     // Entities and value objects
//!     Product, ProductCategory, InventoryStock, ProductId, Sku,
//! };
//! ```
//!
//! ## Example: Creating a Product
//!
//! ```rust,ignore
//! let command = CreateProductCommand {
//!     name: "Widget".to_string(),
//!     unit_of_measure: "unit".to_string(),
//!     base_price: Decimal::new(1999, 2), // $19.99
//!     cost_price: Decimal::new(1000, 2), // $10.00
//!     ..Default::default()
//! };
//!
//! let use_case = CreateProductUseCase::new(product_repo, category_repo, audit_repo);
//! let product = use_case.execute(command, actor_id).await?;
//! ```

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

// =============================================================================
// Public API - Re-exports for convenient access
// =============================================================================

/// Error type for all inventory operations
pub use error::InventoryError;

// -----------------------------------------------------------------------------
// Domain Layer - Value Objects
// -----------------------------------------------------------------------------

// ID value objects - UUID v7 based identifiers for temporal ordering
pub use domain::value_objects::AdjustmentId;
pub use domain::value_objects::CategoryId;
pub use domain::value_objects::IngredientId;
pub use domain::value_objects::MovementId;
pub use domain::value_objects::ProductId;
pub use domain::value_objects::RecipeId;
pub use domain::value_objects::ReservationId;
pub use domain::value_objects::StockId;
pub use domain::value_objects::SubstituteId;
pub use domain::value_objects::TransferId;
pub use domain::value_objects::VariantId;

// Validated value objects
pub use domain::value_objects::Barcode;
pub use domain::value_objects::Currency;
pub use domain::value_objects::Sku;
pub use domain::value_objects::UnitOfMeasure;

// Enum value objects
pub use domain::value_objects::AdjustmentReason;
pub use domain::value_objects::AdjustmentStatus;
pub use domain::value_objects::AdjustmentType;
pub use domain::value_objects::MovementType;
pub use domain::value_objects::ReservationStatus;
pub use domain::value_objects::TransferStatus;

// -----------------------------------------------------------------------------
// Domain Layer - Entities
// -----------------------------------------------------------------------------

// Product catalog entities
pub use domain::entities::Product;
pub use domain::entities::ProductCategory;
pub use domain::entities::ProductVariant;

// Stock management entities
pub use domain::entities::InventoryMovement;
pub use domain::entities::InventoryReservation;
pub use domain::entities::InventoryStock;

// Recipe/BOM entities
pub use domain::entities::IngredientSubstitute;
pub use domain::entities::Recipe;
pub use domain::entities::RecipeIngredient;

// Adjustment and transfer entities
pub use domain::entities::AdjustmentItem;
pub use domain::entities::StockAdjustment;
pub use domain::entities::StockTransfer;
pub use domain::entities::TransferItem;

// -----------------------------------------------------------------------------
// Domain Layer - Repository Traits
// -----------------------------------------------------------------------------

pub use domain::repositories::AdjustmentRepository;
pub use domain::repositories::CategoryRepository;
pub use domain::repositories::InventoryMovementRepository;
pub use domain::repositories::InventoryStockRepository;
pub use domain::repositories::ProductRepository;
pub use domain::repositories::RecipeRepository;
pub use domain::repositories::ReservationRepository;
pub use domain::repositories::TransferRepository;

// -----------------------------------------------------------------------------
// Application Layer - Use Cases
// -----------------------------------------------------------------------------

// Product and category use cases
pub use application::use_cases::CreateCategoryUseCase;
pub use application::use_cases::CreateProductUseCase;
pub use application::use_cases::CreateVariantUseCase;
pub use application::use_cases::ListProductsUseCase;
pub use application::use_cases::ListProductsQuery;
pub use application::use_cases::GetProductUseCase;
pub use application::use_cases::UpdateProductUseCase;
pub use application::use_cases::DeleteProductUseCase;
pub use application::use_cases::ListVariantsUseCase;
pub use application::use_cases::GetVariantUseCase;
pub use application::use_cases::UpdateVariantUseCase;
pub use application::use_cases::DeleteVariantUseCase;

// Stock management use cases
pub use application::use_cases::BulkInitializeStockError;
pub use application::use_cases::BulkInitializeStockResult;
pub use application::use_cases::BulkInitializeStockUseCase;
pub use application::use_cases::CancelReservationUseCase;
pub use application::use_cases::ConfirmReservationUseCase;
pub use application::use_cases::CreateReservationUseCase;
pub use application::use_cases::ExpireReservationsResult;
pub use application::use_cases::ExpireReservationsUseCase;
pub use application::use_cases::GetLowStockAlertsUseCase;
pub use application::use_cases::InitializeStockUseCase;
pub use application::use_cases::ListReservationsUseCase;
pub use application::use_cases::ListReservationsQuery;
pub use application::use_cases::UpdateStockLevelsUseCase;
pub use application::use_cases::UpdateStockUseCase;
pub use application::use_cases::ListStockUseCase;
pub use application::use_cases::ListStockQuery;
pub use application::use_cases::GetStockUseCase;
pub use application::use_cases::GetStoreInventoryUseCase;
pub use application::use_cases::GetProductStockUseCase;

// Recipe use cases
pub use application::use_cases::CalculateRecipeCostUseCase;
pub use application::use_cases::CreateRecipeUseCase;
pub use application::use_cases::RecipeCostResult;
pub use application::use_cases::ListRecipesUseCase;
pub use application::use_cases::ListRecipesQuery;
pub use application::use_cases::GetRecipeUseCase;
pub use application::use_cases::GetProductRecipeUseCase;
pub use application::use_cases::UpdateRecipeUseCase;

// Adjustment use cases
pub use application::use_cases::ApplyAdjustmentUseCase;
pub use application::use_cases::ApproveAdjustmentUseCase;
pub use application::use_cases::CreateAdjustmentUseCase;
pub use application::use_cases::SubmitAdjustmentUseCase;
pub use application::use_cases::ListAdjustmentsUseCase;
pub use application::use_cases::ListAdjustmentsQuery;
pub use application::use_cases::GetAdjustmentUseCase;

// Transfer use cases
pub use application::use_cases::CreateTransferUseCase;
pub use application::use_cases::ReceiveTransferUseCase;
pub use application::use_cases::ShipTransferUseCase;

// -----------------------------------------------------------------------------
// Application Layer - Command DTOs
// -----------------------------------------------------------------------------

// Category commands
pub use application::dtos::CreateCategoryCommand;
pub use application::dtos::UpdateCategoryCommand;

// Product commands
pub use application::dtos::CreateProductCommand;
pub use application::dtos::CreateVariantCommand;
pub use application::dtos::UpdateProductCommand;
pub use application::dtos::UpdateVariantCommand;

// Stock commands
pub use application::dtos::BulkInitializeStockCommand;
pub use application::dtos::BulkInitializeStockItem;
pub use application::dtos::InitializeStockCommand;
pub use application::dtos::UpdateStockCommand;
pub use application::dtos::UpdateStockLevelsCommand;

// Reservation commands
pub use application::dtos::CancelReservationCommand;
pub use application::dtos::ConfirmReservationCommand;
pub use application::dtos::CreateReservationCommand;

// Recipe commands
pub use application::dtos::CreateRecipeCommand;
pub use application::dtos::IngredientSubstituteCommand;
pub use application::dtos::RecipeIngredientCommand;
pub use application::dtos::UpdateRecipeCommand;

// Adjustment commands
pub use application::dtos::AdjustmentItemCommand;
pub use application::dtos::ApplyAdjustmentCommand;
pub use application::dtos::ApproveAdjustmentCommand;
pub use application::dtos::CreateAdjustmentCommand;
pub use application::dtos::SubmitAdjustmentCommand;

// Transfer commands
pub use application::dtos::CreateTransferCommand;
pub use application::dtos::ReceiveTransferCommand;
pub use application::dtos::ReceiveTransferItemCommand;
pub use application::dtos::ShipTransferCommand;
pub use application::dtos::ShipTransferItemCommand;
pub use application::dtos::TransferItemCommand;

// -----------------------------------------------------------------------------
// Application Layer - Response DTOs
// -----------------------------------------------------------------------------

// Category responses
pub use application::dtos::CategoryResponse;
pub use application::dtos::CategoryTreeResponse;

// Product responses
pub use application::dtos::ProductDetailResponse;
pub use application::dtos::ProductResponse;
pub use application::dtos::VariantResponse;

// Stock responses
pub use application::dtos::StockDetailResponse;
pub use application::dtos::StockResponse;

// Reservation responses
pub use application::dtos::ReservationResponse;

// Movement responses
pub use application::dtos::KardexResponse;
pub use application::dtos::MovementResponse;

// Recipe responses
pub use application::dtos::IngredientSubstituteResponse;
pub use application::dtos::RecipeDetailResponse;
pub use application::dtos::RecipeIngredientResponse;
pub use application::dtos::RecipeResponse;

// Adjustment responses
pub use application::dtos::AdjustmentDetailResponse;
pub use application::dtos::AdjustmentItemResponse;
pub use application::dtos::AdjustmentResponse;

// Transfer responses
pub use application::dtos::TransferDetailResponse;
pub use application::dtos::TransferItemResponse;
pub use application::dtos::TransferResponse;

// Pagination
pub use application::dtos::PaginatedResponse;

// -----------------------------------------------------------------------------
// Infrastructure Layer - PostgreSQL Repositories
// -----------------------------------------------------------------------------

pub use infrastructure::persistence::PgAdjustmentRepository;
pub use infrastructure::persistence::PgCategoryRepository;
pub use infrastructure::persistence::PgInventoryMovementRepository;
pub use infrastructure::persistence::PgInventoryStockRepository;
pub use infrastructure::persistence::PgProductRepository;
pub use infrastructure::persistence::PgRecipeRepository;
pub use infrastructure::persistence::PgReservationRepository;
pub use infrastructure::persistence::PgTransferRepository;
