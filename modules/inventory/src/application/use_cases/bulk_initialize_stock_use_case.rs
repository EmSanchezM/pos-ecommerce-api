// BulkInitializeStockUseCase - initializes stock for multiple products at once

use rust_decimal::Decimal;
use std::sync::Arc;

use crate::application::dtos::commands::{BulkInitializeStockCommand, BulkInitializeStockItem};
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
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Result of bulk stock initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkInitializeStockResult {
    /// Successfully initialized stock records
    pub successful: Vec<StockResponse>,
    /// Failed items with error messages
    pub failed: Vec<BulkInitializeStockError>,
    /// Total items processed
    pub total_processed: usize,
    /// Total successful
    pub total_successful: usize,
    /// Total failed
    pub total_failed: usize,
}

/// Error details for a failed bulk initialization item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkInitializeStockError {
    /// Index of the item in the original request
    pub index: usize,
    /// Product ID if provided
    pub product_id: Option<Uuid>,
    /// Variant ID if provided
    pub variant_id: Option<Uuid>,
    /// Error message
    pub error: String,
}

/// Use case for initializing stock for multiple products/variants at once.
///
/// This is useful for initial store setup or bulk imports.
/// Processes all items and returns both successes and failures.
pub struct BulkInitializeStockUseCase<S, P, M, A>
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

impl<S, P, M, A> BulkInitializeStockUseCase<S, P, M, A>
where
    S: InventoryStockRepository,
    P: ProductRepository,
    M: InventoryMovementRepository,
    A: AuditRepository,
{
    /// Creates a new instance of BulkInitializeStockUseCase
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

    /// Executes the use case to initialize stock for multiple items
    ///
    /// # Arguments
    /// * `command` - The bulk initialize stock command
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// BulkInitializeStockResult with successes and failures
    pub async fn execute(
        &self,
        command: BulkInitializeStockCommand,
        actor_id: UserId,
    ) -> Result<BulkInitializeStockResult, InventoryError> {
        let store_id = StoreId::from_uuid(command.store_id);
        let mut successful = Vec::new();
        let mut failed = Vec::new();

        for (index, item) in command.items.into_iter().enumerate() {
            match self
                .process_item(store_id, item.clone(), actor_id, index)
                .await
            {
                Ok(response) => successful.push(response),
                Err(error) => failed.push(BulkInitializeStockError {
                    index,
                    product_id: item.product_id,
                    variant_id: item.variant_id,
                    error: error.to_string(),
                }),
            }
        }

        let total_processed = successful.len() + failed.len();
        let total_successful = successful.len();
        let total_failed = failed.len();

        Ok(BulkInitializeStockResult {
            successful,
            failed,
            total_processed,
            total_successful,
            total_failed,
        })
    }

    async fn process_item(
        &self,
        store_id: StoreId,
        item: BulkInitializeStockItem,
        actor_id: UserId,
        _index: usize,
    ) -> Result<StockResponse, InventoryError> {
        // 1. Validate XOR constraint
        let (product_id, variant_id) = match (item.product_id, item.variant_id) {
            (Some(pid), None) => (Some(ProductId::from_uuid(pid)), None),
            (None, Some(vid)) => (None, Some(VariantId::from_uuid(vid))),
            _ => return Err(InventoryError::InvalidProductVariantConstraint),
        };

        // 2. Validate product/variant exists and check for duplicates
        if let Some(pid) = product_id {
            if self.product_repo.find_by_id(pid).await?.is_none() {
                return Err(InventoryError::ProductNotFound(pid.into_uuid()));
            }

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

        // 3. Create stock entity
        let mut stock = if let Some(pid) = product_id {
            InventoryStock::create_for_product(store_id, pid)?
        } else {
            InventoryStock::create_for_variant(store_id, variant_id.unwrap())?
        };

        // 4. Apply settings
        stock.set_min_stock_level(item.min_stock_level);
        stock.set_max_stock_level(item.max_stock_level);

        if item.initial_quantity > Decimal::ZERO {
            stock.adjust_quantity(item.initial_quantity)?;
        }

        // 5. Save stock record
        self.stock_repo.save(&stock).await?;

        // 6. Create initial movement if there's initial quantity
        if item.initial_quantity > Decimal::ZERO {
            let movement = InventoryMovement::create(
                stock.id(),
                MovementType::In,
                Some("initial_stock".to_string()),
                item.initial_quantity,
                None,
                Currency::hnl(),
                stock.quantity(),
                Some("bulk_initialization".to_string()),
                Some(stock.id().into_uuid()),
                actor_id,
                Some("Bulk stock initialization".to_string()),
            );
            self.movement_repo.save(&movement).await?;
        }

        // 7. Create audit entry
        let audit_entry = AuditEntry::for_create(
            "inventory_stock",
            stock.id().into_uuid(),
            &stock,
            actor_id,
        );
        let _ = self.audit_repo.save(&audit_entry).await;

        // 8. Return response
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
