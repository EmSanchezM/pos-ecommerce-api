// GetProductUseCase - retrieves a product with its variants

use std::sync::Arc;

use crate::application::dtos::responses::{ProductDetailResponse, VariantResponse};
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::ProductId;
use crate::InventoryError;

/// Use case for retrieving a product with full details including variants
pub struct GetProductUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> GetProductUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of GetProductUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to get a product by ID
    ///
    /// # Arguments
    /// * `product_id` - The product's UUID
    ///
    /// # Returns
    /// ProductDetailResponse with variants on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
    ) -> Result<ProductDetailResponse, InventoryError> {
        let id = ProductId::from_uuid(product_id);

        // Find product
        let product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Find variants
        let variants = self.product_repo.find_variants_by_product(id).await?;

        // Convert variants to response DTOs
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

        // Build response
        Ok(ProductDetailResponse {
            id: product.id().into_uuid(),
            sku: product.sku().as_str().to_string(),
            barcode: product.barcode().map(|b| b.as_str().to_string()),
            name: product.name().to_string(),
            description: product.description().map(|s| s.to_string()),
            category_id: product.category_id().map(|id| id.into_uuid()),
            category: None, // Category details can be loaded separately if needed
            brand: product.brand().map(|s| s.to_string()),
            unit_of_measure: product.unit_of_measure().to_string(),
            base_price: product.base_price(),
            cost_price: product.cost_price(),
            currency: product.currency().as_str().to_string(),
            is_perishable: product.is_perishable(),
            is_trackable: product.is_trackable(),
            has_variants: product.has_variants(),
            tax_rate: product.tax_rate(),
            tax_included: product.tax_included(),
            attributes: product.attributes().clone(),
            is_active: product.is_active(),
            variants: variant_responses,
            created_at: product.created_at(),
            updated_at: product.updated_at(),
        })
    }
}
