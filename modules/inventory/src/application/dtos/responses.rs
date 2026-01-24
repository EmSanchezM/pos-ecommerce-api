// Response DTOs for inventory operations
//
// These DTOs represent the output data returned from use cases in the inventory module.
// They are designed for API responses and include all necessary information for clients.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

// =============================================================================
// Category Responses
// =============================================================================

/// Response for a single category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for hierarchical category tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeResponse {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub children: Vec<CategoryTreeResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Product Responses
// =============================================================================

/// Response for a single product (summary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub brand: Option<String>,
    pub unit_of_measure: String,
    pub base_price: Decimal,
    pub cost_price: Decimal,
    pub currency: String,
    pub is_perishable: bool,
    pub is_trackable: bool,
    pub has_variants: bool,
    pub tax_rate: Decimal,
    pub tax_included: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for product with full details including variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDetailResponse {
    pub id: Uuid,
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category: Option<CategoryResponse>,
    pub brand: Option<String>,
    pub unit_of_measure: String,
    pub base_price: Decimal,
    pub cost_price: Decimal,
    pub currency: String,
    pub is_perishable: bool,
    pub is_trackable: bool,
    pub has_variants: bool,
    pub tax_rate: Decimal,
    pub tax_included: bool,
    pub attributes: JsonValue,
    pub is_active: bool,
    pub variants: Vec<VariantResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Variant Responses
// =============================================================================

/// Response for a product variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub variant_attributes: JsonValue,
    pub price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub effective_price: Decimal,
    pub effective_cost: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Stock Responses
// =============================================================================

/// Response for stock summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    pub available_quantity: Decimal,
    pub version: i32,
    pub min_stock_level: Decimal,
    pub max_stock_level: Option<Decimal>,
    pub is_low_stock: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for stock with full details including product/variant info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockDetailResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub product: Option<ProductResponse>,
    pub variant: Option<VariantResponse>,
    pub quantity: Decimal,
    pub reserved_quantity: Decimal,
    pub available_quantity: Decimal,
    pub version: i32,
    pub min_stock_level: Decimal,
    pub max_stock_level: Option<Decimal>,
    pub is_low_stock: bool,
    pub weighted_average_cost: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Reservation Responses
// =============================================================================

/// Response for an inventory reservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationResponse {
    pub id: Uuid,
    pub stock_id: Uuid,
    pub reference_type: String,
    pub reference_id: Uuid,
    pub quantity: Decimal,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Movement Responses
// =============================================================================

/// Response for a single inventory movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementResponse {
    pub id: Uuid,
    pub stock_id: Uuid,
    pub movement_type: String,
    pub movement_reason: Option<String>,
    pub quantity: Decimal,
    pub unit_cost: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub currency: String,
    pub balance_after: Decimal,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub actor_id: Uuid,
    pub notes: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}

/// Response for Kardex report (movement history)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KardexResponse {
    pub stock_id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub product: Option<ProductResponse>,
    pub variant: Option<VariantResponse>,
    pub current_quantity: Decimal,
    pub current_reserved: Decimal,
    pub current_available: Decimal,
    pub weighted_average_cost: Option<Decimal>,
    pub movements: Vec<MovementResponse>,
    pub total_movements: i64,
}

// =============================================================================
// Recipe Responses
// =============================================================================

