// UpdateStockUseCase - updates stock with optimistic locking

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::commands::UpdateStockCommand;
use crate::application::dtos::responses::StockResponse;
use crate::domain::entities::InventoryMovement;
use crate::domain::repositories::{InventoryMovementRepository, InventoryStockRepository};
use crate::domain::value_objects::{Currency, MovementType, StockId};
use crate::InventoryError;
use identity::domain::entities::AuditEntry;
use identity::domain::repositories::AuditRepository;
use identity::domain::value_objects::UserId;

/// Use case for updating stock quantity with optimistic locking.
///
/// Validates version matches, applies quantity change, increments version,
/// creates inventory movement, and creates audit entry.
///
pub struct UpdateStockUseCase<S, M, A>
where
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
    A: AuditRepository,
{
    stock_repo: Arc<S>,
    movement_repo: Arc<M>,
    audit_repo: Arc<A>,
}

impl<S, M, A> UpdateStockUseCase<S, M, A>
where
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
    A: AuditRepository,
{
    /// Creates a new instance of UpdateStockUseCase
    pub fn new(stock_repo: Arc<S>, movement_repo: Arc<M>, audit_repo: Arc<A>) -> Self {
        Self {
            stock_repo,
            movement_repo,
            audit_repo,
        }
    }

    /// Executes the use case to update stock quantity
    ///
    /// # Arguments
    /// * `command` - The update stock command containing stock data
    /// * `actor_id` - ID of the user performing this action (for audit and movement)
    ///
    /// # Returns
    /// StockResponse on success
    ///
    /// # Errors
    /// * `InventoryError::StockNotFound` - If stock record doesn't exist
    /// * `InventoryError::OptimisticLockError` - If version mismatch (concurrent modification)
    /// * `InventoryError::NegativeStock` - If adjustment would result in negative stock
    /// * `InventoryError::ReservedExceedsQuantity` - If adjustment would make quantity < reserved
    /// * `InventoryError::InvalidMovementType` - If movement type is invalid
    pub async fn execute(
        &self,
        command: UpdateStockCommand,
        actor_id: UserId,
    ) -> Result<StockResponse, InventoryError> {
        // 1. Parse and validate movement type (Requirement 5.1)
        let movement_type = MovementType::from_str(&command.movement_type)?;

        // 2. Find stock record
        let stock_id = StockId::from_uuid(command.stock_id);
        let mut stock = self
            .stock_repo
            .find_by_id(stock_id)
            .await?
            .ok_or(InventoryError::StockNotFound(command.stock_id))?;

        // 3. Validate version matches (Requirement 3.3)
        if stock.version() != command.expected_version {
            return Err(InventoryError::OptimisticLockError);
        }

        // Store old state for audit
        let old_stock = stock.clone();

        // 4. Apply quantity change (Requirement 3.4)
        stock.adjust_quantity(command.quantity_delta)?;
        stock.increment_version();

        // 5. Update with optimistic lock (Requirement 3.3)
        self.stock_repo
            .update_with_version(&stock, command.expected_version)
            .await?;

        // 6. Record movement (Requirement 5.1, 5.3)
        let movement = InventoryMovement::create(
            stock_id,
            movement_type,
            command.movement_reason,
            command.quantity_delta,
            command.unit_cost,
            Currency::hnl(), // Default currency, could be parameterized
            stock.quantity(),
            command.reference_type,
            command.reference_id,
            actor_id,
            command.notes,
        );
        self.movement_repo.save(&movement).await?;

        // 7. Create audit entry
        let audit_entry =
            AuditEntry::for_update("inventory_stock", stock_id.into_uuid(), &old_stock, &stock, actor_id);
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|_| InventoryError::NotImplemented)?;

        // 8. Convert to response
        Ok(StockResponse {
            id: stock.id().into_uuid(),
            store_id: stock.store_id().into_uuid(),
            product_id: stock.product_id().map(|id| id.into_uuid()),
            variant_id: stock.variant_id().map(|id| id.into_uuid()),
            quantity: stock.quantity(),
            reserved_quantity: stock.reserved_quantity(),
            available_quantity: stock.available_quantity(),
            version: stock.version(),
            min_stock_level: stock.min_stock_level(),
            max_stock_level: stock.max_stock_level(),
            is_low_stock: stock.is_low_stock(),
            created_at: stock.created_at(),
            updated_at: stock.updated_at(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use identity::domain::entities::AuditEntry;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::domain::entities::InventoryStock;
    use crate::domain::value_objects::ProductId;
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    // Mock repositories for testing
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
            _variant_id: crate::domain::value_objects::VariantId,
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

        async fn find_paginated(
            &self,
            _store_id: Option<StoreId>,
            _product_id: Option<ProductId>,
            _low_stock_only: bool,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<InventoryStock>, i64), InventoryError> {
            unimplemented!()
        }

        async fn find_by_product(
            &self,
            _product_id: ProductId,
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

    struct MockAuditRepository {
        entries: Mutex<Vec<AuditEntry>>,
    }

    impl MockAuditRepository {
        fn new() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl AuditRepository for MockAuditRepository {
        async fn save(&self, entry: &AuditEntry) -> Result<(), identity::IdentityError> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
            Ok(())
        }

        async fn find_by_entity(
            &self,
            _entity_type: &str,
            _entity_id: Uuid,
        ) -> Result<Vec<AuditEntry>, identity::IdentityError> {
            unimplemented!()
        }

        async fn find_by_date_range(
            &self,
            _from: chrono::DateTime<Utc>,
            _to: chrono::DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, identity::IdentityError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_update_stock_increase() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create initial stock
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        let stock_id = stock.id();
        let initial_version = stock.version();
        stock_repo.add_stock(stock);

        let use_case = UpdateStockUseCase::new(stock_repo, movement_repo.clone(), audit_repo);

        let command = UpdateStockCommand {
            stock_id: stock_id.into_uuid(),
            quantity_delta: dec!(50),
            movement_type: "in".to_string(),
            movement_reason: Some("purchase".to_string()),
            unit_cost: Some(dec!(10.00)),
            reference_type: Some("purchase_order".to_string()),
            reference_id: Some(new_uuid()),
            notes: Some("Restocking".to_string()),
            expected_version: initial_version,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.quantity, dec!(150));
        assert_eq!(response.version, initial_version + 1);

        // Verify movement was created
        let movements = movement_repo.movements.lock().unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].quantity(), dec!(50));
        assert_eq!(movements[0].movement_type(), MovementType::In);
    }

    #[tokio::test]
    async fn test_update_stock_decrease() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create initial stock
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        let stock_id = stock.id();
        let initial_version = stock.version();
        stock_repo.add_stock(stock);

        let use_case = UpdateStockUseCase::new(stock_repo, movement_repo, audit_repo);

        let command = UpdateStockCommand {
            stock_id: stock_id.into_uuid(),
            quantity_delta: dec!(-30),
            movement_type: "out".to_string(),
            movement_reason: Some("sale".to_string()),
            unit_cost: None,
            reference_type: Some("order".to_string()),
            reference_id: Some(new_uuid()),
            notes: None,
            expected_version: initial_version,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.quantity, dec!(70));
    }

    #[tokio::test]
    async fn test_update_stock_version_mismatch() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create initial stock
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(100)).unwrap();
        let stock_id = stock.id();
        stock_repo.add_stock(stock);

        let use_case = UpdateStockUseCase::new(stock_repo, movement_repo, audit_repo);

        // Use wrong version
        let command = UpdateStockCommand {
            stock_id: stock_id.into_uuid(),
            quantity_delta: dec!(50),
            movement_type: "in".to_string(),
            movement_reason: None,
            unit_cost: None,
            reference_type: None,
            reference_id: None,
            notes: None,
            expected_version: 999, // Wrong version
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::OptimisticLockError)));
    }

    #[tokio::test]
    async fn test_update_stock_not_found() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        let use_case = UpdateStockUseCase::new(stock_repo, movement_repo, audit_repo);

        let command = UpdateStockCommand {
            stock_id: new_uuid(),
            quantity_delta: dec!(50),
            movement_type: "in".to_string(),
            movement_reason: None,
            unit_cost: None,
            reference_type: None,
            reference_id: None,
            notes: None,
            expected_version: 1,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::StockNotFound(_))));
    }

    #[tokio::test]
    async fn test_update_stock_negative_result() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create initial stock with 50 units
        let store_id = StoreId::new();
        let product_id = ProductId::new();
        let mut stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock.adjust_quantity(dec!(50)).unwrap();
        let stock_id = stock.id();
        let initial_version = stock.version();
        stock_repo.add_stock(stock);

        let use_case = UpdateStockUseCase::new(stock_repo, movement_repo, audit_repo);

        // Try to decrease by more than available
        let command = UpdateStockCommand {
            stock_id: stock_id.into_uuid(),
            quantity_delta: dec!(-100),
            movement_type: "out".to_string(),
            movement_reason: None,
            unit_cost: None,
            reference_type: None,
            reference_id: None,
            notes: None,
            expected_version: initial_version,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::NegativeStock)));
    }
}
