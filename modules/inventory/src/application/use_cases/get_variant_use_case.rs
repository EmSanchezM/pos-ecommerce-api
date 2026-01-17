// GetVariantUseCase - retrieves a specific variant

use std::sync::Arc;

use crate::application::dtos::responses::VariantResponse;
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::{ProductId, VariantId};
use crate::InventoryError;

/// Use case for retrieving a specific product variant
pub struct GetVariantUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> GetVariantUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of GetVariantUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to get a specific variant
    ///
    /// # Arguments
    /// * `product_id` - The parent product's UUID
    /// * `variant_id` - The variant's UUID
    ///
    /// # Returns
    /// VariantResponse on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    /// * `InventoryError::VariantNotFound` - If variant doesn't exist or doesn't belong to product
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
        variant_id: uuid::Uuid,
    ) -> Result<VariantResponse, InventoryError> {
        let prod_id = ProductId::from_uuid(product_id);
        let var_id = VariantId::from_uuid(variant_id);

        // Validate product exists
        let product = self
            .product_repo
            .find_by_id(prod_id)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Find variant
        let variant = self
            .product_repo
            .find_variant_by_id(var_id)
            .await?
            .ok_or(InventoryError::VariantNotFound(variant_id))?;

        // Validate variant belongs to the specified product
        if variant.product_id() != prod_id {
            return Err(InventoryError::VariantNotFound(variant_id));
        }

        // Build response
        let effective_price = variant.price().unwrap_or(product.base_price());
        let effective_cost = variant.cost_price().unwrap_or(product.cost_price());

        Ok(VariantResponse {
            id: variant.id().into_uuid(),
            product_id: variant.product_id().into_uuid(),
            sku: variant.sku().as_str().to_string(),
            barcode: variant.barcode().map(|b| b.as_str().to_string()),
            name: variant.name().to_string(),
            variant_attributes: variant.variant_attributes().clone(),
            price: variant.price(),
            cost_price: variant.cost_price(),
            effective_price,
            effective_cost,
            is_active: variant.is_active(),
            created_at: variant.created_at(),
            updated_at: variant.updated_at(),
        })
    }
}
