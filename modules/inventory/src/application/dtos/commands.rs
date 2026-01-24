// Command DTOs for inventory operations
//
// These DTOs represent the input data for various operations in the inventory module.
// They use primitive types (String, Uuid, Decimal) rather than domain value objects
// to keep the application boundary clean and allow validation in use cases.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

// =============================================================================
// Category Commands
// =============================================================================

/// Command to create a new product category
/// Requirements: 1A.1, 1A.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryCommand {
    /// Category name
    pub name: String,
    /// URL-friendly slug (must be unique)
    pub slug: String,
    /// Optional parent category ID for hierarchical structure
    pub parent_id: Option<Uuid>,
    /// Optional description
    pub description: Option<String>,
    /// Optional icon identifier
    pub icon: Option<String>,
    /// Sort order within the same level (default: 0)
    #[serde(default)]
    pub sort_order: i32,
}

/// Command to update an existing category
/// Requirements: 1A.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryCommand {
    /// New name (if changing)
    pub name: Option<String>,
    /// New slug (if changing, must be unique)
    pub slug: Option<String>,
    /// New parent ID (if changing)
    pub parent_id: Option<Uuid>,
    /// New description (if changing)
    pub description: Option<String>,
    /// New icon (if changing)
    pub icon: Option<String>,
    /// New sort order (if changing)
    pub sort_order: Option<i32>,
    /// New active status (if changing)
    pub is_active: Option<bool>,
}

// =============================================================================
// Product Commands
// =============================================================================

/// Command to create a new product
/// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductCommand {
    /// Product name (required)
    pub name: String,
    /// Unit of measure (required): "unit", "kg", "lb", "liter", "oz"
    pub unit_of_measure: String,
    /// Optional barcode (must be unique if provided)
    pub barcode: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Optional category ID
    pub category_id: Option<Uuid>,
    /// Optional brand name
    pub brand: Option<String>,
    /// Base selling price
    #[serde(default)]
    pub base_price: Decimal,
    /// Cost price
    #[serde(default)]
    pub cost_price: Decimal,
    /// Currency code (ISO 4217, default: "HNL")
    pub currency: Option<String>,
    /// Whether product is perishable
    #[serde(default)]
    pub is_perishable: bool,
    /// Whether inventory is tracked
    #[serde(default = "default_true")]
    pub is_trackable: bool,
    /// Whether product has variants
    #[serde(default)]
    pub has_variants: bool,
    /// Tax rate (e.g., 0.15 for 15%)
    #[serde(default)]
    pub tax_rate: Decimal,
    /// Whether tax is included in price
    #[serde(default)]
    pub tax_included: bool,
    /// Flexible product attributes (JSONB)
    pub attributes: Option<JsonValue>,
}

fn default_true() -> bool {
    true
}

/// Command to update an existing product
/// Requirements: 1.1, 1.9
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductCommand {
    /// New name (if changing)
    pub name: Option<String>,
    /// New barcode (if changing)
    pub barcode: Option<String>,
    /// New description (if changing)
    pub description: Option<String>,
    /// New category ID (if changing)
    pub category_id: Option<Uuid>,
    /// New brand (if changing)
    pub brand: Option<String>,
    /// New unit of measure (if changing)
    pub unit_of_measure: Option<String>,
    /// New base price (if changing)
    pub base_price: Option<Decimal>,
    /// New cost price (if changing)
    pub cost_price: Option<Decimal>,
    /// New currency (if changing)
    pub currency: Option<String>,
    /// New perishable flag (if changing)
    pub is_perishable: Option<bool>,
    /// New trackable flag (if changing)
    pub is_trackable: Option<bool>,
    /// New has_variants flag (if changing)
    pub has_variants: Option<bool>,
    /// New tax rate (if changing)
    pub tax_rate: Option<Decimal>,
    /// New tax_included flag (if changing)
    pub tax_included: Option<bool>,
    /// New attributes (if changing)
    pub attributes: Option<JsonValue>,
    /// New active status (if changing)
    pub is_active: Option<bool>,
}

