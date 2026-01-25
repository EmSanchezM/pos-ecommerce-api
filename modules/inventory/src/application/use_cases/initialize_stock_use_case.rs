// InitializeStockUseCase - initializes stock for a product/variant in a store

use rust_decimal::Decimal;
use std::sync::Arc;

use crate::application::dtos::commands::InitializeStockCommand;
use crate::application::dtos::responses::StockResponse;
use crate::domain::entities::{InventoryMovement, InventoryStock};
use crate::domain::repositories::{
    InventoryMovementRepository, InventoryStockRepository, ProductRepository,
};
use crate::domain::value_objects::{Currency, MovementType, ProductId, VariantId};
use crate::InventoryError;
use identity::domain::entities::AuditEntry;
use identity::domain::repositories::AuditRepository;
use identity::domain::value_objects::UserId;
use identity::StoreId;

/// Use case for initializing stock for a product or variant in a specific store.
///
/// This creates a new inventory_stock record that associates a product/variant
/// with a store, enabling stock tracking for that location.
///
/// Validates:
/// - Either product_id OR variant_id must be provided (not both, not neither)
/// - Product/variant must exist
/// - Stock record doesn't already exist for this store+product/variant combination
pub struct InitializeStockUseCase<S, P, M, A>
where
    S: InventoryStockRepository,
    P: ProductRepository,
    M: InventoryMovementRepository,
    A: AuditRepository,
{
    stock_repo: Arc<S>,
    product_repo: Arc<P>,
    movement_repo: Arc<M>,
    audit_repo: Arc<A>,
}

