// ApplyAdjustmentUseCase - applies an approved adjustment to inventory

use std::sync::Arc;

use crate::application::dtos::commands::ApplyAdjustmentCommand;
use crate::application::dtos::responses::{AdjustmentDetailResponse, AdjustmentItemResponse};
use crate::domain::entities::{InventoryMovement, StockAdjustment};
use crate::domain::repositories::{AdjustmentRepository, InventoryMovementRepository, InventoryStockRepository};
use crate::domain::value_objects::{AdjustmentId, Currency, MovementType};
use crate::InventoryError;
use identity::UserId;

/// Use case for applying an approved stock adjustment to inventory.
///
/// Validates status is approved, updates stock with optimistic locking for each item,
/// records balance_before/after, creates movements, and marks adjustment as applied.
/// Requirements: 9.7, 10.3
pub struct ApplyAdjustmentUseCase<A, S, M>
where
    A: AdjustmentRepository,
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
{
    adjustment_repo: Arc<A>,
    stock_repo: Arc<S>,
    movement_repo: Arc<M>,
}

impl<A, S, M> ApplyAdjustmentUseCase<A, S, M>
where
    A: AdjustmentRepository,
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
{
    /// Creates a new instance of ApplyAdjustmentUseCase
    pub fn new(adjustment_repo: Arc<A>, stock_repo: Arc<S>, movement_repo: Arc<M>) -> Self {
        Self {
            adjustment_repo,
            stock_repo,
            movement_repo,
        }
    }

    /// Executes the use case to apply an approved adjustment
    ///
    /// # Arguments
    /// * `command` - The apply adjustment command containing adjustment ID
    /// * `actor_id` - ID of the user applying the adjustment
    ///
    /// # Returns
    /// AdjustmentDetailResponse on success
    ///
    /// # Errors
    /// * `InventoryError::AdjustmentNotFound` - If adjustment doesn't exist
    /// * `InventoryError::InvalidStatusTransition` - If adjustment is not in approved status
    /// * `InventoryError::StockNotFound` - If a stock record doesn't exist
    /// * `InventoryError::OptimisticLockError` - If concurrent modification detected
    /// * `InventoryError::NegativeStock` - If adjustment would result in negative stock
    pub async fn execute(
        &self,
        command: ApplyAdjustmentCommand,
        actor_id: UserId,
    ) -> Result<AdjustmentDetailResponse, InventoryError> {
        // 1. Find adjustment with items
        let adjustment_id = AdjustmentId::from_uuid(command.adjustment_id);
        let mut adjustment = self
            .adjustment_repo
            .find_by_id_with_items(adjustment_id)
            .await?
            .ok_or(InventoryError::AdjustmentNotFound(command.adjustment_id))?;

        // 2. Validate status is approved (Requirement 9.7)
        if !adjustment.status().can_apply() {
            return Err(InventoryError::InvalidStatusTransition);
        }

        // Extract values needed for movements before mutable borrow
        let adjustment_reason = adjustment.adjustment_reason().to_string();
        let adjustment_uuid = adjustment.id().into_uuid();

        // 3. Apply each item (Requirement 10.3)
        for item in adjustment.items_mut() {
            // Find stock record
            let mut stock = self
                .stock_repo
                .find_by_id(item.stock_id())
                .await?
                .ok_or(InventoryError::StockNotFound(item.stock_id().into_uuid()))?;

            // Record balance before
            let balance_before = stock.quantity();
            let expected_version = stock.version();

            // Apply quantity change
            stock.adjust_quantity(item.quantity())?;
            stock.increment_version();

            // Record balance after
            let balance_after = stock.quantity();
            item.record_balances(balance_before, balance_after);

            // Update stock with optimistic locking
            self.stock_repo
                .update_with_version(&stock, expected_version)
                .await?;

            // Create inventory movement (Requirement 9.7)
            let movement = InventoryMovement::create(
                item.stock_id(),
                MovementType::Adjustment,
                Some(adjustment_reason.clone()),
                item.quantity(),
                item.unit_cost(),
                Currency::hnl(), // Default currency
                balance_after,
                Some("adjustment".to_string()),
                Some(adjustment_uuid),
                actor_id,
                item.notes().map(|s| s.to_string()),
            );
            self.movement_repo.save(&movement).await?;
        }

        // 4. Mark adjustment as applied
        adjustment.mark_applied()?;

        // 5. Update adjustment
        self.adjustment_repo.update(&adjustment).await?;

        // 6. Convert to response
        Ok(self.to_response(&adjustment))
    }

    fn to_response(&self, adjustment: &StockAdjustment) -> AdjustmentDetailResponse {
        let items: Vec<AdjustmentItemResponse> = adjustment
            .items()
            .iter()
            .map(|item| AdjustmentItemResponse {
                id: item.id(),
                adjustment_id: item.adjustment_id().into_uuid(),
                stock_id: item.stock_id().into_uuid(),
                stock: None,
                quantity: item.quantity(),
                unit_cost: item.unit_cost(),
                balance_before: item.balance_before(),
                balance_after: item.balance_after(),
                notes: item.notes().map(|s| s.to_string()),
                created_at: item.created_at(),
            })
            .collect();

        AdjustmentDetailResponse {
            id: adjustment.id().into_uuid(),
            store_id: adjustment.store_id().into_uuid(),
            adjustment_number: adjustment.adjustment_number().to_string(),
            adjustment_type: adjustment.adjustment_type().to_string(),
            adjustment_reason: adjustment.adjustment_reason().to_string(),
            status: adjustment.status().to_string(),
            created_by_id: adjustment.created_by_id().into_uuid(),
            approved_by_id: adjustment.approved_by_id().map(|id| id.into_uuid()),
            approved_at: adjustment.approved_at(),
            applied_at: adjustment.applied_at(),
            notes: adjustment.notes().map(|s| s.to_string()),
            attachments: Some(adjustment.attachments().clone()),
            items,
            created_at: adjustment.created_at(),
            updated_at: adjustment.updated_at(),
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

    use crate::domain::entities::{AdjustmentItem, InventoryStock};
    use crate::domain::value_objects::{
        AdjustmentReason, AdjustmentStatus, AdjustmentType, ProductId, StockId, VariantId,
    };
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    struct MockAdjustmentRepository {
        adjustments: Mutex<HashMap<AdjustmentId, StockAdjustment>>,
    }

    impl MockAdjustmentRepository {
        fn new() -> Self {
            Self {
                adjustments: Mutex::new(HashMap::new()),
            }
        }

        fn add_adjustment(&self, adjustment: StockAdjustment) {
            let mut adjustments = self.adjustments.lock().unwrap();
            adjustments.insert(adjustment.id(), adjustment);
        }
    }

    #[async_trait]
    impl AdjustmentRepository for MockAdjustmentRepository {
        async fn save(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError> {
            let mut adjustments = self.adjustments.lock().unwrap();
            adjustments.insert(adjustment.id(), adjustment.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: AdjustmentId,
        ) -> Result<Option<StockAdjustment>, InventoryError> {
            let adjustments = self.adjustments.lock().unwrap();
            Ok(adjustments.get(&id).cloned())
        }

        async fn find_by_id_with_items(
            &self,
            id: AdjustmentId,
        ) -> Result<Option<StockAdjustment>, InventoryError> {
            self.find_by_id(id).await
        }

        async fn find_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<StockAdjustment>, InventoryError> {
            Ok(vec![])
        }

        async fn update(&self, adjustment: &StockAdjustment) -> Result<(), InventoryError> {
            let mut adjustments = self.adjustments.lock().unwrap();
            adjustments.insert(adjustment.id(), adjustment.clone());
            Ok(())
        }

        async fn generate_adjustment_number(
            &self,
            _store_id: StoreId,
        ) -> Result<String, InventoryError> {
            Ok("ADJ-TEST-00001".to_string())
        }
    }

    struct MockStockRepository {
        stocks: Mutex<HashMap<StockId, InventoryStock>>,
    }

    impl MockStockRepository {
        fn new() -> Self {
            Self {
                stocks: Mutex::new(HashMap::new()),
            }
        }

        fn add_stock(&self, stock: InventoryStock) {
            let mut stocks = self.stocks.lock().unwrap();
            stocks.insert(stock.id(), stock);
        }
    }

    #[async_trait]
    impl InventoryStockRepository for MockStockRepository {
        async fn save(&self, stock: &InventoryStock) -> Result<(), InventoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            stocks.insert(stock.id(), stock.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: StockId) -> Result<Option<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.get(&id).cloned())
        }

        async fn find_by_store_and_product(
            &self,
            _store_id: StoreId,
            _product_id: ProductId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_store_and_variant(
            &self,
            _store_id: StoreId,
            _variant_id: VariantId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            unimplemented!()
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
            unimplemented!()
        }

        async fn find_by_store(
            &self,
            _store_id: StoreId,
        ) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
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
            unimplemented!()
        }

        async fn find_by_reference(
            &self,
            _reference_type: &str,
            _reference_id: Uuid,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            unimplemented!()
        }

        async fn calculate_weighted_average_cost(
            &self,
            _stock_id: StockId,
        ) -> Result<Option<Decimal>, InventoryError> {
            unimplemented!()
        }
    }

    fn create_approved_adjustment(stock_id: StockId) -> StockAdjustment {
        let mut adjustment = StockAdjustment::create(
            StoreId::new(),
            "ADJ-TEST-00001".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Damage,
            UserId::new(),
        );
        let item = AdjustmentItem::create(
            adjustment.id(),
            stock_id,
            dec!(-10),
            Some(dec!(5.00)),
        );
        adjustment.add_item(item).unwrap();
        adjustment.submit_for_approval().unwrap();
        adjustment.approve(UserId::new()).unwrap();
        adjustment
    }

    fn create_stock_with_quantity(quantity: Decimal) -> InventoryStock {
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(quantity).unwrap();
        stock
    }

    #[tokio::test]
    async fn test_apply_adjustment_success() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create stock with 100 units
        let stock = create_stock_with_quantity(dec!(100));
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create approved adjustment to decrease by 10
        let adjustment = create_approved_adjustment(stock_id);
        let adjustment_id = adjustment.id();
        adjustment_repo.add_adjustment(adjustment);

        let use_case = ApplyAdjustmentUseCase::new(
            adjustment_repo.clone(),
            stock_repo.clone(),
            movement_repo.clone(),
        );

        let command = ApplyAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "applied");
        assert!(response.applied_at.is_some());

        // Verify balance_before and balance_after are recorded
        assert_eq!(response.items[0].balance_before, Some(dec!(100)));
        assert_eq!(response.items[0].balance_after, Some(dec!(90)));

        // Verify stock was updated
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.quantity(), dec!(90));

        // Verify movement was created
        let movements = movement_repo.get_movements();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].movement_type(), MovementType::Adjustment);
        assert_eq!(movements[0].quantity(), dec!(-10));
        assert_eq!(movements[0].balance_after(), dec!(90));

        // Verify adjustment status in repository
        let updated_adjustment = adjustment_repo
            .find_by_id(adjustment_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_adjustment.status(), AdjustmentStatus::Applied);
    }

    #[tokio::test]
    async fn test_apply_adjustment_increase() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create stock with 50 units
        let stock = create_stock_with_quantity(dec!(50));
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create approved adjustment to increase by 20
        let mut adjustment = StockAdjustment::create(
            StoreId::new(),
            "ADJ-TEST-00002".to_string(),
            AdjustmentType::Increase,
            AdjustmentReason::Found,
            UserId::new(),
        );
        let item = AdjustmentItem::create(adjustment.id(), stock_id, dec!(20), Some(dec!(10.00)));
        adjustment.add_item(item).unwrap();
        adjustment.submit_for_approval().unwrap();
        adjustment.approve(UserId::new()).unwrap();
        let adjustment_id = adjustment.id();
        adjustment_repo.add_adjustment(adjustment);

        let use_case = ApplyAdjustmentUseCase::new(
            adjustment_repo.clone(),
            stock_repo.clone(),
            movement_repo.clone(),
        );

        let command = ApplyAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.items[0].balance_before, Some(dec!(50)));
        assert_eq!(response.items[0].balance_after, Some(dec!(70)));

        // Verify stock was updated
        let updated_stock = stock_repo.find_by_id(stock_id).await.unwrap().unwrap();
        assert_eq!(updated_stock.quantity(), dec!(70));
    }

    #[tokio::test]
    async fn test_apply_adjustment_not_found() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        let use_case = ApplyAdjustmentUseCase::new(adjustment_repo, stock_repo, movement_repo);

        let command = ApplyAdjustmentCommand {
            adjustment_id: new_uuid(),
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(result, Err(InventoryError::AdjustmentNotFound(_))));
    }

    #[tokio::test]
    async fn test_apply_adjustment_wrong_status() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create pending adjustment (not approved)
        let stock = create_stock_with_quantity(dec!(100));
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        let mut adjustment = StockAdjustment::create(
            StoreId::new(),
            "ADJ-TEST-00001".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Damage,
            UserId::new(),
        );
        let item = AdjustmentItem::create(adjustment.id(), stock_id, dec!(-10), None);
        adjustment.add_item(item).unwrap();
        adjustment.submit_for_approval().unwrap();
        // Not approved!
        let adjustment_id = adjustment.id();
        adjustment_repo.add_adjustment(adjustment);

        let use_case = ApplyAdjustmentUseCase::new(adjustment_repo, stock_repo, movement_repo);

        let command = ApplyAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(
            result,
            Err(InventoryError::InvalidStatusTransition)
        ));
    }

    #[tokio::test]
    async fn test_apply_adjustment_stock_not_found() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create adjustment with non-existent stock
        let adjustment = create_approved_adjustment(StockId::new());
        let adjustment_id = adjustment.id();
        adjustment_repo.add_adjustment(adjustment);

        let use_case = ApplyAdjustmentUseCase::new(adjustment_repo, stock_repo, movement_repo);

        let command = ApplyAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(result, Err(InventoryError::StockNotFound(_))));
    }

    #[tokio::test]
    async fn test_apply_adjustment_negative_stock() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create stock with only 5 units
        let stock = create_stock_with_quantity(dec!(5));
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        // Create adjustment to decrease by 10 (more than available)
        let adjustment = create_approved_adjustment(stock_id);
        let adjustment_id = adjustment.id();
        adjustment_repo.add_adjustment(adjustment);

        let use_case = ApplyAdjustmentUseCase::new(adjustment_repo, stock_repo, movement_repo);

        let command = ApplyAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(matches!(result, Err(InventoryError::NegativeStock)));
    }

    #[tokio::test]
    async fn test_apply_adjustment_multiple_items() {
        let adjustment_repo = Arc::new(MockAdjustmentRepository::new());
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());

        // Create two stocks
        let stock1 = create_stock_with_quantity(dec!(100));
        let stock1_id = stock1.id();
        stock_repo.add_stock(stock1);

        let stock2 = create_stock_with_quantity(dec!(50));
        let stock2_id = stock2.id();
        stock_repo.add_stock(stock2);

        // Create adjustment with two items
        let mut adjustment = StockAdjustment::create(
            StoreId::new(),
            "ADJ-TEST-00003".to_string(),
            AdjustmentType::Decrease,
            AdjustmentReason::Correction,
            UserId::new(),
        );
        let item1 = AdjustmentItem::create(adjustment.id(), stock1_id, dec!(-15), None);
        let item2 = AdjustmentItem::create(adjustment.id(), stock2_id, dec!(-5), None);
        adjustment.add_item(item1).unwrap();
        adjustment.add_item(item2).unwrap();
        adjustment.submit_for_approval().unwrap();
        adjustment.approve(UserId::new()).unwrap();
        let adjustment_id = adjustment.id();
        adjustment_repo.add_adjustment(adjustment);

        let use_case = ApplyAdjustmentUseCase::new(
            adjustment_repo.clone(),
            stock_repo.clone(),
            movement_repo.clone(),
        );

        let command = ApplyAdjustmentCommand {
            adjustment_id: adjustment_id.into_uuid(),
        };

        let result = use_case.execute(command, UserId::new()).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.items.len(), 2);

        // Verify both stocks were updated
        let updated_stock1 = stock_repo.find_by_id(stock1_id).await.unwrap().unwrap();
        assert_eq!(updated_stock1.quantity(), dec!(85));

        let updated_stock2 = stock_repo.find_by_id(stock2_id).await.unwrap().unwrap();
        assert_eq!(updated_stock2.quantity(), dec!(45));

        // Verify two movements were created
        let movements = movement_repo.get_movements();
        assert_eq!(movements.len(), 2);
    }
}
