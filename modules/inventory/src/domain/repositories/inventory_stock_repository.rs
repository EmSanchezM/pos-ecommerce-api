// InventoryStockRepository trait - repository for stock operations with optimistic locking

use async_trait::async_trait;

use crate::domain::entities::InventoryStock;
use crate::domain::value_objects::{ProductId, StockId, VariantId};
use crate::InventoryError;
use identity::StoreId;

/// Repository trait for InventoryStock persistence operations.
/// Supports optimistic locking via version field to prevent concurrent update conflicts.
#[async_trait]
pub trait InventoryStockRepository: Send + Sync {
    /// Saves a new stock record to the repository
    async fn save(&self, stock: &InventoryStock) -> Result<(), InventoryError>;

    /// Finds a stock record by its unique ID
    async fn find_by_id(&self, id: StockId) -> Result<Option<InventoryStock>, InventoryError>;

    /// Finds a stock record by store and product
    async fn find_by_store_and_product(
        &self,
        store_id: StoreId,
        product_id: ProductId,
    ) -> Result<Option<InventoryStock>, InventoryError>;

    /// Finds a stock record by store and variant
    async fn find_by_store_and_variant(
        &self,
        store_id: StoreId,
        variant_id: VariantId,
    ) -> Result<Option<InventoryStock>, InventoryError>;

    /// Updates a stock record with optimistic locking.
    /// Returns OptimisticLockError if the expected_version doesn't match the current version.
    /// 
    /// # Arguments
    /// * `stock` - The stock record with updated values
    /// * `expected_version` - The version number expected in the database
    /// 
    /// # Errors
    /// * `InventoryError::OptimisticLockError` - If version mismatch (concurrent modification)
    async fn update_with_version(
        &self,
        stock: &InventoryStock,
        expected_version: i32,
    ) -> Result<(), InventoryError>;

    /// Finds all stock records with low stock (available_quantity <= min_stock_level)
    async fn find_low_stock(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError>;

    /// Finds all stock records for a specific store
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError>;

    /// Finds stock records with pagination and optional filters
    /// Returns (stocks, total_count)
    async fn find_paginated(
        &self,
        store_id: Option<StoreId>,
        product_id: Option<ProductId>,
        low_stock_only: bool,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<InventoryStock>, i64), InventoryError>;

    /// Finds all stock records for a specific product across all stores
    async fn find_by_product(&self, product_id: ProductId) -> Result<Vec<InventoryStock>, InventoryError>;

    /// Finds all stock records across all stores
    async fn find_all(&self) -> Result<Vec<InventoryStock>, InventoryError>;

    /// Finds all stock records with low stock across all stores
    async fn find_all_low_stock(&self) -> Result<Vec<InventoryStock>, InventoryError>;

    /// Finds all stock records with low stock for a specific store
    async fn find_low_stock_by_store(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError>;
}
