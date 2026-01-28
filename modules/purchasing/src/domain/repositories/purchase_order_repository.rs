// PurchaseOrderRepository trait - repository for purchase order operations

use async_trait::async_trait;

use crate::domain::entities::{PurchaseOrder, PurchaseOrderItem};
use crate::domain::value_objects::{PurchaseOrderId, PurchaseOrderItemId, PurchaseOrderStatus, VendorId};
use crate::PurchasingError;
use identity::StoreId;

/// Filter options for listing purchase orders
#[derive(Debug, Clone, Default)]
pub struct PurchaseOrderFilter {
    /// Filter by store ID
    pub store_id: Option<StoreId>,
    /// Filter by vendor ID
    pub vendor_id: Option<VendorId>,
    /// Filter by status
    pub status: Option<PurchaseOrderStatus>,
    /// Search by order number
    pub search: Option<String>,
}

/// Repository trait for PurchaseOrder persistence operations.
#[async_trait]
pub trait PurchaseOrderRepository: Send + Sync {
    /// Saves a new purchase order to the repository
    async fn save(&self, order: &PurchaseOrder) -> Result<(), PurchasingError>;

    /// Finds a purchase order by its unique ID (without items)
    async fn find_by_id(
        &self,
        id: PurchaseOrderId,
    ) -> Result<Option<PurchaseOrder>, PurchasingError>;

    /// Finds a purchase order by its unique ID with all items loaded
    async fn find_by_id_with_items(
        &self,
        id: PurchaseOrderId,
    ) -> Result<Option<PurchaseOrder>, PurchasingError>;

    /// Finds a purchase order by order number within a store
    async fn find_by_order_number(
        &self,
        store_id: StoreId,
        order_number: &str,
    ) -> Result<Option<PurchaseOrder>, PurchasingError>;

    /// Updates an existing purchase order
    async fn update(&self, order: &PurchaseOrder) -> Result<(), PurchasingError>;

    /// Finds purchase orders with pagination and filters
    ///
    /// # Arguments
    /// * `filter` - Filter options
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Number of items per page
    ///
    /// # Returns
    /// Tuple of (orders, total_count) for pagination
    async fn find_paginated(
        &self,
        filter: PurchaseOrderFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<PurchaseOrder>, i64), PurchasingError>;

    /// Generates a unique order number for a store
    /// Format: PO-{YEAR}-{SEQUENCE}
    async fn generate_order_number(&self, store_id: StoreId) -> Result<String, PurchasingError>;

    // -------------------------------------------------------------------------
    // Item operations
    // -------------------------------------------------------------------------

    /// Saves a new purchase order item
    async fn save_item(&self, item: &PurchaseOrderItem) -> Result<(), PurchasingError>;

    /// Updates an existing purchase order item
    async fn update_item(&self, item: &PurchaseOrderItem) -> Result<(), PurchasingError>;

    /// Deletes a purchase order item
    async fn delete_item(&self, item_id: PurchaseOrderItemId) -> Result<(), PurchasingError>;

    /// Finds all items for a purchase order
    async fn find_items_by_order(
        &self,
        order_id: PurchaseOrderId,
    ) -> Result<Vec<PurchaseOrderItem>, PurchasingError>;

    /// Finds a single item by ID
    async fn find_item_by_id(
        &self,
        item_id: PurchaseOrderItemId,
    ) -> Result<Option<PurchaseOrderItem>, PurchasingError>;
}
