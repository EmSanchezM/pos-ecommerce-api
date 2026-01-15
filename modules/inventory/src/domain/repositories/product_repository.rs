// ProductRepository trait - repository for product and variant operations

use async_trait::async_trait;

use crate::domain::entities::{Product, ProductVariant};
use crate::domain::value_objects::{Barcode, CategoryId, ProductId, Sku, VariantId};
use crate::InventoryError;

/// Repository trait for Product persistence operations.
/// Handles both products and their variants.
#[async_trait]
pub trait ProductRepository: Send + Sync {
    /// Saves a new product to the repository
    async fn save(&self, product: &Product) -> Result<(), InventoryError>;

    /// Finds a product by its unique ID
    async fn find_by_id(&self, id: ProductId) -> Result<Option<Product>, InventoryError>;

    /// Finds a product by its unique SKU
    async fn find_by_sku(&self, sku: &Sku) -> Result<Option<Product>, InventoryError>;

    /// Finds a product by its barcode
    async fn find_by_barcode(&self, barcode: &Barcode) -> Result<Option<Product>, InventoryError>;

    /// Updates an existing product
    async fn update(&self, product: &Product) -> Result<(), InventoryError>;

    /// Deletes a product by ID
    /// This will cascade delete all associated variants
    async fn delete(&self, id: ProductId) -> Result<(), InventoryError>;

    /// Finds all active products
    async fn find_active(&self) -> Result<Vec<Product>, InventoryError>;

    /// Finds all products in a specific category
    async fn find_by_category(&self, category_id: CategoryId) -> Result<Vec<Product>, InventoryError>;

    // =========================================================================
    // Variant operations
    // =========================================================================

    /// Saves a new product variant
    async fn save_variant(&self, variant: &ProductVariant) -> Result<(), InventoryError>;

    /// Finds a variant by its unique ID
    async fn find_variant_by_id(&self, id: VariantId) -> Result<Option<ProductVariant>, InventoryError>;

    /// Finds a variant by its unique SKU
    async fn find_variant_by_sku(&self, sku: &Sku) -> Result<Option<ProductVariant>, InventoryError>;

    /// Finds a variant by its barcode
    async fn find_variant_by_barcode(&self, barcode: &Barcode) -> Result<Option<ProductVariant>, InventoryError>;

    /// Finds all variants for a product
    async fn find_variants_by_product(&self, product_id: ProductId) -> Result<Vec<ProductVariant>, InventoryError>;

    /// Updates an existing variant
    async fn update_variant(&self, variant: &ProductVariant) -> Result<(), InventoryError>;

    /// Deletes a variant by ID
    async fn delete_variant(&self, id: VariantId) -> Result<(), InventoryError>;

    /// Counts the number of variants for a product (used for SKU generation)
    async fn count_variants(&self, product_id: ProductId) -> Result<u32, InventoryError>;
}
