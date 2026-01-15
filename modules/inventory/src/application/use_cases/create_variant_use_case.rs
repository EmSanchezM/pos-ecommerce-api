// CreateVariantUseCase - creates a new product variant
//
// Requirements: 1.7, 2.1, 2.6

use std::sync::Arc;

use crate::application::dtos::commands::CreateVariantCommand;
use crate::application::dtos::responses::VariantResponse;
use crate::domain::entities::ProductVariant;
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::{Barcode, ProductId};
use crate::InventoryError;

/// Use case for creating a new product variant
///
/// Validates that the product has variants enabled, auto-generates variant SKU,
/// and validates barcode uniqueness.
///
/// Requirements: 1.7, 2.1, 2.6
pub struct CreateVariantUseCase<P>
where
    P: ProductRepository,
{
    product_repo: Arc<P>,
}

impl<P> CreateVariantUseCase<P>
where
    P: ProductRepository,
{
    /// Creates a new instance of CreateVariantUseCase
    pub fn new(product_repo: Arc<P>) -> Self {
        Self { product_repo }
    }

    /// Executes the use case to create a new product variant
    ///
    /// # Arguments
    /// * `command` - The create variant command containing variant data
    ///
    /// # Returns
    /// VariantResponse on success
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If product_id doesn't exist
    /// * `InventoryError::VariantsNotEnabled` - If product has_variants is false
    /// * `InventoryError::DuplicateBarcode` - If barcode already exists
    /// * `InventoryError::InvalidBarcode` - If barcode format is invalid
    pub async fn execute(
        &self,
        command: CreateVariantCommand,
    ) -> Result<VariantResponse, InventoryError> {
        let product_id = ProductId::from_uuid(command.product_id);

        // Validate product exists
        let product = self
            .product_repo
            .find_by_id(product_id)
            .await?
            .ok_or(InventoryError::ProductNotFound(command.product_id))?;

        // Validate product has_variants is true (Requirement 1.7, 2.1)
        if !product.has_variants() {
            return Err(InventoryError::VariantsNotEnabled);
        }

        // Validate barcode uniqueness if provided (Requirement 2.6)
        let barcode = if let Some(barcode_str) = &command.barcode {
            let barcode = Barcode::new(barcode_str)?;
            
            // Check against products
            if self
                .product_repo
                .find_by_barcode(&barcode)
                .await?
                .is_some()
            {
                return Err(InventoryError::DuplicateBarcode(barcode_str.clone()));
            }
            
            // Check against variants
            if self
                .product_repo
                .find_variant_by_barcode(&barcode)
                .await?
                .is_some()
            {
                return Err(InventoryError::DuplicateBarcode(barcode_str.clone()));
            }
            
            Some(barcode)
        } else {
            None
        };

        // Get the next variant index for SKU generation
        let variant_count = self.product_repo.count_variants(product_id).await?;
        let variant_index = variant_count + 1;

        // Create variant entity with auto-generated SKU (Requirement 2.1)
        let mut variant = ProductVariant::create(
            product_id,
            product.sku(),
            variant_index,
            command.name,
        );

        // Apply optional fields
        if let Some(barcode) = barcode {
            variant.set_barcode(Some(barcode));
        }
        variant.set_variant_attributes(command.variant_attributes);
        if let Some(price) = command.price {
            variant.set_price(Some(price));
        }
        if let Some(cost_price) = command.cost_price {
            variant.set_cost_price(Some(cost_price));
        }

        // Save to repository
        self.product_repo.save_variant(&variant).await?;

        // Calculate effective prices for response
        let effective_price = variant.effective_price(product.base_price());
        let effective_cost = variant.effective_cost(product.cost_price());

        // Convert to response
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::domain::entities::Product;
    use crate::domain::value_objects::{CategoryId, Sku, UnitOfMeasure, VariantId};

    // Mock repository for testing
    struct MockProductRepository {
        products: Mutex<HashMap<ProductId, Product>>,
        variants: Mutex<HashMap<VariantId, ProductVariant>>,
        product_variants: Mutex<HashMap<ProductId, Vec<VariantId>>>,
        barcodes: Mutex<HashMap<String, ProductId>>,
        variant_barcodes: Mutex<HashMap<String, VariantId>>,
    }

    impl MockProductRepository {
        fn new() -> Self {
            Self {
                products: Mutex::new(HashMap::new()),
                variants: Mutex::new(HashMap::new()),
                product_variants: Mutex::new(HashMap::new()),
                barcodes: Mutex::new(HashMap::new()),
                variant_barcodes: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ProductRepository for MockProductRepository {
        async fn save(&self, product: &Product) -> Result<(), InventoryError> {
            let mut products = self.products.lock().unwrap();
            products.insert(product.id(), product.clone());
            if let Some(barcode) = product.barcode() {
                let mut barcodes = self.barcodes.lock().unwrap();
                barcodes.insert(barcode.as_str().to_string(), product.id());
            }
            Ok(())
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
            barcode: &Barcode,
        ) -> Result<Option<Product>, InventoryError> {
            let barcodes = self.barcodes.lock().unwrap();
            let products = self.products.lock().unwrap();
            Ok(barcodes
                .get(barcode.as_str())
                .and_then(|id| products.get(id).cloned()))
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
            _category_id: CategoryId,
        ) -> Result<Vec<Product>, InventoryError> {
            unimplemented!()
        }

        async fn save_variant(&self, variant: &ProductVariant) -> Result<(), InventoryError> {
            let mut variants = self.variants.lock().unwrap();
            let mut product_variants = self.product_variants.lock().unwrap();
            
            variants.insert(variant.id(), variant.clone());
            product_variants
                .entry(variant.product_id())
                .or_insert_with(Vec::new)
                .push(variant.id());
            
            if let Some(barcode) = variant.barcode() {
                let mut variant_barcodes = self.variant_barcodes.lock().unwrap();
                variant_barcodes.insert(barcode.as_str().to_string(), variant.id());
            }
            
            Ok(())
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
            barcode: &Barcode,
        ) -> Result<Option<ProductVariant>, InventoryError> {
            let variant_barcodes = self.variant_barcodes.lock().unwrap();
            let variants = self.variants.lock().unwrap();
            Ok(variant_barcodes
                .get(barcode.as_str())
                .and_then(|id| variants.get(id).cloned()))
        }

        async fn find_variants_by_product(
            &self,
            product_id: ProductId,
        ) -> Result<Vec<ProductVariant>, InventoryError> {
            let product_variants = self.product_variants.lock().unwrap();
            let variants = self.variants.lock().unwrap();
            
            Ok(product_variants
                .get(&product_id)
                .map(|variant_ids| {
                    variant_ids
                        .iter()
                        .filter_map(|id| variants.get(id).cloned())
                        .collect()
                })
                .unwrap_or_default())
        }

        async fn update_variant(&self, _variant: &ProductVariant) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete_variant(&self, _id: VariantId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn count_variants(&self, product_id: ProductId) -> Result<u32, InventoryError> {
            let product_variants = self.product_variants.lock().unwrap();
            Ok(product_variants
                .get(&product_id)
                .map(|v| v.len() as u32)
                .unwrap_or(0))
        }
    }

    #[tokio::test]
    async fn test_create_variant_basic() {
        let repo = Arc::new(MockProductRepository::new());
        let use_case = CreateVariantUseCase::new(repo.clone());

        // Create a product with variants enabled
        let mut product = Product::create("T-Shirt".to_string(), UnitOfMeasure::Unit, None);
        product.set_has_variants(true);
        product.set_base_price(dec!(29.99));
        product.set_cost_price(dec!(15.00));
        repo.save(&product).await.unwrap();

        let command = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Red - Large".to_string(),
            variant_attributes: serde_json::json!({
                "color": "red",
                "size": "L"
            }),
            price: Some(dec!(34.99)),
            cost_price: None,
            barcode: None,
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.name, "Red - Large");
        assert_eq!(response.product_id, product.id().into_uuid());
        assert!(response.sku.ends_with("-V001"));
        assert_eq!(response.price, Some(dec!(34.99)));
        assert_eq!(response.effective_price, dec!(34.99));
        assert_eq!(response.effective_cost, dec!(15.00)); // Uses product cost
    }

    #[tokio::test]
    async fn test_create_multiple_variants() {
        let repo = Arc::new(MockProductRepository::new());
        let use_case = CreateVariantUseCase::new(repo.clone());

        // Create a product with variants enabled
        let mut product = Product::create("Shoes".to_string(), UnitOfMeasure::Unit, None);
        product.set_has_variants(true);
        repo.save(&product).await.unwrap();

        // Create first variant
        let command1 = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Size 8".to_string(),
            variant_attributes: serde_json::json!({"size": 8}),
            price: None,
            cost_price: None,
            barcode: None,
        };
        let result1 = use_case.execute(command1).await;
        assert!(result1.is_ok());
        assert!(result1.unwrap().sku.ends_with("-V001"));

        // Create second variant
        let command2 = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Size 9".to_string(),
            variant_attributes: serde_json::json!({"size": 9}),
            price: None,
            cost_price: None,
            barcode: None,
        };
        let result2 = use_case.execute(command2).await;
        assert!(result2.is_ok());
        assert!(result2.unwrap().sku.ends_with("-V002"));
    }

    #[tokio::test]
    async fn test_variants_not_enabled_error() {
        let repo = Arc::new(MockProductRepository::new());
        let use_case = CreateVariantUseCase::new(repo.clone());

        // Create a product WITHOUT variants enabled
        let product = Product::create("Simple Product".to_string(), UnitOfMeasure::Unit, None);
        repo.save(&product).await.unwrap();

        let command = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Variant".to_string(),
            variant_attributes: serde_json::json!({}),
            price: None,
            cost_price: None,
            barcode: None,
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(InventoryError::VariantsNotEnabled)));
    }

    #[tokio::test]
    async fn test_product_not_found_error() {
        let repo = Arc::new(MockProductRepository::new());
        let use_case = CreateVariantUseCase::new(repo);

        let non_existent_product = ProductId::new().into_uuid();
        let command = CreateVariantCommand {
            product_id: non_existent_product,
            name: "Variant".to_string(),
            variant_attributes: serde_json::json!({}),
            price: None,
            cost_price: None,
            barcode: None,
        };

        let result = use_case.execute(command).await;
        assert!(matches!(result, Err(InventoryError::ProductNotFound(_))));
    }

    #[tokio::test]
    async fn test_duplicate_barcode_error() {
        let repo = Arc::new(MockProductRepository::new());
        let use_case = CreateVariantUseCase::new(repo.clone());

        // Create a product with variants enabled
        let mut product = Product::create("Product".to_string(), UnitOfMeasure::Unit, None);
        product.set_has_variants(true);
        repo.save(&product).await.unwrap();

        // Create first variant with barcode
        let command1 = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Variant 1".to_string(),
            variant_attributes: serde_json::json!({}),
            price: None,
            cost_price: None,
            barcode: Some("1234567890123".to_string()),
        };
        use_case.execute(command1).await.unwrap();

        // Try to create second variant with same barcode
        let command2 = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Variant 2".to_string(),
            variant_attributes: serde_json::json!({}),
            price: None,
            cost_price: None,
            barcode: Some("1234567890123".to_string()),
        };

        let result = use_case.execute(command2).await;
        assert!(matches!(result, Err(InventoryError::DuplicateBarcode(_))));
    }

    #[tokio::test]
    async fn test_effective_prices() {
        let repo = Arc::new(MockProductRepository::new());
        let use_case = CreateVariantUseCase::new(repo.clone());

        // Create a product with base prices
        let mut product = Product::create("Product".to_string(), UnitOfMeasure::Unit, None);
        product.set_has_variants(true);
        product.set_base_price(dec!(100.00));
        product.set_cost_price(dec!(50.00));
        repo.save(&product).await.unwrap();

        // Create variant without price overrides
        let command1 = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Standard".to_string(),
            variant_attributes: serde_json::json!({}),
            price: None,
            cost_price: None,
            barcode: None,
        };
        let result1 = use_case.execute(command1).await.unwrap();
        assert_eq!(result1.effective_price, dec!(100.00));
        assert_eq!(result1.effective_cost, dec!(50.00));

        // Create variant with price overrides
        let command2 = CreateVariantCommand {
            product_id: product.id().into_uuid(),
            name: "Premium".to_string(),
            variant_attributes: serde_json::json!({}),
            price: Some(dec!(150.00)),
            cost_price: Some(dec!(75.00)),
            barcode: None,
        };
        let result2 = use_case.execute(command2).await.unwrap();
        assert_eq!(result2.effective_price, dec!(150.00));
        assert_eq!(result2.effective_cost, dec!(75.00));
    }
}
