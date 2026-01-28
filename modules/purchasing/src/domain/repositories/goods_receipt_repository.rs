// GoodsReceiptRepository trait - repository for goods receipt operations

use async_trait::async_trait;

use crate::domain::entities::GoodsReceipt;
use crate::domain::value_objects::{GoodsReceiptId, GoodsReceiptStatus, PurchaseOrderId};
use crate::PurchasingError;
use identity::StoreId;

/// Filter options for listing goods receipts
#[derive(Debug, Clone, Default)]
pub struct GoodsReceiptFilter {
    /// Filter by store ID
    pub store_id: Option<StoreId>,
    /// Filter by purchase order ID
    pub purchase_order_id: Option<PurchaseOrderId>,
    /// Filter by status
    pub status: Option<GoodsReceiptStatus>,
}

/// Repository trait for GoodsReceipt persistence operations.
#[async_trait]
pub trait GoodsReceiptRepository: Send + Sync {
    /// Saves a new goods receipt to the repository
    async fn save(&self, receipt: &GoodsReceipt) -> Result<(), PurchasingError>;

    /// Finds a goods receipt by its unique ID (without items)
    async fn find_by_id(
        &self,
        id: GoodsReceiptId,
    ) -> Result<Option<GoodsReceipt>, PurchasingError>;

    /// Finds a goods receipt by its unique ID with all items loaded
    async fn find_by_id_with_items(
        &self,
        id: GoodsReceiptId,
    ) -> Result<Option<GoodsReceipt>, PurchasingError>;

    /// Updates an existing goods receipt
    async fn update(&self, receipt: &GoodsReceipt) -> Result<(), PurchasingError>;

    /// Finds all goods receipts for a purchase order
    async fn find_by_purchase_order(
        &self,
        order_id: PurchaseOrderId,
    ) -> Result<Vec<GoodsReceipt>, PurchasingError>;

    /// Finds goods receipts with pagination and filters
    ///
    /// # Arguments
    /// * `filter` - Filter options
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Number of items per page
    ///
    /// # Returns
    /// Tuple of (receipts, total_count) for pagination
    async fn find_paginated(
        &self,
        filter: GoodsReceiptFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<GoodsReceipt>, i64), PurchasingError>;

    /// Generates a unique receipt number for a store
    /// Format: GR-{YEAR}-{SEQUENCE}
    async fn generate_receipt_number(&self, store_id: StoreId) -> Result<String, PurchasingError>;
}
