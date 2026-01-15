// CreateProductUseCase - creates a new product

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::commands::CreateProductCommand;
use crate::application::dtos::responses::ProductResponse;
use crate::domain::entities::Product;
use crate::domain::repositories::{CategoryRepository, ProductRepository};
use crate::domain::value_objects::{Barcode, CategoryId, Currency, UnitOfMeasure};
use crate::InventoryError;
use identity::domain::entities::AuditEntry;
use identity::domain::repositories::AuditRepository;
use identity::domain::value_objects::UserId;

/// Use case for creating a new product
///
/// Auto-generates SKU, validates barcode uniqueness, validates category existence,
/// and creates an audit entry.
///
/// Requirements: 1.1, 1.2, 1.3
pub struct CreateProductUseCase<P, C, A>
where
    P: ProductRepository,
    C: CategoryRepository,
    A: AuditRepository,
{
    product_repo: Arc<P>,
    category_repo: Arc<C>,
    audit_repo: Arc<A>,
}

impl<P, C, A> CreateProductUseCase<P, C, A>
where
    P: ProductRepository,
    C: CategoryRepository,
    A: AuditRepository,
{
    /// Creates a new instance of CreateProductUseCase
    pub fn new(product_repo: Arc<P>, category_repo: Arc<C>, audit_repo: Arc<A>) -> Self {
        Self {
            product_repo,
            category_repo,
            audit_repo,
        }
    }

    /// Executes the use case to create a new product
    ///
    /// # Arguments
    /// * `command` - The create product command containing product data
    /// * `actor_id` - ID of the user performing this action (for audit)
    ///
    /// # Returns
    /// ProductResponse on success
    ///
    /// # Errors
    /// * `InventoryError::DuplicateBarcode` - If barcode already exists
    /// * `InventoryError::CategoryNotFound` - If category_id is provided but doesn't exist
    /// * `InventoryError::InvalidCurrency` - If currency code is invalid
    /// * `InventoryError::InvalidUnitOfMeasure` - If unit of measure is invalid
    /// * `InventoryError::InvalidBarcode` - If barcode format is invalid
    pub async fn execute(
        &self,
        command: CreateProductCommand,
        actor_id: UserId,
    ) -> Result<ProductResponse, InventoryError> {
        // Validate unit of measure (Requirement 1.2)
        let unit_of_measure = UnitOfMeasure::from_str(&command.unit_of_measure)?;

        // Validate barcode uniqueness if provided (Requirement 1.2)
        let barcode = if let Some(barcode_str) = &command.barcode {
            let barcode = Barcode::new(barcode_str)?;
            if self
                .product_repo
                .find_by_barcode(&barcode)
                .await?
                .is_some()
            {
                return Err(InventoryError::DuplicateBarcode(barcode_str.clone()));
            }
            Some(barcode)
        } else {
            None
        };

        // Validate category exists if provided (Requirement 1.3)
        let category_id = if let Some(cat_uuid) = command.category_id {
            let cat_id = CategoryId::from_uuid(cat_uuid);
            if self.category_repo.find_by_id(cat_id).await?.is_none() {
                return Err(InventoryError::CategoryNotFound(cat_uuid));
            }
            Some(cat_id)
        } else {
            None
        };

        // Get category code for SKU generation
        let category_code = category_id.and_then(|_| {
            // Extract first 3 chars of category name if available
            // For now, we'll use None and let the SKU generator use "GEN"
            // In a real implementation, we'd fetch the category and extract its code
            None
        });

        // Create product entity with auto-generated SKU (Requirement 1.1)
        let mut product = Product::create(command.name, unit_of_measure, category_code);

        // Apply optional and additional fields
        if let Some(barcode) = barcode {
            product.set_barcode(Some(barcode));
        }
        if let Some(description) = command.description {
            product.set_description(Some(description));
        }
        if let Some(category_id) = category_id {
            product.set_category_id(Some(category_id));
        }
        if let Some(brand) = command.brand {
            product.set_brand(Some(brand));
        }
        product.set_base_price(command.base_price);
        product.set_cost_price(command.cost_price);

        // Validate and set currency (Requirement 1.4)
        if let Some(currency_str) = command.currency {
            let currency = Currency::new(&currency_str)?;
            product.set_currency(currency);
        }

        product.set_perishable(command.is_perishable);
        product.set_trackable(command.is_trackable);
        product.set_has_variants(command.has_variants);
        product.set_tax_rate(command.tax_rate);
        product.set_tax_included(command.tax_included);

        if let Some(attributes) = command.attributes {
            product.set_attributes(attributes);
        }

        // Save to repository
        self.product_repo.save(&product).await?;

        // Create audit entry (Requirement 1.3)
        let audit_entry = AuditEntry::for_create(
            "product",
            product.id().into_uuid(),
            &product,
            actor_id,
        );
        self.audit_repo
            .save(&audit_entry)
            .await
            .map_err(|_| InventoryError::NotImplemented)?; // Convert IdentityError to InventoryError

        // Convert to response
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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use identity::domain::entities::AuditEntry;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::domain::entities::ProductCategory;
    use crate::domain::value_objects::{ProductId, Sku};

    // Mock repositories for testing
    struct MockProductRepository {
        products: Mutex<HashMap<ProductId, Product>>,
        barcodes: Mutex<HashMap<String, ProductId>>,
    }

    impl MockProductRepository {
        fn new() -> Self {
            Self {
                products: Mutex::new(HashMap::new()),
                barcodes: Mutex::new(HashMap::new()),
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

        async fn save_variant(
            &self,
            _variant: &crate::domain::entities::ProductVariant,
        ) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_id(
            &self,
            _id: crate::domain::value_objects::VariantId,
        ) -> Result<Option<crate::domain::entities::ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_sku(
            &self,
            _sku: &Sku,
        ) -> Result<Option<crate::domain::entities::ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_barcode(
            &self,
            _barcode: &Barcode,
        ) -> Result<Option<crate::domain::entities::ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn find_variants_by_product(
            &self,
            _product_id: ProductId,
        ) -> Result<Vec<crate::domain::entities::ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn update_variant(
            &self,
            _variant: &crate::domain::entities::ProductVariant,
        ) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete_variant(
            &self,
            _id: crate::domain::value_objects::VariantId,
        ) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn count_variants(&self, _product_id: ProductId) -> Result<u32, InventoryError> {
            unimplemented!()
        }
    }

    struct MockCategoryRepository {
        categories: Mutex<HashMap<CategoryId, ProductCategory>>,
    }

    impl MockCategoryRepository {
        fn new() -> Self {
            Self {
                categories: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CategoryRepository for MockCategoryRepository {
        async fn save(&self, category: &ProductCategory) -> Result<(), InventoryError> {
            let mut categories = self.categories.lock().unwrap();
            categories.insert(category.id(), category.clone());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: CategoryId,
        ) -> Result<Option<ProductCategory>, InventoryError> {
            let categories = self.categories.lock().unwrap();
            Ok(categories.get(&id).cloned())
        }

        async fn find_by_slug(
            &self,
            _slug: &str,
        ) -> Result<Option<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn find_root_categories(&self) -> Result<Vec<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn find_children(
            &self,
            _parent_id: CategoryId,
        ) -> Result<Vec<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn find_all_active(&self) -> Result<Vec<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn update(&self, _category: &ProductCategory) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: CategoryId) -> Result<(), InventoryError> {
            unimplemented!()
        }
    }

    struct MockAuditRepository {
        entries: Mutex<Vec<AuditEntry>>,
    }

    impl MockAuditRepository {
        fn new() -> Self {
            Self {
                entries: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl AuditRepository for MockAuditRepository {
        async fn save(&self, entry: &AuditEntry) -> Result<(), identity::IdentityError> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
            Ok(())
        }

        async fn find_by_entity(
            &self,
            _entity_type: &str,
            _entity_id: uuid::Uuid,
        ) -> Result<Vec<AuditEntry>, identity::IdentityError> {
            unimplemented!()
        }

        async fn find_by_date_range(
            &self,
            _from: chrono::DateTime<Utc>,
            _to: chrono::DateTime<Utc>,
        ) -> Result<Vec<AuditEntry>, identity::IdentityError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_create_product_basic() {
        let product_repo = Arc::new(MockProductRepository::new());
        let category_repo = Arc::new(MockCategoryRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateProductUseCase::new(product_repo, category_repo, audit_repo);

        let command = CreateProductCommand {
            name: "Test Product".to_string(),
            unit_of_measure: "unit".to_string(),
            barcode: None,
            description: Some("A test product".to_string()),
            category_id: None,
            brand: Some("TestBrand".to_string()),
            base_price: dec!(99.99),
            cost_price: dec!(50.00),
            currency: Some("USD".to_string()),
            is_perishable: false,
            is_trackable: true,
            has_variants: false,
            tax_rate: dec!(0.15),
            tax_included: false,
            attributes: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.name, "Test Product");
        assert_eq!(response.base_price, dec!(99.99));
        assert_eq!(response.cost_price, dec!(50.00));
        assert_eq!(response.currency, "USD");
        assert!(response.is_active);
        assert!(response.sku.starts_with("PRD-"));
    }

    #[tokio::test]
    async fn test_create_product_with_category() {
        let product_repo = Arc::new(MockProductRepository::new());
        let category_repo = Arc::new(MockCategoryRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create a category first
        let category =
            ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        category_repo.save(&category).await.unwrap();

        let use_case = CreateProductUseCase::new(product_repo, category_repo, audit_repo);

        let command = CreateProductCommand {
            name: "Smartphone".to_string(),
            unit_of_measure: "unit".to_string(),
            barcode: None,
            description: None,
            category_id: Some(category.id().into_uuid()),
            brand: None,
            base_price: dec!(599.99),
            cost_price: dec!(300.00),
            currency: None,
            is_perishable: false,
            is_trackable: true,
            has_variants: false,
            tax_rate: dec!(0.0),
            tax_included: false,
            attributes: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.category_id, Some(category.id().into_uuid()));
    }

    #[tokio::test]
    async fn test_duplicate_barcode_error() {
        let product_repo = Arc::new(MockProductRepository::new());
        let category_repo = Arc::new(MockCategoryRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());

        // Create first product with barcode
        let mut product = Product::create("Product 1".to_string(), UnitOfMeasure::Unit, None);
        let barcode = Barcode::new("1234567890123").unwrap();
        product.set_barcode(Some(barcode));
        product_repo.save(&product).await.unwrap();

        let use_case = CreateProductUseCase::new(product_repo, category_repo, audit_repo);

        // Try to create another product with same barcode
        let command = CreateProductCommand {
            name: "Product 2".to_string(),
            unit_of_measure: "unit".to_string(),
            barcode: Some("1234567890123".to_string()),
            description: None,
            category_id: None,
            brand: None,
            base_price: dec!(0.0),
            cost_price: dec!(0.0),
            currency: None,
            is_perishable: false,
            is_trackable: true,
            has_variants: false,
            tax_rate: dec!(0.0),
            tax_included: false,
            attributes: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::DuplicateBarcode(_))));
    }

    #[tokio::test]
    async fn test_category_not_found_error() {
        let product_repo = Arc::new(MockProductRepository::new());
        let category_repo = Arc::new(MockCategoryRepository::new());
        let audit_repo = Arc::new(MockAuditRepository::new());
        let use_case = CreateProductUseCase::new(product_repo, category_repo, audit_repo);

        let non_existent_category = CategoryId::new().into_uuid();
        let command = CreateProductCommand {
            name: "Product".to_string(),
            unit_of_measure: "unit".to_string(),
            barcode: None,
            description: None,
            category_id: Some(non_existent_category),
            brand: None,
            base_price: dec!(0.0),
            cost_price: dec!(0.0),
            currency: None,
            is_perishable: false,
            is_trackable: true,
            has_variants: false,
            tax_rate: dec!(0.0),
            tax_included: false,
            attributes: None,
        };

        let actor_id = UserId::new();
        let result = use_case.execute(command, actor_id).await;
        assert!(matches!(result, Err(InventoryError::CategoryNotFound(_))));
    }
}
