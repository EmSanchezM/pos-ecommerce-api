// UpdateProductUseCase - updates an existing product

use std::sync::Arc;

use crate::application::dtos::commands::UpdateProductCommand;
use crate::application::dtos::responses::ProductResponse;
use crate::domain::repositories::{CategoryRepository, ProductRepository};
use crate::domain::value_objects::{Barcode, CategoryId, Currency, ProductId, UnitOfMeasure};
use crate::InventoryError;
use identity::domain::entities::AuditEntry;
use identity::domain::repositories::AuditRepository;
use identity::domain::value_objects::UserId;

/// Use case for updating an existing product
pub struct UpdateProductUseCase<P, C, A>
where
    P: ProductRepository,
    C: CategoryRepository,
    A: AuditRepository,
{
    product_repo: Arc<P>,
    category_repo: Arc<C>,
    audit_repo: Arc<A>,
}

impl<P, C, A> UpdateProductUseCase<P, C, A>
where
    P: ProductRepository,
    C: CategoryRepository,
    A: AuditRepository,
{
    /// Creates a new instance of UpdateProductUseCase
    pub fn new(product_repo: Arc<P>, category_repo: Arc<C>, audit_repo: Arc<A>) -> Self {
        Self {
            product_repo,
            category_repo,
            audit_repo,
        }
    }

    /// Executes the use case to update a product
    ///
    /// # Arguments
    /// * `product_id` - The product's UUID
    /// * `command` - The update command containing fields to update
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// ProductResponse on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product doesn't exist
    /// * `InventoryError::DuplicateBarcode` - If barcode is used by another product
    /// * `InventoryError::CategoryNotFound` - If category doesn't exist
    /// * `InventoryError::InvalidCurrency` - If currency code is invalid
    /// * `InventoryError::InvalidUnitOfMeasure` - If unit of measure is invalid
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
        command: UpdateProductCommand,
        actor_id: UserId,
    ) -> Result<ProductResponse, InventoryError> {
        let id = ProductId::from_uuid(product_id);

        // Find existing product
        let mut product = self
            .product_repo
            .find_by_id(id)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Clone for audit trail (old value)
        let old_product = product.clone();

        // Apply updates
        if let Some(name) = command.name {
            product.set_name(name);
        }

        if let Some(description) = command.description {
            product.set_description(Some(description));
        }

        if let Some(brand) = command.brand {
            product.set_brand(Some(brand));
        }

        if let Some(base_price) = command.base_price {
            product.set_base_price(base_price);
        }

        if let Some(cost_price) = command.cost_price {
            product.set_cost_price(cost_price);
        }

        if let Some(is_perishable) = command.is_perishable {
            product.set_perishable(is_perishable);
        }

        if let Some(is_trackable) = command.is_trackable {
            product.set_trackable(is_trackable);
        }

        if let Some(has_variants) = command.has_variants {
            product.set_has_variants(has_variants);
        }

        if let Some(tax_rate) = command.tax_rate {
            product.set_tax_rate(tax_rate);
        }

        if let Some(tax_included) = command.tax_included {
            product.set_tax_included(tax_included);
        }

        if let Some(attributes) = command.attributes {
            product.set_attributes(attributes);
        }

        if let Some(is_active) = command.is_active {
            if is_active {
                product.activate();
            } else {
                product.deactivate();
            }
        }

        // Handle barcode update with uniqueness check
        if let Some(barcode_str) = command.barcode {
            let barcode = Barcode::new(&barcode_str)?;

            // Check if barcode is already used by another product
            if let Some(existing) = self.product_repo.find_by_barcode(&barcode).await? {
                if existing.id() != product.id() {
                    return Err(InventoryError::DuplicateBarcode(barcode_str));
                }
            }
            product.set_barcode(Some(barcode));
        }

        // Handle category update with existence check
        if let Some(category_uuid) = command.category_id {
            let cat_id = CategoryId::from_uuid(category_uuid);
            if self.category_repo.find_by_id(cat_id).await?.is_none() {
                return Err(InventoryError::CategoryNotFound(category_uuid));
            }
            product.set_category_id(Some(cat_id));
        }

        // Handle currency update
        if let Some(currency_str) = command.currency {
            let currency = Currency::new(&currency_str)?;
            product.set_currency(currency);
        }

        // Handle unit of measure update
        if let Some(uom_str) = command.unit_of_measure {
            let uom: UnitOfMeasure = uom_str.parse()?;
            product.set_unit_of_measure(uom);
        }

        // Save updated product
        self.product_repo.update(&product).await?;

        // Create audit entry
        let audit_entry = AuditEntry::for_update(
            "product",
            product.id().into_uuid(),
            &old_product,
            &product,
            actor_id,
        );
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|_| InventoryError::NotImplemented)?;

        // Build response
        Ok(ProductResponse {
            id: product.id().into_uuid(),
            sku: product.sku().as_str().to_string(),
            barcode: product.barcode().map(|b| b.as_str().to_string()),
            name: product.name().to_string(),
            description: product.description().map(|s| s.to_string()),
            category_id: product.category_id().map(|id| id.into_uuid()),
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
            is_active: product.is_active(),
            created_at: product.created_at(),
            updated_at: product.updated_at(),
        })
    }
}
