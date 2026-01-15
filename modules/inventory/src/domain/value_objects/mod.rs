//! Domain value objects for inventory management.
//!
//! Value objects are immutable objects defined by their attributes rather than identity.
//! They encapsulate validation rules and provide type safety.
//!
//! ## ID Value Objects
//!
//! All IDs use UUID v7 for temporal ordering:
//!
//! - [`ProductId`], [`VariantId`], [`CategoryId`]: Product catalog identifiers
//! - [`StockId`], [`MovementId`], [`ReservationId`]: Stock management identifiers
//! - [`RecipeId`], [`IngredientId`], [`SubstituteId`]: Recipe/BOM identifiers
//! - [`AdjustmentId`], [`TransferId`]: Workflow document identifiers
//!
//! ## Validated Value Objects
//!
//! - [`Sku`]: Auto-generated stock keeping unit codes
//! - [`Barcode`]: Optional product barcodes (max 100 chars)
//! - [`Currency`]: ISO 4217 currency codes (3 uppercase letters)
//! - [`UnitOfMeasure`]: Measurement units (Unit, Kg, Lb, Liter, Oz)
//!
//! ## Enum Value Objects
//!
//! - [`MovementType`]: Types of inventory movements (In, Out, Adjustment, etc.)
//! - [`ReservationStatus`]: Reservation lifecycle states
//! - [`AdjustmentStatus`]: Adjustment workflow states
//! - [`AdjustmentType`]: Increase or decrease adjustments
//! - [`AdjustmentReason`]: Reasons for adjustments (Damage, Theft, etc.)
//! - [`TransferStatus`]: Transfer workflow states

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

// Re-exports - ID value objects
pub use product_id::ProductId;
pub use variant_id::VariantId;
pub use stock_id::StockId;
pub use movement_id::MovementId;
pub use reservation_id::ReservationId;
pub use recipe_id::RecipeId;
pub use ingredient_id::IngredientId;
pub use substitute_id::SubstituteId;
pub use adjustment_id::AdjustmentId;
pub use transfer_id::TransferId;
pub use category_id::CategoryId;

// Re-exports - Validated value objects
pub use sku::Sku;
pub use barcode::Barcode;
pub use currency::Currency;
pub use unit_of_measure::UnitOfMeasure;

// Re-exports - Enum value objects
pub use movement_type::MovementType;
pub use reservation_status::ReservationStatus;
pub use adjustment_status::AdjustmentStatus;
pub use adjustment_type::AdjustmentType;
pub use adjustment_reason::AdjustmentReason;
pub use transfer_status::TransferStatus;
