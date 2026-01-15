// Inventory module errors

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InventoryError {
    // Category errors
    #[error("Category not found: {0}")]
    CategoryNotFound(Uuid),

    #[error("Category slug '{0}' already exists")]
    DuplicateCategorySlug(String),

    #[error("Parent category not found: {0}")]
    ParentCategoryNotFound(Uuid),

    // Product errors
    #[error("Product not found: {0}")]
    ProductNotFound(Uuid),

    #[error("SKU '{0}' already exists")]
    DuplicateSku(String),

    #[error("Barcode '{0}' already exists")]
    DuplicateBarcode(String),

    #[error("Product does not have variants enabled")]
    VariantsNotEnabled,

    // Variant errors
    #[error("Variant not found: {0}")]
    VariantNotFound(Uuid),

    // Stock errors
    #[error("Stock not found: {0}")]
    StockNotFound(Uuid),

    #[error("Insufficient stock available")]
    InsufficientStock,

    #[error("Cannot have negative stock")]
    NegativeStock,

    #[error("Reserved quantity cannot exceed total quantity")]
    ReservedExceedsQuantity,

    #[error("Invalid release quantity")]
    InvalidReleaseQuantity,

    #[error("Optimistic lock error: record was modified by another process")]
    OptimisticLockError,

    // Reservation errors
    #[error("Reservation not found: {0}")]
    ReservationNotFound(Uuid),

    #[error("Reservation has expired")]
    ReservationExpired,

    #[error("Invalid reservation status transition")]
    InvalidReservationStatus,

    // Recipe errors
    #[error("Recipe not found: {0}")]
    RecipeNotFound(Uuid),

    #[error("Active recipe already exists for this product/variant")]
    ActiveRecipeExists,

    #[error("Ingredient not found: {0}")]
    IngredientNotFound(Uuid),

    #[error("Cannot delete ingredient used in active recipes")]
    IngredientInUse,

    #[error("Yield quantity must be positive")]
    InvalidYieldQuantity,

    #[error("Ingredient quantity must be positive")]
    InvalidIngredientQuantity,

    #[error("Waste percentage must be between 0 and 1")]
    InvalidWastePercentage,

    #[error("Conversion ratio must be positive")]
    InvalidConversionRatio,

    #[error("Substitute priority must be non-negative")]
    InvalidSubstitutePriority,

    #[error("Ingredient does not allow substitutes")]
    SubstitutesNotAllowed,

    #[error("Substitute not found: {0}")]
    SubstituteNotFound(Uuid),

    // Adjustment errors
    #[error("Adjustment not found: {0}")]
    AdjustmentNotFound(Uuid),

    #[error("Adjustment has no items")]
    EmptyAdjustment,

    #[error("Cannot modify applied adjustment")]
    AdjustmentAlreadyApplied,

    // Transfer errors
    #[error("Transfer not found: {0}")]
    TransferNotFound(Uuid),

    #[error("Cannot transfer to the same store")]
    SameStoreTransfer,

    #[error("Transfer has no items")]
    EmptyTransfer,

    // Workflow errors
    #[error("Invalid status transition")]
    InvalidStatusTransition,

    // Validation errors
    #[error("Invalid currency code: must be 3 uppercase letters (ISO 4217)")]
    InvalidCurrency,

    #[error("Invalid barcode: maximum 100 characters")]
    InvalidBarcode,

    #[error("Invalid unit of measure")]
    InvalidUnitOfMeasure,

    #[error("Invalid movement type")]
    InvalidMovementType,

    #[error("Invalid adjustment type")]
    InvalidAdjustmentType,

    #[error("Invalid adjustment reason")]
    InvalidAdjustmentReason,

    #[error("Invalid transfer status")]
    InvalidTransferStatus,

    #[error("Invalid reservation status")]
    InvalidReservationStatusValue,

    #[error("Invalid adjustment status")]
    InvalidAdjustmentStatus,

    #[error("Must specify either product_id or variant_id, but not both")]
    InvalidProductVariantConstraint,

    // Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // General errors
    #[error("Not implemented")]
    NotImplemented,
}
