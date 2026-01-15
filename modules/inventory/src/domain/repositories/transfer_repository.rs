// TransferRepository trait - repository for stock transfer operations

use async_trait::async_trait;

use crate::domain::entities::StockTransfer;
use crate::domain::value_objects::TransferId;
use crate::InventoryError;
use identity::StoreId;

/// Repository trait for StockTransfer persistence operations.
/// Handles transfer documents with shipping workflow.
#[async_trait]
pub trait TransferRepository: Send + Sync {
    /// Saves a new transfer to the repository
    async fn save(&self, transfer: &StockTransfer) -> Result<(), InventoryError>;

    /// Finds a transfer by its unique ID (without items)
    async fn find_by_id(&self, id: TransferId) -> Result<Option<StockTransfer>, InventoryError>;

    /// Finds a transfer by its unique ID with all items loaded
    async fn find_by_id_with_items(&self, id: TransferId) -> Result<Option<StockTransfer>, InventoryError>;

    /// Finds all transfers for a specific store (as source or destination)
    /// Results are ordered by created_at DESC
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<StockTransfer>, InventoryError>;

    /// Finds all outgoing transfers from a specific store
    async fn find_outgoing_by_store(&self, store_id: StoreId) -> Result<Vec<StockTransfer>, InventoryError>;

    /// Finds all incoming transfers to a specific store
    async fn find_incoming_by_store(&self, store_id: StoreId) -> Result<Vec<StockTransfer>, InventoryError>;

    /// Updates an existing transfer
    async fn update(&self, transfer: &StockTransfer) -> Result<(), InventoryError>;

    /// Generates a unique transfer number globally
    /// Format: TRF-{YYYYMMDD}-{SEQUENCE}
    async fn generate_transfer_number(&self) -> Result<String, InventoryError>;
}
