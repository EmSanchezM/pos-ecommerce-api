// GetValuationReportUseCase - generates inventory valuation report

use std::sync::Arc;

use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::application::dtos::responses::{ValuationItemResponse, ValuationReportResponse};
use crate::domain::repositories::{InventoryMovementRepository, InventoryStockRepository, ProductRepository};
use crate::InventoryError;

/// Query parameters for valuation report
#[derive(Debug, Clone)]
pub struct ValuationReportQuery {
    /// Filter by store ID (optional - if None, includes all stores)
    pub store_id: Option<Uuid>,
    /// Currency for the report (defaults to HNL)
    pub currency: Option<String>,
}

/// Use case for generating inventory valuation report
pub struct GetValuationReportUseCase<S, M, P>
where
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
    P: ProductRepository,
{
    stock_repo: Arc<S>,
    movement_repo: Arc<M>,
    product_repo: Arc<P>,
}

impl<S, M, P> GetValuationReportUseCase<S, M, P>
where
    S: InventoryStockRepository,
    M: InventoryMovementRepository,
    P: ProductRepository,
{
    pub fn new(stock_repo: Arc<S>, movement_repo: Arc<M>, product_repo: Arc<P>) -> Self {
        Self {
            stock_repo,
            movement_repo,
            product_repo,
        }
    }

    /// Executes the use case to generate valuation report
    ///
    /// # Arguments
    /// * `query` - Query parameters including optional store filter
    ///
    /// # Returns
    /// ValuationReportResponse with stock values and totals
    pub async fn execute(&self, query: ValuationReportQuery) -> Result<ValuationReportResponse, InventoryError> {
        let currency = query.currency.unwrap_or_else(|| "HNL".to_string());

        // Get all stock records (optionally filtered by store)
        let stocks = if let Some(store_id) = query.store_id {
            self.stock_repo.find_by_store(store_id.into()).await?
        } else {
            self.stock_repo.find_all().await?
        };

        let mut items = Vec::new();
        let mut total_value = Decimal::ZERO;

        for stock in stocks {
            // Skip stocks with zero quantity
            if stock.quantity() <= Decimal::ZERO {
                continue;
            }

            // Get weighted average cost for this stock
            let unit_cost = self
                .movement_repo
                .calculate_weighted_average_cost(stock.id())
                .await?
                .unwrap_or(Decimal::ZERO);

            let item_total_value = stock.quantity() * unit_cost;
            total_value += item_total_value;

            // Get product name and SKU
            let (product_name, variant_name, sku) = self
                .get_product_info(&stock)
                .await?;

            items.push(ValuationItemResponse {
                stock_id: stock.id().into_uuid(),
                store_id: *stock.store_id().as_uuid(),
                product_id: stock.product_id().map(|id| id.into_uuid()),
                variant_id: stock.variant_id().map(|id| id.into_uuid()),
                product_name,
                variant_name,
                sku,
                quantity: stock.quantity(),
                unit_cost,
                total_value: item_total_value,
                currency: currency.clone(),
            });
        }

        Ok(ValuationReportResponse {
            items: items.clone(),
            total_items: items.len() as i64,
            total_value,
            currency,
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

        if let Some(product_id) = stock.product_id()
            && let Some(product) = self.product_repo.find_by_id(product_id).await? {
                product_name = Some(product.name().to_string());
                sku = Some(product.sku().to_string());
            }

        if let Some(variant_id) = stock.variant_id()
            && let Some(variant) = self.product_repo.find_variant_by_id(variant_id).await? {
                variant_name = Some(variant.name().to_string());
                sku = Some(variant.sku().to_string());

                // Also get parent product name if not already set
                if product_name.is_none()
                    && let Some(product) = self.product_repo.find_by_id(variant.product_id()).await? {
                        product_name = Some(product.name().to_string());
                    }
            }

        Ok((product_name, variant_name, sku))
    }
}
