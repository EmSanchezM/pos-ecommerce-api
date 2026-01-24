// ListVariantsUseCase - lists all variants for a product

use std::sync::Arc;

use crate::application::dtos::responses::{ListResponse, VariantResponse};
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::ProductId;
use crate::InventoryError;

/// Use case for listing all variants of a product
pub struct ListVariantsUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> ListVariantsUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of ListVariantsUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to list all variants for a product
    ///
    /// # Arguments
    /// * `product_id` - The product's UUID
    ///
    /// # Returns
    /// ListResponse containing variants on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
    ) -> Result<ListResponse<VariantResponse>, InventoryError> {
        let id = ProductId::from_uuid(product_id);

        // Validate product exists
        let product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Get all variants for this product
        let variants = self.product_repo.find_variants_by_product(id).await?;

        // Convert to response DTOs
        let variant_responses: Vec<VariantResponse> = variants
            .into_iter()
            .map(|v| {
                let effective_price = v.price().unwrap_or(product.base_price());
                let effective_cost = v.cost_price().unwrap_or(product.cost_price());
                VariantResponse {
                    id: v.id().into_uuid(),
                    product_id: v.product_id().into_uuid(),
                    sku: v.sku().as_str().to_string(),
                    barcode: v.barcode().map(|b| b.as_str().to_string()),
                    name: v.name().to_string(),
                    variant_attributes: v.variant_attributes().clone(),
                    price: v.price(),
                    cost_price: v.cost_price(),
                    effective_price,
                    effective_cost,
                    is_active: v.is_active(),
                    created_at: v.created_at(),
                    updated_at: v.updated_at(),
                }
            })
            .collect();

        Ok(ListResponse::new(variant_responses))
    }
}
