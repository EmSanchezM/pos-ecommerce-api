// UpdateVariantUseCase - updates an existing product variant

use std::sync::Arc;

use crate::application::dtos::commands::UpdateVariantCommand;
use crate::application::dtos::responses::VariantResponse;
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::{Barcode, ProductId, VariantId};
use crate::InventoryError;

/// Use case for updating an existing product variant
pub struct UpdateVariantUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> UpdateVariantUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of UpdateVariantUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to update a variant
    ///
    /// # Arguments
    /// * `product_id` - The parent product's UUID
    /// * `variant_id` - The variant's UUID
    /// * `command` - The update command containing fields to update
    ///
    /// # Returns
    /// VariantResponse on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    /// * `InventoryError::VariantNotFound` - If variant doesn't exist or doesn't belong to product
    /// * `InventoryError::DuplicateBarcode` - If barcode is used by another product/variant
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
        variant_id: uuid::Uuid,
        command: UpdateVariantCommand,
    ) -> Result<VariantResponse, InventoryError> {
        let prod_id = ProductId::from_uuid(product_id);
        let var_id = VariantId::from_uuid(variant_id);

        // Validate product exists
        let product = self
            .product_repo
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

        // Apply updates
        if let Some(name) = command.name {
            variant.set_name(name);
        }

        if let Some(variant_attributes) = command.variant_attributes {
            variant.set_variant_attributes(variant_attributes);
        }

        if let Some(price) = command.price {
            variant.set_price(Some(price));
        }

        if let Some(cost_price) = command.cost_price {
            variant.set_cost_price(Some(cost_price));
        }

        if let Some(is_active) = command.is_active {
            if is_active {
                variant.activate();
            } else {
                variant.deactivate();
            }
        }

        // Handle barcode update with uniqueness check
        if let Some(barcode_str) = command.barcode {
            let barcode = Barcode::new(&barcode_str)?;

            // Check against products
            if let Some(_existing_product) = self.product_repo.find_by_barcode(&barcode).await? {
                return Err(InventoryError::DuplicateBarcode(barcode_str));
            }

            // Check against other variants
            if let Some(existing_variant) =
                self.product_repo.find_variant_by_barcode(&barcode).await?
                && existing_variant.id() != variant.id() {
                    return Err(InventoryError::DuplicateBarcode(barcode_str));
                }

            variant.set_barcode(Some(barcode));
        }

        // Save updated variant
        self.product_repo.update_variant(&variant).await?;

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
