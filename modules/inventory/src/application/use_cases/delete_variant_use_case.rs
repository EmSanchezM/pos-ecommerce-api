// DeleteVariantUseCase - soft deletes a product variant by deactivating it

use std::sync::Arc;

use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::{ProductId, VariantId};
use crate::InventoryError;

/// Use case for soft deleting a product variant (sets is_active to false)
pub struct DeleteVariantUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> DeleteVariantUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of DeleteVariantUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to soft delete a variant
    ///
    /// # Arguments
    /// * `product_id` - The parent product's UUID
    /// * `variant_id` - The variant's UUID
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    /// * `InventoryError::VariantNotFound` - If variant doesn't exist or doesn't belong to product
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
        variant_id: uuid::Uuid,
    ) -> Result<(), InventoryError> {
        let prod_id = ProductId::from_uuid(product_id);
        let var_id = VariantId::from_uuid(variant_id);

        // Validate product exists
        self.product_repo
            .find_by_id(prod_id)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Find existing variant
        let mut variant = self
            .product_repo
            .find_variant_by_id(var_id)
            .await?
            .ok_or(InventoryError::VariantNotFound(variant_id))?;

        // Validate variant belongs to the specified product
        if variant.product_id() != prod_id {
            return Err(InventoryError::VariantNotFound(variant_id));
        }

        // Soft delete by deactivating
        variant.deactivate();

        // Save updated variant
        self.product_repo.update_variant(&variant).await?;

        Ok(())
    }
}
