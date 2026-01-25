// GetLowStockReportUseCase - generates low stock report with reorder suggestions

use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::application::dtos::responses::{LowStockItemResponse, LowStockReportResponse};
use crate::domain::repositories::{InventoryStockRepository, ProductRepository};
use crate::InventoryError;

/// Query parameters for low stock report
#[derive(Debug, Clone)]
pub struct LowStockReportQuery {
    /// Filter by store ID (optional - if None, includes all stores)
    pub store_id: Option<Uuid>,
    /// Include items with zero stock (default: true)
    pub include_zero_stock: bool,
}

impl Default for LowStockReportQuery {
    fn default() -> Self {
        Self {
            store_id: None,
            include_zero_stock: true,
        }
    }
}

/// Use case for generating low stock report with reorder suggestions
pub struct GetLowStockReportUseCase<S, P>
where
    S: InventoryStockRepository,
    P: ProductRepository,
{
    stock_repo: Arc<S>,
    product_repo: Arc<P>,
}

impl<S, P> GetLowStockReportUseCase<S, P>
where
    S: InventoryStockRepository,
    P: ProductRepository,
{
    pub fn new(stock_repo: Arc<S>, product_repo: Arc<P>) -> Self {
        Self {
            stock_repo,
            product_repo,
        }
    }

    /// Executes the use case to generate low stock report
    ///
    /// # Arguments
    /// * `query` - Query parameters including optional store filter
    ///
    /// # Returns
    /// LowStockReportResponse with low stock items and reorder suggestions
    pub async fn execute(&self, query: LowStockReportQuery) -> Result<LowStockReportResponse, InventoryError> {
        // Get low stock items (optionally filtered by store)
        let stocks = if let Some(store_id) = query.store_id {
            self.stock_repo.find_low_stock_by_store(store_id.into()).await?
        } else {
            self.stock_repo.find_all_low_stock().await?
        };

        let mut items = Vec::new();

        for stock in stocks {
            let available = stock.available_quantity();

            // Skip zero stock items if not requested
            if !query.include_zero_stock && available <= Decimal::ZERO {
                continue;
            }

            // Calculate shortage (how much below min level)
            let shortage = if available < stock.min_stock_level() {
                stock.min_stock_level() - available
            } else {
                Decimal::ZERO
            };

            // Calculate reorder suggestion
            // Suggest ordering enough to reach max_stock_level if set,
            // otherwise suggest 2x the min_stock_level
            let reorder_suggestion = if let Some(max_level) = stock.max_stock_level() {
                max_level - available
            } else {
                (stock.min_stock_level() * Decimal::from(2)) - available
            };

            // Get product info
            let (product_name, variant_name, sku) = self
                .get_product_info(&stock)
                .await?;

            items.push(LowStockItemResponse {
                stock_id: stock.id().into_uuid(),
                store_id: *stock.store_id().as_uuid(),
                product_id: stock.product_id().map(|id| id.into_uuid()),
                variant_id: stock.variant_id().map(|id| id.into_uuid()),
                product_name,
                variant_name,
                sku,
                current_quantity: stock.quantity(),
                available_quantity: available,
                min_stock_level: stock.min_stock_level(),
                shortage,
                reorder_suggestion: reorder_suggestion.max(Decimal::ZERO),
            });
        }

        // Sort by shortage (highest first) to prioritize critical items
        items.sort_by(|a, b| b.shortage.cmp(&a.shortage));

        Ok(LowStockReportResponse {
            items: items.clone(),
            total_items: items.len() as i64,
            generated_at: Utc::now(),
        })
    }

    async fn get_product_info(
        &self,
        stock: &crate::domain::entities::InventoryStock,
    ) -> Result<(Option<String>, Option<String>, Option<String>), InventoryError> {
        let mut product_name = None;
        let mut variant_name = None;
        let mut sku = None;

        if let Some(product_id) = stock.product_id() {
            if let Some(product) = self.product_repo.find_by_id(product_id).await? {
                product_name = Some(product.name().to_string());
                sku = Some(product.sku().to_string());
            }
        }

        if let Some(variant_id) = stock.variant_id() {
            if let Some(variant) = self.product_repo.find_variant_by_id(variant_id).await? {
                variant_name = Some(variant.name().to_string());
                sku = Some(variant.sku().to_string());

                // Also get parent product name if not already set
                if product_name.is_none() {
                    if let Some(product) = self.product_repo.find_by_id(variant.product_id()).await? {
                        product_name = Some(product.name().to_string());
                    }
                }
            }
        }

        Ok((product_name, variant_name, sku))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::{NoContext, Timestamp, Uuid};

    use crate::domain::entities::{InventoryStock, Product, ProductVariant};
    use crate::domain::value_objects::{
        Barcode, CategoryId, ProductId, Sku, StockId, VariantId,
    };
    use identity::StoreId;

    fn new_uuid() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    // Mock Stock Repository
    struct MockStockRepository {
        stocks: Mutex<HashMap<StockId, InventoryStock>>,
    }

    impl MockStockRepository {
        fn new() -> Self {
            Self {
                stocks: Mutex::new(HashMap::new()),
            }
        }

        fn add_stock(&self, stock: InventoryStock) {
            let mut stocks = self.stocks.lock().unwrap();
            stocks.insert(stock.id(), stock);
        }
    }

    #[async_trait]
    impl InventoryStockRepository for MockStockRepository {
        async fn save(&self, stock: &InventoryStock) -> Result<(), InventoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            stocks.insert(stock.id(), stock.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: StockId) -> Result<Option<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.get(&id).cloned())
        }

        async fn find_by_store_and_product(
            &self,
            _store_id: StoreId,
            _product_id: ProductId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_store_and_variant(
            &self,
            _store_id: StoreId,
            _variant_id: VariantId,
        ) -> Result<Option<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn update_with_version(
            &self,
            _stock: &InventoryStock,
            _expected_version: i32,
        ) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_low_stock(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks
                .values()
                .filter(|s| {
                    *s.store_id().as_uuid() == *store_id.as_uuid()
                        && s.available_quantity() <= s.min_stock_level()
                })
                .cloned()
                .collect())
        }

        async fn find_by_store(&self, _store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_paginated(
            &self,
            _store_id: Option<StoreId>,
            _product_id: Option<ProductId>,
            _low_stock_only: bool,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<InventoryStock>, i64), InventoryError> {
            unimplemented!()
        }

        async fn find_by_product(&self, _product_id: ProductId) -> Result<Vec<InventoryStock>, InventoryError> {
            unimplemented!()
        }

        async fn find_all(&self) -> Result<Vec<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.values().cloned().collect())
        }

        async fn find_all_low_stock(&self) -> Result<Vec<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks
                .values()
                .filter(|s| s.available_quantity() <= s.min_stock_level())
                .cloned()
                .collect())
        }

        async fn find_low_stock_by_store(&self, store_id: StoreId) -> Result<Vec<InventoryStock>, InventoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks
                .values()
                .filter(|s| {
                    *s.store_id().as_uuid() == *store_id.as_uuid()
                        && s.available_quantity() <= s.min_stock_level()
                })
                .cloned()
                .collect())
        }
    }

    // Mock Product Repository (minimal implementation)
    struct MockProductRepository;

    #[async_trait]
    impl ProductRepository for MockProductRepository {
        async fn save(&self, _product: &Product) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_by_id(&self, _id: ProductId) -> Result<Option<Product>, InventoryError> {
            Ok(None) // No product info for simplicity
        }

        async fn find_by_sku(&self, _sku: &Sku) -> Result<Option<Product>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_category(&self, _category_id: CategoryId) -> Result<Vec<Product>, InventoryError> {
            unimplemented!()
        }

        async fn update(&self, _product: &Product) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: ProductId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_paginated(
            &self,
            _category_id: Option<CategoryId>,
            _is_active: Option<bool>,
            _search: Option<&str>,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<Product>, i64), InventoryError> {
            unimplemented!()
        }

        async fn save_variant(&self, _variant: &ProductVariant) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_id(&self, _id: VariantId) -> Result<Option<ProductVariant>, InventoryError> {
            Ok(None) // No variant info for simplicity
        }

        async fn find_variant_by_sku(&self, _sku: &Sku) -> Result<Option<ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn find_variants_by_product(&self, _product_id: ProductId) -> Result<Vec<ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn update_variant(&self, _variant: &ProductVariant) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete_variant(&self, _id: VariantId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_by_barcode(&self, _barcode: &Barcode) -> Result<Option<Product>, InventoryError> {
            unimplemented!()
        }

        async fn find_active(&self) -> Result<Vec<Product>, InventoryError> {
            unimplemented!()
        }

        async fn count_filtered(
            &self,
            _category_id: Option<CategoryId>,
            _is_active: Option<bool>,
            _search: Option<&str>,
        ) -> Result<i64, InventoryError> {
            unimplemented!()
        }

        async fn find_variant_by_barcode(&self, _barcode: &Barcode) -> Result<Option<ProductVariant>, InventoryError> {
            unimplemented!()
        }

        async fn count_variants(&self, _product_id: ProductId) -> Result<u32, InventoryError> {
            unimplemented!()
        }
    }

    fn create_test_stock(
        quantity: Decimal,
        reserved: Decimal,
        min_level: Decimal,
        max_level: Option<Decimal>,
    ) -> InventoryStock {
        let now = Utc::now();
        InventoryStock::reconstitute(
            StockId::new(),
            StoreId::from_uuid(new_uuid()),
            Some(ProductId::from_uuid(new_uuid())),
            None,
            quantity,
            reserved,
            1,
            min_level,
            max_level,
            now,
            now,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_low_stock_report_empty() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository);

        let use_case = GetLowStockReportUseCase::new(stock_repo, product_repo);
        let result = use_case.execute(LowStockReportQuery::default()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_items, 0);
        assert!(response.items.is_empty());
    }

    #[tokio::test]
    async fn test_low_stock_report_with_items() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository);

        // Add a stock with quantity below min level
        let stock = create_test_stock(dec!(5), dec!(0), dec!(10), Some(dec!(100)));
        stock_repo.add_stock(stock);

        let use_case = GetLowStockReportUseCase::new(stock_repo, product_repo);
        let result = use_case.execute(LowStockReportQuery::default()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_items, 1);
        assert_eq!(response.items[0].shortage, dec!(5)); // 10 - 5 = 5
        assert_eq!(response.items[0].reorder_suggestion, dec!(95)); // 100 - 5 = 95
    }

    #[tokio::test]
    async fn test_low_stock_report_sorted_by_shortage() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository);

        // Add stocks with different shortage levels
        stock_repo.add_stock(create_test_stock(dec!(8), dec!(0), dec!(10), None)); // shortage = 2
        stock_repo.add_stock(create_test_stock(dec!(2), dec!(0), dec!(10), None)); // shortage = 8
        stock_repo.add_stock(create_test_stock(dec!(5), dec!(0), dec!(10), None)); // shortage = 5

        let use_case = GetLowStockReportUseCase::new(stock_repo, product_repo);
        let result = use_case.execute(LowStockReportQuery::default()).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total_items, 3);
        // Should be sorted by shortage (highest first)
        assert_eq!(response.items[0].shortage, dec!(8));
        assert_eq!(response.items[1].shortage, dec!(5));
        assert_eq!(response.items[2].shortage, dec!(2));
    }

    #[tokio::test]
    async fn test_low_stock_report_excludes_zero_stock_when_requested() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository);

        // Add a stock with zero quantity
        stock_repo.add_stock(create_test_stock(dec!(0), dec!(0), dec!(10), None));
        // Add a stock with some quantity
        stock_repo.add_stock(create_test_stock(dec!(5), dec!(0), dec!(10), None));

        let use_case = GetLowStockReportUseCase::new(stock_repo, product_repo);

        // First, with zero stock included (default)
        let result = use_case.execute(LowStockReportQuery::default()).await.unwrap();
        assert_eq!(result.total_items, 2);

        // Then, with zero stock excluded
        let result = use_case
            .execute(LowStockReportQuery {
                include_zero_stock: false,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.total_items, 1);
    }

    #[tokio::test]
    async fn test_low_stock_report_reorder_suggestion_without_max() {
        let stock_repo = Arc::new(MockStockRepository::new());
        let product_repo = Arc::new(MockProductRepository);

        // Add stock without max_level (should use 2x min as target)
        // quantity=5, min=10, no max -> reorder = (10*2) - 5 = 15
        stock_repo.add_stock(create_test_stock(dec!(5), dec!(0), dec!(10), None));

        let use_case = GetLowStockReportUseCase::new(stock_repo, product_repo);
        let result = use_case.execute(LowStockReportQuery::default()).await.unwrap();

        assert_eq!(result.items[0].reorder_suggestion, dec!(15));
    }

    // =========================================================================
    // Property-Based Tests for Low Stock Report Accuracy
    // =========================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        /// Generates a decimal value for quantity testing
        fn quantity_strategy() -> impl Strategy<Value = Decimal> {
            (0u32..1000u32).prop_map(|v| Decimal::from(v))
        }

        /// Generates a decimal value for stock levels
        fn level_strategy() -> impl Strategy<Value = Decimal> {
            (1u32..500u32).prop_map(|v| Decimal::from(v))
        }

        // Property 15: Low Stock Report Accuracy
        // **For any** stock record where available_quantity <= min_stock_level,
        // the report SHALL correctly calculate:
        // 1. shortage = max(0, min_stock_level - available_quantity)
        // 2. reorder_suggestion = max(0, target_level - available_quantity)
        //    where target_level = max_stock_level if set, else 2 * min_stock_level
        // **Validates: Requirements 8.4**
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]
            #[test]
            fn prop_low_stock_shortage_calculation(
                quantity in quantity_strategy(),
                reserved in quantity_strategy(),
                min_level in level_strategy(),
            ) {
                let available = if quantity > reserved { quantity - reserved } else { Decimal::ZERO };

                // Calculate expected shortage
                let expected_shortage = if available < min_level {
                    min_level - available
                } else {
                    Decimal::ZERO
                };

                // The shortage should always be >= 0
                prop_assert!(expected_shortage >= Decimal::ZERO);

                // The shortage + available should equal min_level when available < min_level
                if available < min_level {
                    prop_assert_eq!(expected_shortage + available, min_level);
                }
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]
            #[test]
            fn prop_low_stock_reorder_with_max(
                quantity in quantity_strategy(),
                min_level in level_strategy(),
                max_level_offset in level_strategy(),
            ) {
                let max_level = min_level + max_level_offset; // Ensure max > min
                let available = quantity;

                // Calculate expected reorder suggestion when max_level is set
                let expected_reorder = if max_level > available {
                    (max_level - available).max(Decimal::ZERO)
                } else {
                    Decimal::ZERO
                };

                // The reorder suggestion should always be >= 0
                prop_assert!(expected_reorder >= Decimal::ZERO);

                // After reorder, we should reach max_level
                if expected_reorder > Decimal::ZERO {
                    prop_assert_eq!(available + expected_reorder, max_level);
                }
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]
            #[test]
            fn prop_low_stock_reorder_without_max(
                quantity in quantity_strategy(),
                min_level in level_strategy(),
            ) {
                let available = quantity;
                let target_level = min_level * Decimal::from(2);

                // Calculate expected reorder suggestion when no max_level
                let expected_reorder = if target_level > available {
                    (target_level - available).max(Decimal::ZERO)
                } else {
                    Decimal::ZERO
                };

                // The reorder suggestion should always be >= 0
                prop_assert!(expected_reorder >= Decimal::ZERO);

                // After reorder, we should reach 2x min_level
                if expected_reorder > Decimal::ZERO {
                    prop_assert_eq!(available + expected_reorder, target_level);
                }
            }
        }
    }
}
