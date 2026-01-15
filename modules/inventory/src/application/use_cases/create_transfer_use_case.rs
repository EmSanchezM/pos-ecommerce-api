// CreateTransferUseCase - creates a new stock transfer in draft status

use std::sync::Arc;

use crate::application::dtos::commands::{CreateTransferCommand, TransferItemCommand};
use crate::application::dtos::responses::{TransferDetailResponse, TransferItemResponse};
use crate::domain::entities::{StockTransfer, TransferItem};
use crate::domain::repositories::TransferRepository;
use crate::domain::value_objects::{ProductId, TransferId, VariantId};
use crate::InventoryError;
use identity::{StoreId, UserId};

/// Use case for creating a new stock transfer.
///
/// Validates from_store_id != to_store_id, generates transfer number,
/// sets status to draft, and adds items.
pub struct CreateTransferUseCase<T>
where
    T: TransferRepository,
{
    transfer_repo: Arc<T>,
}

impl<T> CreateTransferUseCase<T>
where
    T: TransferRepository,
{
    /// Creates a new instance of CreateTransferUseCase
    pub fn new(transfer_repo: Arc<T>) -> Self {
        Self { transfer_repo }
    }

    /// Executes the use case to create a new stock transfer
    ///
    /// # Arguments
    /// * `command` - The create transfer command containing transfer data
    /// * `actor_id` - ID of the user creating the transfer
    ///
    /// # Returns
    /// TransferDetailResponse on success
    ///
    /// # Errors
    /// * `InventoryError::SameStoreTransfer` - If from_store_id equals to_store_id
    /// * `InventoryError::EmptyTransfer` - If no items provided
    /// * `InventoryError::InvalidProductVariantConstraint` - If item has invalid product/variant
    pub async fn execute(
        &self,
        command: CreateTransferCommand,
        actor_id: UserId,
    ) -> Result<TransferDetailResponse, InventoryError> {
        // 1. Validate items are provided
        if command.items.is_empty() {
            return Err(InventoryError::EmptyTransfer);
        }

        // 2. Generate transfer number (Requirement 11.8)
        let transfer_number = self.transfer_repo.generate_transfer_number().await?;

        // 3. Create transfer entity (validates from_store_id != to_store_id) (Requirement 11.2)
        let from_store_id = StoreId::from_uuid(command.from_store_id);
        let to_store_id = StoreId::from_uuid(command.to_store_id);
        
        let mut transfer = StockTransfer::create(
            transfer_number,
            from_store_id,
            to_store_id,
            actor_id,
        )?;

        // 4. Set optional fields
        if let Some(notes) = command.notes {
            transfer.set_notes(Some(notes))?;
        }
        if let Some(shipping_method) = command.shipping_method {
            transfer.set_shipping_method(Some(shipping_method))?;
        }

        // 5. Add items to transfer
        for item_cmd in &command.items {
            let item = self.create_transfer_item(transfer.id(), item_cmd)?;
            transfer.add_item(item)?;
        }

        // 6. Save transfer
        self.transfer_repo.save(&transfer).await?;

        // 7. Convert to response
        Ok(self.to_response(&transfer))
    }

    fn create_transfer_item(
        &self,
        transfer_id: TransferId,
        cmd: &TransferItemCommand,
    ) -> Result<TransferItem, InventoryError> {
        // Validate XOR constraint: exactly one of product_id or variant_id must be set
        match (cmd.product_id, cmd.variant_id) {
            (Some(product_uuid), None) => {
                let product_id = ProductId::from_uuid(product_uuid);
                let mut item = TransferItem::create_for_product(
                    transfer_id,
                    product_id,
                    cmd.quantity_requested,
                    cmd.unit_cost,
                )?;
                if let Some(ref notes) = cmd.notes {
                    item.set_notes(Some(notes.clone()));
                }
                Ok(item)
            }
            (None, Some(variant_uuid)) => {
                let variant_id = VariantId::from_uuid(variant_uuid);
                let mut item = TransferItem::create_for_variant(
                    transfer_id,
                    variant_id,
                    cmd.quantity_requested,
                    cmd.unit_cost,
                )?;
                if let Some(ref notes) = cmd.notes {
                    item.set_notes(Some(notes.clone()));
                }
                Ok(item)
            }
            _ => Err(InventoryError::InvalidProductVariantConstraint),
        }
    }

    fn to_response(&self, transfer: &StockTransfer) -> TransferDetailResponse {
        let items: Vec<TransferItemResponse> = transfer
            .items()
            .iter()
            .map(|item| TransferItemResponse {
                id: item.id(),
                transfer_id: item.transfer_id().into_uuid(),
                product_id: item.product_id().map(|id| id.into_uuid()),
                variant_id: item.variant_id().map(|id| id.into_uuid()),
                product: None,
                variant: None,
                quantity_requested: item.quantity_requested(),
                quantity_shipped: item.quantity_shipped(),
                quantity_received: item.quantity_received(),
                unit_cost: item.unit_cost(),
                notes: item.notes().map(|s| s.to_string()),
                created_at: item.created_at(),
            })
            .collect();

        TransferDetailResponse {
            id: transfer.id().into_uuid(),
            transfer_number: transfer.transfer_number().to_string(),
            from_store_id: transfer.from_store_id().into_uuid(),
            to_store_id: transfer.to_store_id().into_uuid(),
            status: transfer.status().to_string(),
            requested_date: transfer.requested_date(),
            shipped_date: transfer.shipped_date(),
            received_date: transfer.received_date(),
            requested_by_id: transfer.requested_by_id().into_uuid(),
            shipped_by_id: transfer.shipped_by_id().map(|id| id.into_uuid()),
            received_by_id: transfer.received_by_id().map(|id| id.into_uuid()),
            notes: transfer.notes().map(|s| s.to_string()),
            shipping_method: transfer.shipping_method().map(|s| s.to_string()),
            tracking_number: transfer.tracking_number().map(|s| s.to_string()),
            items,
            created_at: transfer.created_at(),
            updated_at: transfer.updated_at(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    struct MockTransferRepository {
        transfers: Mutex<HashMap<TransferId, StockTransfer>>,
        counter: Mutex<i32>,
    }

    impl MockTransferRepository {
        fn new() -> Self {
            Self {
                transfers: Mutex::new(HashMap::new()),
                counter: Mutex::new(0),
            }
        }
    }

    #[async_trait]
    impl TransferRepository for MockTransferRepository {
        async fn save(&self, transfer: &StockTransfer) -> Result<(), InventoryError> {
            let mut transfers = self.transfers.lock().unwrap();
            transfers.insert(transfer.id(), transfer.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: TransferId,
        ) -> Result<Option<StockTransfer>, InventoryError> {
            let transfers = self.transfers.lock().unwrap();
            Ok(transfers.get(&id).cloned())
        }

        async fn find_by_id_with_items(
            &self,
            id: TransferId,
        ) -> Result<Option<StockTransfer>, InventoryError> {
            self.find_by_id(id).await
        }

        async fn find_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<StockTransfer>, InventoryError> {
            Ok(vec![])
        }

        async fn find_outgoing_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<StockTransfer>, InventoryError> {
            Ok(vec![])
        }

        async fn find_incoming_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<StockTransfer>, InventoryError> {
            Ok(vec![])
        }

        async fn update(&self, transfer: &StockTransfer) -> Result<(), InventoryError> {
            let mut transfers = self.transfers.lock().unwrap();
            transfers.insert(transfer.id(), transfer.clone());
            Ok(())
        }

        async fn generate_transfer_number(&self) -> Result<String, InventoryError> {
            let mut counter = self.counter.lock().unwrap();
            *counter += 1;
            Ok(format!("TRF-TEST-{:05}", *counter))
        }
    }

    #[tokio::test]
    async fn test_create_transfer_success() {
        let repo = Arc::new(MockTransferRepository::new());
        let use_case = CreateTransferUseCase::new(repo.clone());

        let from_store_id = new_uuid();
        let to_store_id = new_uuid();
        let product_id = new_uuid();

        let command = CreateTransferCommand {
            from_store_id,
            to_store_id,
            notes: Some("Test transfer".to_string()),
            shipping_method: Some("Ground".to_string()),
            items: vec![TransferItemCommand {
                product_id: Some(product_id),
                variant_id: None,
                quantity_requested: dec!(10),
                unit_cost: Some(dec!(5.00)),
                notes: Some("Item notes".to_string()),
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.transfer_number, "TRF-TEST-00001");
        assert_eq!(response.from_store_id, from_store_id);
        assert_eq!(response.to_store_id, to_store_id);
        assert_eq!(response.status, "draft");
        assert_eq!(response.notes, Some("Test transfer".to_string()));
        assert_eq!(response.shipping_method, Some("Ground".to_string()));
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].quantity_requested, dec!(10));
        assert_eq!(response.items[0].product_id, Some(product_id));
    }

    #[tokio::test]
    async fn test_create_transfer_multiple_items() {
        let repo = Arc::new(MockTransferRepository::new());
        let use_case = CreateTransferUseCase::new(repo);

        let command = CreateTransferCommand {
            from_store_id: new_uuid(),
            to_store_id: new_uuid(),
            notes: None,
            shipping_method: None,
            items: vec![
                TransferItemCommand {
                    product_id: Some(new_uuid()),
                    variant_id: None,
                    quantity_requested: dec!(5),
                    unit_cost: None,
                    notes: None,
                },
                TransferItemCommand {
                    product_id: None,
                    variant_id: Some(new_uuid()),
                    quantity_requested: dec!(10),
                    unit_cost: Some(dec!(15.00)),
                    notes: None,
                },
            ],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.items.len(), 2);
    }

    #[tokio::test]
    async fn test_create_transfer_same_store() {
        let repo = Arc::new(MockTransferRepository::new());
        let use_case = CreateTransferUseCase::new(repo);

        let store_id = new_uuid();

        let command = CreateTransferCommand {
            from_store_id: store_id,
            to_store_id: store_id, // Same store!
            notes: None,
            shipping_method: None,
            items: vec![TransferItemCommand {
                product_id: Some(new_uuid()),
                variant_id: None,
                quantity_requested: dec!(10),
                unit_cost: None,
                notes: None,
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::SameStoreTransfer)));
    }

    #[tokio::test]
    async fn test_create_transfer_empty_items() {
        let repo = Arc::new(MockTransferRepository::new());
        let use_case = CreateTransferUseCase::new(repo);

        let command = CreateTransferCommand {
            from_store_id: new_uuid(),
            to_store_id: new_uuid(),
            notes: None,
            shipping_method: None,
            items: vec![],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::EmptyTransfer)));
    }

    #[tokio::test]
    async fn test_create_transfer_invalid_item_both_ids() {
        let repo = Arc::new(MockTransferRepository::new());
        let use_case = CreateTransferUseCase::new(repo);

        let command = CreateTransferCommand {
            from_store_id: new_uuid(),
            to_store_id: new_uuid(),
            notes: None,
            shipping_method: None,
            items: vec![TransferItemCommand {
                product_id: Some(new_uuid()),
                variant_id: Some(new_uuid()), // Both set!
                quantity_requested: dec!(10),
                unit_cost: None,
                notes: None,
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidProductVariantConstraint)
        ));
    }

    #[tokio::test]
    async fn test_create_transfer_invalid_item_neither_id() {
        let repo = Arc::new(MockTransferRepository::new());
        let use_case = CreateTransferUseCase::new(repo);

        let command = CreateTransferCommand {
            from_store_id: new_uuid(),
            to_store_id: new_uuid(),
            notes: None,
            shipping_method: None,
            items: vec![TransferItemCommand {
                product_id: None,
                variant_id: None, // Neither set!
                quantity_requested: dec!(10),
                unit_cost: None,
                notes: None,
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidProductVariantConstraint)
        ));
    }
}