impl<S, P, M, A> InitializeStockUseCase<S, P, M, A>
where
    S: InventoryStockRepository,
    P: ProductRepository,
    M: InventoryMovementRepository,
    A: AuditRepository,
{
    /// Creates a new instance of InitializeStockUseCase
    pub fn new(
        stock_repo: Arc<S>,
        product_repo: Arc<P>,
        movement_repo: Arc<M>,
        audit_repo: Arc<A>,
    ) -> Self {
        Self {
            stock_repo,
            product_repo,
            movement_repo,
            audit_repo,
        }
    }

    /// Executes the use case to initialize stock
    ///
    /// # Arguments
    /// * `command` - The initialize stock command
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// StockResponse on success
    ///
    /// # Errors
    /// * `InventoryError::InvalidProductVariantConstraint` - If both or neither product_id/variant_id provided
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    /// * `InventoryError::VariantNotFound` - If variant doesn't exist
    /// * `InventoryError::StockAlreadyExists` - If stock record already exists for this combination
    pub async fn execute(
        &self,
        command: InitializeStockCommand,
        actor_id: UserId,
    ) -> Result<StockResponse, InventoryError> {
        let store_id = StoreId::from_uuid(command.store_id);

        // 1. Validate XOR constraint: exactly one of product_id or variant_id must be provided
        let (product_id, variant_id) = match (command.product_id, command.variant_id) {
            (Some(pid), None) => (Some(ProductId::from_uuid(pid)), None),
            (None, Some(vid)) => (None, Some(VariantId::from_uuid(vid))),
            _ => return Err(InventoryError::InvalidProductVariantConstraint),
        };

        // 2. Validate product/variant exists
        if let Some(pid) = product_id {
            if self.product_repo.find_by_id(pid).await?.is_none() {
                return Err(InventoryError::ProductNotFound(pid.into_uuid()));
            }

            // 3. Check if stock already exists for this store+product
            if self
                .stock_repo
                .find_by_store_and_product(store_id, pid)
                .await?
                .is_some()
            {
                return Err(InventoryError::StockAlreadyExists {
                    store_id: store_id.into_uuid(),
                    product_id: Some(pid.into_uuid()),
                    variant_id: None,
                });
            }
        }

        if let Some(vid) = variant_id {
            if self.product_repo.find_variant_by_id(vid).await?.is_none() {
                return Err(InventoryError::VariantNotFound(vid.into_uuid()));
            }

            // 3. Check if stock already exists for this store+variant
            if self
                .stock_repo
                .find_by_store_and_variant(store_id, vid)
                .await?
                .is_some()
            {
                return Err(InventoryError::StockAlreadyExists {
                    store_id: store_id.into_uuid(),
                    product_id: None,
                    variant_id: Some(vid.into_uuid()),
                });
            }
        }

        // 4. Create stock entity
        let mut stock = if let Some(pid) = product_id {
            InventoryStock::create_for_product(store_id, pid)?
        } else {
            InventoryStock::create_for_variant(store_id, variant_id.unwrap())?
        };

        // 5. Apply optional settings
        stock.set_min_stock_level(command.min_stock_level);
        stock.set_max_stock_level(command.max_stock_level);

        // 6. Apply initial quantity if provided
        if command.initial_quantity > Decimal::ZERO {
            stock.adjust_quantity(command.initial_quantity)?;
        }

        // 7. Save stock record
        self.stock_repo.save(&stock).await?;

        // 8. Create initial movement if there's initial quantity
        if command.initial_quantity > Decimal::ZERO {
            let movement = InventoryMovement::create(
                stock.id(),
                MovementType::In,
                Some("initial_stock".to_string()),
                command.initial_quantity,
                None, // unit_cost
                Currency::hnl(),
                stock.quantity(),
                Some("stock_initialization".to_string()),
                Some(stock.id().into_uuid()),
                actor_id,
                Some("Initial stock setup".to_string()),
            );
            self.movement_repo.save(&movement).await?;
        }

        // 9. Create audit entry
        let audit_entry = AuditEntry::for_create(
            "inventory_stock",
            stock.id().into_uuid(),
            &stock,
            actor_id,
        );
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|_| InventoryError::NotImplemented)?;

        // 10. Return response
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
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    use crate::domain::entities::{InventoryMovement, Product, ProductVariant};
    use crate::domain::value_objects::{Barcode, Sku, StockId, UnitOfMeasure};

    // Mock repositories
    struct MockStockRepository {
        stocks: Mutex<HashMap<StockId, InventoryStock>>,
    }

    impl MockStockRepository {
        fn new() -> Self {
            Self {
                stocks: Mutex::new(HashMap::new()),
            }
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
            store_id: StoreId,
            product_id: ProductId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.values().find(|s| {
                s.store_id() == store_id && s.product_id() == Some(product_id)
            }).cloned())
        }

        async fn find_by_store_and_variant(
            &self,
            store_id: StoreId,
            variant_id: VariantId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.values().find(|s| {
                s.store_id() == store_id && s.variant_id() == Some(variant_id)
            }).cloned())
        }

        async fn update_with_version(
            &self,
            _stock: &InventoryStock,
            _expected_version: i32,
        ) -> Result<(), InventoryError> {
            unimplemented!()
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

        async fn find_all(&self) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_all_low_stock(&self) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_low_stock_by_store(&self, _store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }
    }

    struct MockProductRepository {
        products: Mutex<HashMap<ProductId, Product>>,
        variants: Mutex<HashMap<VariantId, ProductVariant>>,
    }

    impl MockProductRepository {
        fn new() -> Self {
            Self {
                products: Mutex::new(HashMap::new()),
                variants: Mutex::new(HashMap::new()),
            }
        }

        fn add_product(&self, product: Product) {
            let mut products = self.products.lock().unwrap();
            products.insert(product.id(), product);
        }

        fn add_variant(&self, variant: ProductVariant) {
            let mut variants = self.variants.lock().unwrap();
            variants.insert(variant.id(), variant);
        }
    }

    #[async_trait]
    impl ProductRepository for MockProductRepository {
        async fn save(&self, _product: &Product) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_by_id(&self, id: ProductId) -> Result<Option<Product>, InventoryError> {
            let products = self.products.lock().unwrap();
            Ok(products.get(&id).cloned())
        }

        async fn find_by_sku(&self, _sku: &Sku) -> Result<Option<Product>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_barcode(
            &self,
            _barcode: &Barcode,
        ) -> Result<Option<Product>, InventoryError> {
            unimplemented!()
        }

        async fn update(&self, _product: &Product) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: ProductId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_active(&self) -> Result<Vec<Product>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_category(
            &self,
            _category_id: crate::domain::value_objects::CategoryId,
        ) -> Result<Vec<Product>, InventoryError> {
            unimplemented!()
        }

        async fn save_variant(&self, _variant: &ProductVariant) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_id(
            &self,
            id: VariantId,
        ) -> Result<Option<ProductVariant>, InventoryError> {
            let variants = self.variants.lock().unwrap();
            Ok(variants.get(&id).cloned())
        }

        async fn find_variant_by_sku(
            &self,
            _sku: &Sku,
        ) -> Result<Option<ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_barcode(
            &self,
            _barcode: &Barcode,
        ) -> Result<Option<ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn find_variants_by_product(
            &self,
            _product_id: ProductId,
        ) -> Result<Vec<ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn update_variant(&self, _variant: &ProductVariant) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete_variant(&self, _id: VariantId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn count_variants(&self, _product_id: ProductId) -> Result<u32, InventoryError> {
            unimplemented!()
        }

        async fn find_paginated(
            &self,
            _category_id: Option<crate::domain::value_objects::CategoryId>,
            _is_active: Option<bool>,
            _search: Option<&str>,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<Product>, i64), InventoryError> {
            unimplemented!()
        }

        async fn count_filtered(
            &self,
            _category_id: Option<crate::domain::value_objects::CategoryId>,
            _is_active: Option<bool>,
            _search: Option<&str>,
        ) -> Result<i64, InventoryError> {
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

        async fn count_by_stock_id(&self, _stock_id: StockId) -> Result<i64, InventoryError> {
            unimplemented!()
        }

        async fn find_by_stock_id_and_date_range(
            &self,
            _stock_id: StockId,
            _from_date: Option<chrono::DateTime<chrono::Utc>>,
            _to_date: Option<chrono::DateTime<chrono::Utc>>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            unimplemented!()
        }

        async fn count_by_stock_id_and_date_range(
            &self,
            _stock_id: StockId,
            _from_date: Option<chrono::DateTime<chrono::Utc>>,
            _to_date: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<i64, InventoryError> {
            unimplemented!()
        }

        async fn find_with_filters(
            &self,
            _query: &crate::domain::repositories::MovementQuery,
        ) -> Result<Vec<InventoryMovement>, InventoryError> {
            unimplemented!()
        }

        async fn count_with_filters(&self, _query: &crate::domain::repositories::MovementQuery) -> Result<i64, InventoryError> {
            unimplemented!()
        }

        async fn calculate_weighted_average_cost(
            &self,
            _stock_id: StockId,
        ) -> Result<Option<rust_decimal::Decimal>, InventoryError> {
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
    async fn test_initialize_stock_for_product() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create a product
        let product = Product::create("Test Product".to_string(), UnitOfMeasure::Unit, None);
        let product_id = product.id();
        product_repo.add_product(product);

        let use_case = InitializeStockUseCase::new(
            stock_repo.clone(),
            product_repo,
            movement_repo,
            audit_repo,
        );

        let store_id = StoreId::new();
        let command = InitializeStockCommand {
            store_id: store_id.into_uuid(),
            product_id: Some(product_id.into_uuid()),
            variant_id: None,
            initial_quantity: dec!(0),
            min_stock_level: dec!(10),
            max_stock_level: Some(dec!(100)),
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.store_id, store_id.into_uuid());
        assert_eq!(response.product_id, Some(product_id.into_uuid()));
        assert_eq!(response.quantity, dec!(0));
        assert_eq!(response.min_stock_level, dec!(10));
        assert_eq!(response.max_stock_level, Some(dec!(100)));
    }

    #[tokio::test]
    async fn test_initialize_stock_with_initial_quantity() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        let product = Product::create("Test Product".to_string(), UnitOfMeasure::Unit, None);
        let product_id = product.id();
        product_repo.add_product(product);

        let use_case = InitializeStockUseCase::new(
            stock_repo,
            product_repo,
            movement_repo.clone(),
            audit_repo,
        );

        let store_id = StoreId::new();
        let command = InitializeStockCommand {
            store_id: store_id.into_uuid(),
            product_id: Some(product_id.into_uuid()),
            variant_id: None,
            initial_quantity: dec!(50),
            min_stock_level: dec!(0),
            max_stock_level: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.quantity, dec!(50));

        // Verify movement was created
        let movements = movement_repo.movements.lock().unwrap();
        assert_eq!(movements.len(), 1);
        assert_eq!(movements[0].quantity(), dec!(50));
    }

    #[tokio::test]
    async fn test_initialize_stock_product_not_found() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        let use_case = InitializeStockUseCase::new(stock_repo, product_repo, movement_repo, audit_repo);

        let store_id = StoreId::new();
        let non_existent_product_id = ProductId::new();
        let command = InitializeStockCommand {
            store_id: store_id.into_uuid(),
            product_id: Some(non_existent_product_id.into_uuid()),
            variant_id: None,
            initial_quantity: dec!(0),
            min_stock_level: dec!(0),
            max_stock_level: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::ProductNotFound(_))));
    }

    #[tokio::test]
    async fn test_initialize_stock_invalid_constraint_both() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        let use_case = InitializeStockUseCase::new(stock_repo, product_repo, movement_repo, audit_repo);

        let store_id = StoreId::new();
        let command = InitializeStockCommand {
            store_id: store_id.into_uuid(),
            product_id: Some(ProductId::new().into_uuid()),
            variant_id: Some(VariantId::new().into_uuid()),
            initial_quantity: dec!(0),
            min_stock_level: dec!(0),
            max_stock_level: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[tokio::test]
    async fn test_initialize_stock_invalid_constraint_neither() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        let use_case = InitializeStockUseCase::new(stock_repo, product_repo, movement_repo, audit_repo);

        let store_id = StoreId::new();
        let command = InitializeStockCommand {
            store_id: store_id.into_uuid(),
            product_id: None,
            variant_id: None,
            initial_quantity: dec!(0),
            min_stock_level: dec!(0),
            max_stock_level: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::InvalidProductVariantConstraint)));
    }

    #[tokio::test]
    async fn test_initialize_stock_already_exists() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository::new());
        let movement_repo = Arc::new(MockMovementRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        let product = Product::create("Test Product".to_string(), UnitOfMeasure::Unit, None);
        let product_id = product.id();
        product_repo.add_product(product);

        // Pre-create stock for this product in this store
        let store_id = StoreId::new();
        let existing_stock = InventoryStock::create_for_product(store_id, product_id).unwrap();
        stock_repo.save(&existing_stock).await.unwrap();

        let use_case = InitializeStockUseCase::new(stock_repo, product_repo, movement_repo, audit_repo);

        let command = InitializeStockCommand {
            store_id: store_id.into_uuid(),
            product_id: Some(product_id.into_uuid()),
            variant_id: None,
            initial_quantity: dec!(0),
            min_stock_level: dec!(0),
            max_stock_level: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::StockAlreadyExists { .. })));
    }
}
