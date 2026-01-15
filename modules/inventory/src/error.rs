//! Inventory module error types.
//!
//! This module defines all error types that can occur during inventory operations.
//! Errors are categorized by domain area (products, stock, reservations, etc.).

use thiserror::Error;
use uuid::Uuid;

/// Error type for all inventory module operations.
///
/// This enum covers all possible error conditions that can occur when working
/// with the inventory module, including validation errors, not-found errors,
/// and workflow constraint violations.
///
/// # Example
///
/// ```rust,ignore
/// use inventory::InventoryError;
///
/// fn handle_error(err: InventoryError) {
///     match err {
///         InventoryError::ProductNotFound(id) => {
///             println!("Product {} not found", id);
///         }
///         InventoryError::InsufficientStock => {
///             println!("Not enough stock available");
///         }
///         _ => println!("Other error: {}", err),
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum InventoryError {
    // -------------------------------------------------------------------------
    // Category errors
    // -------------------------------------------------------------------------
    
    /// The requested category was not found in the database.
    #[error("Category not found: {0}")]
    CategoryNotFound(Uuid),

    /// A category with the given slug already exists.
    #[error("Category slug '{0}' already exists")]
    DuplicateCategorySlug(String),

    /// The specified parent category does not exist.
    #[error("Parent category not found: {0}")]
    ParentCategoryNotFound(Uuid),

    // -------------------------------------------------------------------------
    // Product errors
    // -------------------------------------------------------------------------
    
    /// The requested product was not found in the database.
    #[error("Product not found: {0}")]
    ProductNotFound(Uuid),

    /// A product with the given SKU already exists.
    #[error("SKU '{0}' already exists")]
    DuplicateSku(String),

    /// A product with the given barcode already exists.
    #[error("Barcode '{0}' already exists")]
    DuplicateBarcode(String),

    /// Attempted to create a variant for a product that doesn't have variants enabled.
    #[error("Product does not have variants enabled")]
    VariantsNotEnabled,

    // -------------------------------------------------------------------------
    // Variant errors
    // -------------------------------------------------------------------------
    
    /// The requested product variant was not found.
    #[error("Variant not found: {0}")]
    VariantNotFound(Uuid),

    // -------------------------------------------------------------------------
    // Stock errors
    // -------------------------------------------------------------------------
    
    /// The requested stock record was not found.
    #[error("Stock not found: {0}")]
    StockNotFound(Uuid),

    /// There is not enough available stock to complete the operation.
    #[error("Insufficient stock available")]
    InsufficientStock,

    /// The operation would result in negative stock, which is not allowed.
    #[error("Cannot have negative stock")]
    NegativeStock,

    /// The reserved quantity would exceed the total quantity.
    #[error("Reserved quantity cannot exceed total quantity")]
    ReservedExceedsQuantity,

    /// Attempted to release more quantity than is currently reserved.
    #[error("Invalid release quantity")]
    InvalidReleaseQuantity,

    /// The stock record was modified by another process (optimistic locking failure).
    /// Retry the operation with the updated version.
    #[error("Optimistic lock error: record was modified by another process")]
    OptimisticLockError,

    // -------------------------------------------------------------------------
    // Reservation errors
    // -------------------------------------------------------------------------
    
    /// The requested reservation was not found.
    #[error("Reservation not found: {0}")]
    ReservationNotFound(Uuid),

    /// The reservation has already expired.
    #[error("Reservation has expired")]
    ReservationExpired,

    /// Invalid status transition for a reservation.
    #[error("Invalid reservation status transition")]
    InvalidReservationStatus,

    // -------------------------------------------------------------------------
    // Recipe errors
    // -------------------------------------------------------------------------
    
    /// The requested recipe was not found.
    #[error("Recipe not found: {0}")]
    RecipeNotFound(Uuid),

    /// An active recipe already exists for this product or variant.
    /// Only one active recipe is allowed per product/variant.
    #[error("Active recipe already exists for this product/variant")]
    ActiveRecipeExists,

    /// The requested ingredient was not found.
    #[error("Ingredient not found: {0}")]
    IngredientNotFound(Uuid),

    /// Cannot delete an ingredient that is used in active recipes.
    #[error("Cannot delete ingredient used in active recipes")]
    IngredientInUse,

    /// Recipe yield quantity must be greater than zero.
    #[error("Yield quantity must be positive")]
    InvalidYieldQuantity,

    /// Ingredient quantity must be greater than zero.
    #[error("Ingredient quantity must be positive")]
    InvalidIngredientQuantity,

    /// Waste percentage must be between 0 and 1 (0% to 100%).
    #[error("Waste percentage must be between 0 and 1")]
    InvalidWastePercentage,

    /// Conversion ratio for substitutes must be greater than zero.
    #[error("Conversion ratio must be positive")]
    InvalidConversionRatio,

    /// Substitute priority must be non-negative.
    #[error("Substitute priority must be non-negative")]
    InvalidSubstitutePriority,

    /// The ingredient does not allow substitutes (can_substitute is false).
    #[error("Ingredient does not allow substitutes")]
    SubstitutesNotAllowed,

    /// The requested substitute was not found.
    #[error("Substitute not found: {0}")]
    SubstituteNotFound(Uuid),

    // -------------------------------------------------------------------------
    // Adjustment errors
    // -------------------------------------------------------------------------
    
    /// The requested adjustment was not found.
    #[error("Adjustment not found: {0}")]
    AdjustmentNotFound(Uuid),

    /// Cannot submit an adjustment with no items.
    #[error("Adjustment has no items")]
    EmptyAdjustment,

    /// Cannot modify an adjustment that has already been applied.
    #[error("Cannot modify applied adjustment")]
    AdjustmentAlreadyApplied,

    // -------------------------------------------------------------------------
    // Transfer errors
    // -------------------------------------------------------------------------
    
    /// The requested transfer was not found.
    #[error("Transfer not found: {0}")]
    TransferNotFound(Uuid),

    /// Cannot create a transfer where source and destination are the same store.
    #[error("Cannot transfer to the same store")]
    SameStoreTransfer,

    /// Cannot submit a transfer with no items.
    #[error("Transfer has no items")]
    EmptyTransfer,

    // -------------------------------------------------------------------------
    // Workflow errors
    // -------------------------------------------------------------------------
    
    /// The requested status transition is not valid for the current state.
    #[error("Invalid status transition")]
    InvalidStatusTransition,

    // -------------------------------------------------------------------------
    // Validation errors
    // -------------------------------------------------------------------------
    
    /// Currency code must be exactly 3 uppercase letters (ISO 4217 format).
    #[error("Invalid currency code: must be 3 uppercase letters (ISO 4217)")]
    InvalidCurrency,

    /// Barcode exceeds maximum length of 100 characters.
    #[error("Invalid barcode: maximum 100 characters")]
    InvalidBarcode,

    /// The provided unit of measure is not recognized.
    #[error("Invalid unit of measure")]
    InvalidUnitOfMeasure,

    /// The provided movement type is not recognized.
    #[error("Invalid movement type")]
    InvalidMovementType,

    /// The provided adjustment type is not recognized.
    #[error("Invalid adjustment type")]
    InvalidAdjustmentType,

    /// The provided adjustment reason is not recognized.
    #[error("Invalid adjustment reason")]
    InvalidAdjustmentReason,

    /// The provided transfer status is not recognized.
    #[error("Invalid transfer status")]
    InvalidTransferStatus,

    /// The provided reservation status is not recognized.
    #[error("Invalid reservation status")]
    InvalidReservationStatusValue,

    /// The provided adjustment status is not recognized.
    #[error("Invalid adjustment status")]
    InvalidAdjustmentStatus,

    /// Must specify exactly one of product_id or variant_id, not both or neither.
    #[error("Must specify either product_id or variant_id, but not both")]
    InvalidProductVariantConstraint,

    // -------------------------------------------------------------------------
    // Database errors
    // -------------------------------------------------------------------------
    
    /// A database error occurred during the operation.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // -------------------------------------------------------------------------
    // General errors
    // -------------------------------------------------------------------------
    
    /// The requested functionality is not yet implemented.
    #[error("Not implemented")]
    NotImplemented,
}