/// Response for a recipe summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeResponse {
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub yield_quantity: Decimal,
    pub is_active: bool,
    pub preparation_time_minutes: Option<i32>,
    pub calculate_cost_from_ingredients: bool,
    pub calculated_cost: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for recipe with full details including ingredients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDetailResponse {
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub product: Option<ProductResponse>,
    pub variant: Option<VariantResponse>,
    pub name: String,
    pub description: Option<String>,
    pub version: i32,
    pub yield_quantity: Decimal,
    pub is_active: bool,
    pub preparation_time_minutes: Option<i32>,
    pub calculate_cost_from_ingredients: bool,
    pub calculated_cost: Option<Decimal>,
    pub notes: Option<String>,
    pub metadata: Option<JsonValue>,
    pub ingredients: Vec<RecipeIngredientResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for a recipe ingredient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredientResponse {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient_product_id: Option<Uuid>,
    pub ingredient_variant_id: Option<Uuid>,
    pub ingredient_product: Option<ProductResponse>,
    pub ingredient_variant: Option<VariantResponse>,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    pub is_optional: bool,
    pub can_substitute: bool,
    pub sort_order: i32,
    pub preparation_step: Option<String>,
    pub estimated_cost_per_unit: Option<Decimal>,
    pub estimated_waste_percentage: Decimal,
    pub notes: Option<String>,
    pub substitutes: Vec<IngredientSubstituteResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for an ingredient substitute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientSubstituteResponse {
    pub id: Uuid,
    pub recipe_ingredient_id: Uuid,
    pub substitute_product_id: Option<Uuid>,
    pub substitute_variant_id: Option<Uuid>,
    pub substitute_product: Option<ProductResponse>,
    pub substitute_variant: Option<VariantResponse>,
    pub conversion_ratio: Decimal,
    pub priority: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Adjustment Responses
// =============================================================================

/// Response for an adjustment summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub adjustment_number: String,
    pub adjustment_type: String,
    pub adjustment_reason: String,
    pub status: String,
    pub created_by_id: Uuid,
    pub approved_by_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for adjustment with full details including items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentDetailResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub adjustment_number: String,
    pub adjustment_type: String,
    pub adjustment_reason: String,
    pub status: String,
    pub created_by_id: Uuid,
    pub approved_by_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub attachments: Option<JsonValue>,
    pub items: Vec<AdjustmentItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for an adjustment item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentItemResponse {
    pub id: Uuid,
    pub adjustment_id: Uuid,
    pub stock_id: Uuid,
    pub stock: Option<StockResponse>,
    pub quantity: Decimal,
    pub unit_cost: Option<Decimal>,
    pub balance_before: Option<Decimal>,
    pub balance_after: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Transfer Responses
// =============================================================================

/// Response for a transfer summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub id: Uuid,
    pub transfer_number: String,
    pub from_store_id: Uuid,
    pub to_store_id: Uuid,
    pub status: String,
    pub requested_date: DateTime<Utc>,
    pub shipped_date: Option<DateTime<Utc>>,
    pub received_date: Option<DateTime<Utc>>,
    pub requested_by_id: Uuid,
    pub shipped_by_id: Option<Uuid>,
    pub received_by_id: Option<Uuid>,
    pub notes: Option<String>,
    pub shipping_method: Option<String>,
    pub tracking_number: Option<String>,
    pub item_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for transfer with full details including items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDetailResponse {
    pub id: Uuid,
    pub transfer_number: String,
    pub from_store_id: Uuid,
    pub to_store_id: Uuid,
    pub status: String,
    pub requested_date: DateTime<Utc>,
    pub shipped_date: Option<DateTime<Utc>>,
    pub received_date: Option<DateTime<Utc>>,
    pub requested_by_id: Uuid,
    pub shipped_by_id: Option<Uuid>,
    pub received_by_id: Option<Uuid>,
    pub notes: Option<String>,
    pub shipping_method: Option<String>,
    pub tracking_number: Option<String>,
    pub items: Vec<TransferItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for a transfer item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItemResponse {
    pub id: Uuid,
    pub transfer_id: Uuid,
    pub product_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub product: Option<ProductResponse>,
    pub variant: Option<VariantResponse>,
    pub quantity_requested: Decimal,
    pub quantity_shipped: Option<Decimal>,
    pub quantity_received: Option<Decimal>,
    pub unit_cost: Option<Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// List Response (Simple)
// =============================================================================

/// Generic list response wrapper for endpoints that don't need full pagination
/// Use this for collections that are typically small (e.g., product variants)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    /// The data items
    pub data: Vec<T>,
    /// Total number of items
    pub total: i64,
}

impl<T> ListResponse<T> {
    pub fn new(data: Vec<T>) -> Self {
        let total = data.len() as i64;
        Self { data, total }
    }
}

// =============================================================================
// Pagination Response
// =============================================================================

/// Generic paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The data items for this page
    pub data: Vec<T>,
    /// Current page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
    /// Total number of items across all pages
    pub total_items: i64,
    /// Total number of pages
    pub total_pages: i64,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_previous: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: i64, page_size: i64, total_items: i64) -> Self {
        let total_pages = if total_items == 0 {
            1
        } else {
            (total_items + page_size - 1) / page_size
        };
        
        Self {
            data,
            page,
            page_size,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_previous: page > 1,
        }
    }
}
