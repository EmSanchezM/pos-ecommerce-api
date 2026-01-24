// UpdateStockLevelsUseCase - updates min/max stock level thresholds

use std::sync::Arc;

use crate::application::dtos::commands::UpdateStockLevelsCommand;
use crate::application::dtos::responses::StockResponse;
use crate::domain::repositories::InventoryStockRepository;
use crate::domain::value_objects::StockId;
use crate::InventoryError;
use identity::domain::entities::AuditEntry;
use identity::domain::repositories::AuditRepository;
use identity::domain::value_objects::UserId;

/// Use case for updating stock level thresholds (min/max).
///
/// This allows updating the minimum and maximum stock levels for alerting
/// and inventory management without changing the actual quantity.
pub struct UpdateStockLevelsUseCase<S, A>
where
    S: InventoryStockRepository,
    A: AuditRepository,
{
    stock_repo: Arc<S>,
    audit_repo: Arc<A>,
}

impl<S, A> UpdateStockLevelsUseCase<S, A>
where
    S: InventoryStockRepository,
    A: AuditRepository,
{
    /// Creates a new instance of UpdateStockLevelsUseCase
    pub fn new(stock_repo: Arc<S>, audit_repo: Arc<A>) -> Self {
        Self {
            stock_repo,
            audit_repo,
        }
    }

    /// Executes the use case to update stock levels
    ///
    /// # Arguments
    /// * `command` - The update stock levels command
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// StockResponse on success
    ///
    /// # Errors
    /// * `InventoryError::StockNotFound` - If stock record doesn't exist
    /// * `InventoryError::OptimisticLockError` - If version mismatch
    pub async fn execute(
        &self,
        command: UpdateStockLevelsCommand,
        actor_id: UserId,
    ) -> Result<StockResponse, InventoryError> {
        let stock_id = StockId::from_uuid(command.stock_id);

        // 1. Find existing stock record
        let mut stock = self
            .stock_repo
            .find_by_id(stock_id)
            .await?
            .ok_or(InventoryError::StockNotFound(command.stock_id))?;

        let old_state = stock.clone();

        // 2. Update levels
        stock.set_min_stock_level(command.min_stock_level);
        stock.set_max_stock_level(command.max_stock_level);

        // 3. Save with optimistic locking
        self.stock_repo
            .update_with_version(&stock, command.expected_version)
            .await?;

        // 4. Create audit entry
        let audit_entry = AuditEntry::for_update(
            "inventory_stock",
            stock.id().into_uuid(),
            &old_state,
            &stock,
            actor_id,
        );
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|_| InventoryError::NotImplemented)?;

        // 5. Return response
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
