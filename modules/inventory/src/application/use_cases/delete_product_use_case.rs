// DeleteProductUseCase - soft deletes a product by deactivating it

use std::sync::Arc;

use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::ProductId;
use crate::InventoryError;
use identity::domain::entities::AuditEntry;
use identity::domain::repositories::AuditRepository;
use identity::domain::value_objects::UserId;

/// Use case for soft deleting a product (sets is_active to false)
pub struct DeleteProductUseCase<P, A>
where
    P: ProductRepository,
    A: AuditRepository,
{
    product_repo: Arc<P>,
    audit_repo: Arc<A>,
}

impl<P, A> DeleteProductUseCase<P, A>
where
    P: ProductRepository,
    A: AuditRepository,
{
    /// Creates a new instance of DeleteProductUseCase
    pub fn new(product_repo: Arc<P>, audit_repo: Arc<A>) -> Self {
        Self {
            product_repo,
            audit_repo,
        }
    }

    /// Executes the use case to soft delete a product
    ///
    /// # Arguments
    /// * `product_id` - The product's UUID
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
        actor_id: UserId,
    ) -> Result<(), InventoryError> {
        let id = ProductId::from_uuid(product_id);

        // Find existing product
        let mut product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Soft delete by deactivating
        product.deactivate();

        // Save updated product
        self.product_repo.update(&product).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_delete(
            "product",
            product.id().into_uuid(),
            &product,
            actor_id,
        );
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|_| InventoryError::NotImplemented)?;

        Ok(())
    }
}