// =============================================================================
// Variant Commands
// =============================================================================

/// Command to create a product variant
/// Requirements: 2.1, 2.2, 2.3, 2.4, 2.6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVariantCommand {
    /// Parent product ID
    pub product_id: Uuid,
    /// Variant name (e.g., "Large", "Red", "Vanilla")
    pub name: String,
    /// Variant-specific attributes (JSONB)
    pub variant_attributes: JsonValue,
    /// Optional price override (if different from product base_price)
    pub price: Option<Decimal>,
    /// Optional cost override (if different from product cost_price)
    pub cost_price: Option<Decimal>,
    /// Optional unique barcode for this variant
    pub barcode: Option<String>,
}

/// Command to update a product variant
/// Requirements: 2.1, 2.6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVariantCommand {
    /// New name (if changing)
    pub name: Option<String>,
    /// New variant attributes (if changing)
    pub variant_attributes: Option<JsonValue>,
    /// New price override (if changing)
    pub price: Option<Decimal>,
    /// New cost override (if changing)
    pub cost_price: Option<Decimal>,
    /// New barcode (if changing)
    pub barcode: Option<String>,
    /// New active status (if changing)
    pub is_active: Option<bool>,
}

// =============================================================================
// Stock Commands
// =============================================================================

/// Command to initialize stock for a product or variant in a store
/// Creates a new inventory_stock record associating the product/variant with a store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeStockCommand {
    /// Store ID where stock will be tracked
    pub store_id: Uuid,
    /// Product ID (mutually exclusive with variant_id)
    pub product_id: Option<Uuid>,
    /// Variant ID (mutually exclusive with product_id)
    pub variant_id: Option<Uuid>,
    /// Initial quantity (defaults to 0)
    #[serde(default)]
    pub initial_quantity: Decimal,
    /// Minimum stock level for low stock alerts
    #[serde(default)]
    pub min_stock_level: Decimal,
    /// Maximum stock level (optional)
    pub max_stock_level: Option<Decimal>,
}

/// Command to update stock quantity
/// Requirements: 3.3, 3.4, 5.1, 5.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStockCommand {
    /// Stock record ID
    pub stock_id: Uuid,
    /// Quantity change (positive for increase, negative for decrease)
    pub quantity_delta: Decimal,
    /// Movement type: "in", "out", "adjustment", "transfer_out", "transfer_in", "reservation", "release"
    pub movement_type: String,
    /// Optional reason for the movement
    pub movement_reason: Option<String>,
    /// Optional unit cost for cost tracking
    pub unit_cost: Option<Decimal>,
    /// Optional reference type (e.g., "order", "adjustment", "transfer")
    pub reference_type: Option<String>,
    /// Optional reference ID linking to source document
    pub reference_id: Option<Uuid>,
    /// Optional notes
    pub notes: Option<String>,
    /// Expected version for optimistic locking
    pub expected_version: i32,
}

// =============================================================================
// Reservation Commands
// =============================================================================

/// Command to create an inventory reservation
/// Requirements: 4.1, 4.2, 4.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReservationCommand {
    /// Stock record ID to reserve from
    pub stock_id: Uuid,
    /// Reference type (e.g., "cart", "order", "quote")
    pub reference_type: String,
    /// Reference ID linking to the source entity
    pub reference_id: Uuid,
    /// Quantity to reserve
    pub quantity: Decimal,
    /// Expiration timestamp (must be in future)
    pub expires_at: DateTime<Utc>,
}

/// Command to confirm a reservation
/// Requirements: 4.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmReservationCommand {
    /// Reservation ID to confirm
    pub reservation_id: Uuid,
}

/// Command to cancel a reservation
/// Requirements: 4.6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelReservationCommand {
    /// Reservation ID to cancel
    pub reservation_id: Uuid,
}

