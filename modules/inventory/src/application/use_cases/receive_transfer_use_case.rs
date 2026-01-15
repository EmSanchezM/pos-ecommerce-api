// ReceiveTransferUseCase - receives a transfer at destination, increasing destination stock

use std::sync::Arc;

use crate::application::dtos::commands::ReceiveTransferCommand;
use crate::application::dtos::responses::{TransferDetailResponse, TransferItemResponse};
use crate::domain::entities::{InventoryMovement, InventoryStock, StockTransfer};
use crate::domain::repositories::{InventoryMovementRepository, InventoryStockRepository, TransferRepository};
use crate::domain::value_objects::{Currency, MovementType, TransferId};
use crate::InventoryError;
use identity::UserId;

/// Use case for receiving a stock transfer at destination.
///
/// Validates status is in_transit, records receiver and timestamp,
/// increases destination stock for each item (creating stock records if needed),
/// creates transfer_in movements, and changes status to completed.
pub struct ReceiveTransferUseCase<T, S, M>
where
    T: TransferRepository,
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
{
    transfer_repo: Arc<T>,
    stock_repo: Arc<S>,
    movement_repo: Arc<M>,
}

impl<T, S, M> ReceiveTransferUseCase<T, S, M>
where
    T: TransferRepository,
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
{
    /// Creates a new instance of ReceiveTransferUseCase
    pub fn new(transfer_repo: Arc<T>, stock_repo: Arc<S>, movement_repo: Arc<M>) -> Self {
        Self {
            transfer_repo,
            stock_repo,
            movement_repo,
        }
    }

    /// Executes the use case to receive a transfer
    ///
    /// # Arguments
    /// * `command` - The receive transfer command containing transfer ID and received quantities
    /// * `actor_id` - ID of the user receiving the transfer
    ///
    /// # Returns
    /// TransferDetailResponse on success
    ///
    /// # Errors
    /// * `InventoryError::TransferNotFound` - If transfer doesn't exist
    /// * `InventoryError::InvalidStatusTransition` - If transfer is not in in_transit status
    /// * `InventoryError::OptimisticLockError` - If concurrent modification detected
    pub async fn execute(
        &self,
        command: ReceiveTransferCommand,
        actor_id: UserId,
    ) -> Result<TransferDetailResponse, InventoryError> {
        // 1. Find transfer with items
        let transfer_id = TransferId::from_uuid(command.transfer_id);
        let mut transfer = self
            .transfer_repo
            .find_by_id_with_items(transfer_id)
            .await?
            .ok_or(InventoryError::TransferNotFound(command.transfer_id))?;

        // 2. Receive the transfer (validates status is in_transit) (Requirement 11.6)
        transfer.receive(actor_id)?;

        // 3. Build a map of item_id -> received quantity from command
        let received_quantities: std::collections::HashMap<uuid::Uuid, rust_decimal::Decimal> = command
            .items
            .iter()
            .map(|item| (item.item_id, item.quantity_received))
            .collect();

        // 4. Process each item: increase destination stock and create movements (Requirement 11.7)
        let to_store_id = transfer.to_store_id();
        let transfer_uuid = transfer.id().into_uuid();

        for item in transfer.items_mut() {
            // Get received quantity from command, default to shipped if not specified
            let quantity_received = received_quantities
                .get(&item.id())
                .copied()
                .unwrap_or_else(|| item.quantity_shipped().unwrap_or(item.quantity_requested()));

            // Record received quantity on item
            item.record_received(quantity_received);

            // Find or create stock record for this item at destination store
            let stock = if let Some(product_id) = item.product_id() {
                let existing = self.stock_repo
                    .find_by_store_and_product(to_store_id, product_id)
                    .await?;
                
                match existing {
                    Some(s) => s,
                    None => {
                        // Create new stock record
                        let new_stock = InventoryStock::create_for_product(to_store_id, product_id)?;
                        self.stock_repo.save(&new_stock).await?;
                        new_stock
                    }
                }
            } else if let Some(variant_id) = item.variant_id() {
                let existing = self.stock_repo
                    .find_by_store_and_variant(to_store_id, variant_id)
                    .await?;
                
                match existing {
                    Some(s) => s,
                    None => {
                        // Create new stock record
                        let new_stock = InventoryStock::create_for_variant(to_store_id, variant_id)?;
                        self.stock_repo.save(&new_stock).await?;
                        new_stock
                    }
                }
            } else {
                return Err(InventoryError::InvalidProductVariantConstraint);
            };

            let mut stock = stock;

            // Increase stock (positive delta)
            let expected_version = stock.version();
            stock.adjust_quantity(quantity_received)?;
            stock.increment_version();

            // Update stock with optimistic locking
            self.stock_repo
                .update_with_version(&stock, expected_version)
                .await?;

            // Create transfer_in movement
            let movement = InventoryMovement::create(
                stock.id(),
                MovementType::TransferIn,
                Some(format!("Transfer from store")),
                quantity_received,
                item.unit_cost(),
                Currency::hnl(),
                stock.quantity(),
                Some("transfer".to_string()),
                Some(transfer_uuid),
                actor_id,
                None,
            );
            self.movement_repo.save(&movement).await?;
        }

        // 5. Update transfer
        self.transfer_repo.update(&transfer).await?;

        // 6. Convert to response
        Ok(self.to_response(&transfer))
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
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::application::dtos::commands::ReceiveTransferItemCommand;
    use crate::domain::entities::TransferItem;
    use crate::domain::value_objects::{ProductId, StockId, VariantId};
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    struct MockTransferRepository {
        transfers: Mutex<HashMap<TransferId, StockTransfer>>,
    }

    impl MockTransferRepository {
        fn new() -> Self {
            Self {
                transfers: Mutex::new(HashMap::new()),
            }
        }

        fn add_transfer(&self, transfer: StockTransfer) {
            let mut transfers = self.transfers.lock().unwrap();
            transfers.insert(transfer.id(), transfer);
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
            Ok("TRF-TEST-00001".to_string())
        }
    }

    struct MockStockRepository {
        stocks: Mutex<HashMap<StockId, InventoryStock>>,
        product_stocks: Mutex<HashMap<(StoreId, ProductId), StockId>>,
        variant_stocks: Mutex<HashMap<(StoreId, VariantId), StockId>>,
    }

    impl MockStockRepository {
        fn new() -> Self {
            Self {
                stocks: Mutex::new(HashMap::new()),
                product_stocks: Mutex::new(HashMap::new()),
                variant_stocks: Mutex::new(HashMap::new()),
            }
        }

        fn add_stock(&self, stock: InventoryStock) {
            let mut stocks = self.stocks.lock().unwrap();
            let stock_id = stock.id();
            
            if let Some(product_id) = stock.product_id() {
                let mut product_stocks = self.product_stocks.lock().unwrap();
                product_stocks.insert((stock.store_id(), product_id), stock_id);
            }
            if let Some(variant_id) = stock.variant_id() {
                let mut variant_stocks = self.variant_stocks.lock().unwrap();
                variant_stocks.insert((stock.store_id(), variant_id), stock_id);
            }
            
            stocks.insert(stock_id, stock);
        }

        fn get_stock_by_store_and_product(&self, store_id: StoreId, product_id: ProductId) -> Option<InventoryStock> {
            let product_stocks = self.product_stocks.lock().unwrap();
            if let Some(stock_id) = product_stocks.get(&(store_id, product_id)) {
                let stocks = self.stocks.lock().unwrap();
                stocks.get(stock_id).cloned()
            } else {
                None
            }
        }
    }

    #[async_trait]
    impl InventoryStockRepository for MockStockRepository {
        async fn save(&self, stock: &InventoryStock) -> Result<(), InventoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            let stock_id = stock.id();
            
            if let Some(product_id) = stock.product_id() {
                let mut product_stocks = self.product_stocks.lock().unwrap();
                product_stocks.insert((stock.store_id(), product_id), stock_id);
            }
            if let Some(variant_id) = stock.variant_id() {
                let mut variant_stocks = self.variant_stocks.lock().unwrap();
                variant_stocks.insert((stock.store_id(), variant_id), stock_id);
            }
            
            stocks.insert(stock_id, stock.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: StockId) -> Result<Option<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.get(&id).cloned())
        }

        async fn find_by_store_and_product(
            &self,
            store_id: StoreId,
            product_id: ProductId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            let product_stocks = self.product_stocks.lock().unwrap();
            if let Some(stock_id) = product_stocks.get(&(store_id, product_id)) {
                let stocks = self.stocks.lock().unwrap();
                Ok(stocks.get(stock_id).cloned())
            } else {
                Ok(None)
            }
        }

        async fn find_by_store_and_variant(
            &self,
            store_id: StoreId,
            variant_id: VariantId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            let variant_stocks = self.variant_stocks.lock().unwrap();
            if let Some(stock_id) = variant_stocks.get(&(store_id, variant_id)) {
                let stocks = self.stocks.lock().unwrap();
                Ok(stocks.get(stock_id).cloned())
            } else {
                Ok(None)
            }
        }

        async fn update_with_version(
            &self,
            stock: &InventoryStock,
            expected_version: i32,
        ) -> Result<(), InventoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            if let Some(existing) = stocks.get(&stock.id()) {
                if existing.version() != expected_version {
                    return Err(InventoryError::OptimisticLockError);
                }
                stocks.insert(stock.id(), stock.clone());
                Ok(())
            } else {
                Err(InventoryError::StockNotFound(stock.id().into_uuid()))
            }
        }

        async fn find_low_stock(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<InventoryStock>, InventoryError> {
            Ok(vec![])
        }

        async fn find_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<InventoryStock>, InventoryError> {
            Ok(vec![])
        }
    }

    struct MockMovementRepository {
        movements: Mutex<Vec<InventoryMovement>>,
    }

    impl MockMovementRepository {
        fn new() -> Self {
            Self {
                movements: Mutex::new(Vec::new()),
            }
        }

        fn get_movements(&self) -> Vec<InventoryMovement> {
            self.movements.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl InventoryMovementRepository for MockMovementRepository {
        async fn save(&self, movement: &InventoryMovement) -> Result<(), InventoryError> {
            let mut movements = self.movements.lock().unwrap();
            movements.push(movement.clone());
            Ok(())
        }

        async fn find_by_stock_id(
            &self,
            _stock_id: StockId,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            Ok(vec![])
        }

        async fn find_by_reference(
            &self,
            _reference_type: &str,
            _reference_id: Uuid,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            Ok(vec![])
        }

        async fn calculate_weighted_average_cost(
            &self,
            _stock_id: StockId,
        ) -> Result<Option<Decimal>, InventoryError> {
            Ok(None)
        }
    }

    fn create_in_transit_transfer(from_store_id: StoreId, to_store_id: StoreId, product_id: ProductId) -> StockTransfer {
        let mut transfer = StockTransfer::create(
            "TRF-TEST-00001".to_string(),
            from_store_id,
            to_store_id,
            UserId::new(),
        ).unwrap();
        
        let mut item = TransferItem::create_for_product(
            transfer.id(),
            product_id,
            dec!(10),
            Some(dec!(5.00)),
        ).unwrap();
        item.record_shipped(dec!(10));
        transfer.add_item(item).unwrap();
        transfer.submit().unwrap();
        transfer.ship(UserId::new()).unwrap();
        transfer
    }

    fn create_stock_with_quantity(store_id: StoreId, product_id: ProductId, quantity: Decimal) -> InventoryStock {
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(quantity).unwrap();
        stock
    }

    #[tokio::test]
    async fn test_receive_transfer_success() {
        let transfer_repo = Arc::new(MockTransferRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let from_store_id = StoreId::new();
        let to_store_id = StoreId::new();
        let product_id = ProductId::new();

        // Create existing stock at destination with 50 units
        let dest_stock = create_stock_with_quantity(to_store_id, product_id, dec!(50));
        stock_repo.add_stock(dest_stock);

        // Create in_transit transfer
        let transfer = create_in_transit_transfer(from_store_id, to_store_id, product_id);
        let transfer_id = transfer.id();
        let item_id = transfer.items()[0].id();
        transfer_repo.add_transfer(transfer);

        let use_case = ReceiveTransferUseCase::new(
            transfer_repo.clone(),
            stock_repo.clone(),
            movement_repo.clone(),
        );

        let command = ReceiveTransferCommand {
            transfer_id: transfer_id.into_uuid(),
            items: vec![ReceiveTransferItemCommand {
                item_id,
                quantity_received: dec!(10),
            }],
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "completed");
        assert!(response.received_date.is_some());
        assert!(response.received_by_id.is_some());
        assert_eq!(response.items[0].quantity_received, Some(dec!(10)));

        // Verify stock was increased at destination
        let updated_stock = stock_repo
            .get_stock_by_store_and_product(to_store_id, product_id)
            .unwrap();
        assert_eq!(updated_stock.quantity(), dec!(60)); // 50 + 10

        // Verify movement was created
        let movements = movement_repo.get_movements();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].movement_type(), MovementType::TransferIn);
        assert_eq!(movements[0].quantity(), dec!(10));
    }

    #[tokio::test]
    async fn test_receive_transfer_creates_stock_if_not_exists() {
        let transfer_repo = Arc::new(MockTransferRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let from_store_id = StoreId::new();
        let to_store_id = StoreId::new();
        let product_id = ProductId::new();

        // No stock at destination - should be created

        // Create in_transit transfer
        let transfer = create_in_transit_transfer(from_store_id, to_store_id, product_id);
        let transfer_id = transfer.id();
        let item_id = transfer.items()[0].id();
        transfer_repo.add_transfer(transfer);

        let use_case = ReceiveTransferUseCase::new(
            transfer_repo.clone(),
            stock_repo.clone(),
            movement_repo.clone(),
        );

        let command = ReceiveTransferCommand {
            transfer_id: transfer_id.into_uuid(),
            items: vec![ReceiveTransferItemCommand {
                item_id,
                quantity_received: dec!(10),
            }],
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(result.is_ok());

        // Verify stock was created at destination
        let created_stock = stock_repo
            .get_stock_by_store_and_product(to_store_id, product_id)
            .unwrap();
        assert_eq!(created_stock.quantity(), dec!(10));
    }

    #[tokio::test]
    async fn test_receive_transfer_partial_quantity() {
        let transfer_repo = Arc::new(MockTransferRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let from_store_id = StoreId::new();
        let to_store_id = StoreId::new();
        let product_id = ProductId::new();

        // Create in_transit transfer (shipped 10)
        let transfer = create_in_transit_transfer(from_store_id, to_store_id, product_id);
        let transfer_id = transfer.id();
        let item_id = transfer.items()[0].id();
        transfer_repo.add_transfer(transfer);

        let use_case = ReceiveTransferUseCase::new(
            transfer_repo.clone(),
            stock_repo.clone(),
            movement_repo.clone(),
        );

        // Receive only 8 (2 damaged/lost)
        let command = ReceiveTransferCommand {
            transfer_id: transfer_id.into_uuid(),
            items: vec![ReceiveTransferItemCommand {
                item_id,
                quantity_received: dec!(8),
            }],
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.items[0].quantity_shipped, Some(dec!(10)));
        assert_eq!(response.items[0].quantity_received, Some(dec!(8)));

        // Verify only 8 was added to stock
        let created_stock = stock_repo
            .get_stock_by_store_and_product(to_store_id, product_id)
            .unwrap();
        assert_eq!(created_stock.quantity(), dec!(8));
    }

    #[tokio::test]
    async fn test_receive_transfer_not_found() {
        let transfer_repo = Arc::new(MockTransferRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let use_case = ReceiveTransferUseCase::new(transfer_repo, stock_repo, movement_repo);

        let command = ReceiveTransferCommand {
            transfer_id: new_uuid(),
            items: vec![],
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(result, Err(InventoryError::TransferNotFound(_))));
    }

    #[tokio::test]
    async fn test_receive_transfer_wrong_status() {
        let transfer_repo = Arc::new(MockTransferRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let from_store_id = StoreId::new();
        let to_store_id = StoreId::new();
        let product_id = ProductId::new();

        // Create pending transfer (not shipped yet)
        let mut transfer = StockTransfer::create(
            "TRF-TEST-00001".to_string(),
            from_store_id,
            to_store_id,
            UserId::new(),
        ).unwrap();
        let item = TransferItem::create_for_product(
            transfer.id(),
            product_id,
            dec!(10),
            None,
        ).unwrap();
        transfer.add_item(item).unwrap();
        transfer.submit().unwrap();
        // NOT shipped - still pending
        let transfer_id = transfer.id();
        transfer_repo.add_transfer(transfer);

        let use_case = ReceiveTransferUseCase::new(transfer_repo, stock_repo, movement_repo);

        let command = ReceiveTransferCommand {
            transfer_id: transfer_id.into_uuid(),
            items: vec![],
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(result, Err(InventoryError::InvalidStatusTransition)));
    }
}