// =============================================================================
// Recipe Commands
// =============================================================================

/// Command to create a recipe
/// Requirements: 6.1, 6.2, 6.3, 6.4, 6.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipeCommand {
    /// Product ID this recipe produces (mutually exclusive with variant_id)
    pub product_id: Option<Uuid>,
    /// Variant ID this recipe produces (mutually exclusive with product_id)
    pub variant_id: Option<Uuid>,
    /// Recipe name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Recipe version number
    #[serde(default = "default_version")]
    pub version: i32,
    /// Quantity produced by this recipe
    pub yield_quantity: Decimal,
    /// Preparation time in minutes
    pub preparation_time_minutes: Option<i32>,
    /// Whether to calculate cost from ingredients
    #[serde(default)]
    pub calculate_cost_from_ingredients: bool,
    /// Optional notes
    pub notes: Option<String>,
    /// Optional metadata (JSONB)
    pub metadata: Option<JsonValue>,
    /// Recipe ingredients
    pub ingredients: Vec<RecipeIngredientCommand>,
}

fn default_version() -> i32 {
    1
}

/// Command to update a recipe
/// Requirements: 6.1, 6.6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecipeCommand {
    /// New name (if changing)
    pub name: Option<String>,
    /// New description (if changing)
    pub description: Option<String>,
    /// New version (if changing)
    pub version: Option<i32>,
    /// New yield quantity (if changing)
    pub yield_quantity: Option<Decimal>,
    /// New preparation time (if changing)
    pub preparation_time_minutes: Option<i32>,
    /// New calculate_cost flag (if changing)
    pub calculate_cost_from_ingredients: Option<bool>,
    /// New notes (if changing)
    pub notes: Option<String>,
    /// New metadata (if changing)
    pub metadata: Option<JsonValue>,
    /// New active status (if changing)
    pub is_active: Option<bool>,
}

/// Recipe ingredient within CreateRecipeCommand
/// Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredientCommand {
    /// Ingredient product ID (mutually exclusive with ingredient_variant_id)
    pub ingredient_product_id: Option<Uuid>,
    /// Ingredient variant ID (mutually exclusive with ingredient_product_id)
    pub ingredient_variant_id: Option<Uuid>,
    /// Quantity required
    pub quantity: Decimal,
    /// Unit of measure: "unit", "kg", "lb", "liter", "oz"
    pub unit_of_measure: String,
    /// Whether ingredient is optional
    #[serde(default)]
    pub is_optional: bool,
    /// Whether substitutes are allowed
    #[serde(default)]
    pub can_substitute: bool,
    /// Sort order for preparation sequence
    #[serde(default)]
    pub sort_order: i32,
    /// Optional preparation step description
    pub preparation_step: Option<String>,
    /// Optional estimated cost per unit
    pub estimated_cost_per_unit: Option<Decimal>,
    /// Optional estimated waste percentage (e.g., 0.10 for 10%)
    #[serde(default)]
    pub estimated_waste_percentage: Decimal,
    /// Optional notes
    pub notes: Option<String>,
    /// Optional substitutes
    #[serde(default)]
    pub substitutes: Vec<IngredientSubstituteCommand>,
}

/// Ingredient substitute within RecipeIngredientCommand
/// Requirements: 8.1, 8.2, 8.3, 8.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientSubstituteCommand {
    /// Substitute product ID (mutually exclusive with substitute_variant_id)
    pub substitute_product_id: Option<Uuid>,
    /// Substitute variant ID (mutually exclusive with substitute_product_id)
    pub substitute_variant_id: Option<Uuid>,
    /// Conversion ratio (e.g., 1.5 means use 1.5x quantity of substitute)
    pub conversion_ratio: Decimal,
    /// Priority (lower number = higher preference)
    #[serde(default)]
    pub priority: i32,
    /// Optional notes
    pub notes: Option<String>,
}

// =============================================================================
// Adjustment Commands
// =============================================================================

/// Command to create a stock adjustment
/// Requirements: 9.1, 9.2, 9.3, 9.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdjustmentCommand {
    /// Store ID where adjustment occurs
    pub store_id: Uuid,
    /// Adjustment type: "increase" or "decrease"
    pub adjustment_type: String,
    /// Adjustment reason: "damage", "theft", "loss", "found", "correction", "expiration"
    pub adjustment_reason: String,
    /// Optional notes
    pub notes: Option<String>,
    /// Optional attachments (JSONB)
    pub attachments: Option<JsonValue>,
    /// Adjustment items
    pub items: Vec<AdjustmentItemCommand>,
}

/// Adjustment item within CreateAdjustmentCommand
/// Requirements: 10.1, 10.2, 10.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentItemCommand {
    /// Stock record ID
    pub stock_id: Uuid,
    /// Quantity (positive for increase, negative for decrease)
    pub quantity: Decimal,
    /// Optional unit cost for cost impact tracking
    pub unit_cost: Option<Decimal>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Command to submit adjustment for approval
/// Requirements: 9.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAdjustmentCommand {
    /// Adjustment ID to submit
    pub adjustment_id: Uuid,
}

/// Command to approve or reject an adjustment
/// Requirements: 9.6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveAdjustmentCommand {
    /// Adjustment ID to approve/reject
    pub adjustment_id: Uuid,
    /// Whether to approve (true) or reject (false)
    pub approve: bool,
    /// Optional notes for approval/rejection
    pub notes: Option<String>,
}

/// Command to apply an approved adjustment
/// Requirements: 9.7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyAdjustmentCommand {
    /// Adjustment ID to apply
    pub adjustment_id: Uuid,
}

// =============================================================================
// Transfer Commands
// =============================================================================

/// Command to create a stock transfer
/// Requirements: 11.1, 11.2, 11.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransferCommand {
    /// Source store ID
    pub from_store_id: Uuid,
    /// Destination store ID (must be different from from_store_id)
    pub to_store_id: Uuid,
    /// Optional notes
    pub notes: Option<String>,
    /// Optional shipping method
    pub shipping_method: Option<String>,
    /// Transfer items
    pub items: Vec<TransferItemCommand>,
}

/// Transfer item within CreateTransferCommand
/// Requirements: 12.1, 12.2, 12.3, 12.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItemCommand {
    /// Product ID to transfer (mutually exclusive with variant_id)
    pub product_id: Option<Uuid>,
    /// Variant ID to transfer (mutually exclusive with product_id)
    pub variant_id: Option<Uuid>,
    /// Quantity requested
    pub quantity_requested: Decimal,
    /// Optional unit cost for transfer valuation
    pub unit_cost: Option<Decimal>,
    /// Optional notes
    pub notes: Option<String>,
}

/// Command to ship a transfer
/// Requirements: 11.4, 11.5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipTransferCommand {
    /// Transfer ID to ship
    pub transfer_id: Uuid,
    /// Tracking number (if available)
    pub tracking_number: Option<String>,
    /// Items with actual shipped quantities
    pub items: Vec<ShipTransferItemCommand>,
}

/// Shipped item quantities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipTransferItemCommand {
    /// Transfer item ID
    pub item_id: Uuid,
    /// Actual quantity shipped (may differ from requested)
    pub quantity_shipped: Decimal,
}

/// Command to receive a transfer
/// Requirements: 11.6, 11.7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveTransferCommand {
    /// Transfer ID to receive
    pub transfer_id: Uuid,
    /// Items with actual received quantities
    pub items: Vec<ReceiveTransferItemCommand>,
}

/// Received item quantities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveTransferItemCommand {
    /// Transfer item ID
    pub item_id: Uuid,
    /// Actual quantity received (may differ from shipped due to damage/loss)
    pub quantity_received: Decimal,
}
