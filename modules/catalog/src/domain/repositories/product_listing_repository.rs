use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::CatalogError;
use crate::domain::entities::ProductListing;
use crate::domain::value_objects::ProductListingId;
use identity::StoreId;

#[derive(Debug, Clone, Default)]
pub struct CatalogFilter {
    pub store_id: Option<StoreId>,
    pub category_id: Option<Uuid>,
    pub is_published: Option<bool>,
    pub is_featured: Option<bool>,
    pub search: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    /// `price_asc`, `price_desc`, `newest`, `popular` (view_count desc)
    pub sort_by: Option<String>,
}

#[async_trait]
pub trait ProductListingRepository: Send + Sync {
    async fn save(&self, listing: &ProductListing) -> Result<(), CatalogError>;
    async fn find_by_id(
        &self,
        id: ProductListingId,
    ) -> Result<Option<ProductListing>, CatalogError>;
    async fn find_by_slug(
        &self,
        store_id: StoreId,
        slug: &str,
    ) -> Result<Option<ProductListing>, CatalogError>;
    async fn find_by_product_id(
        &self,
        product_id: Uuid,
    ) -> Result<Option<ProductListing>, CatalogError>;
    async fn update(&self, listing: &ProductListing) -> Result<(), CatalogError>;
    async fn find_paginated(
        &self,
        filter: CatalogFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ProductListing>, i64), CatalogError>;
    async fn find_featured(
        &self,
        store_id: StoreId,
        limit: i64,
    ) -> Result<Vec<ProductListing>, CatalogError>;
    async fn increment_view_count(&self, id: ProductListingId) -> Result<(), CatalogError>;
    async fn delete(&self, id: ProductListingId) -> Result<(), CatalogError>;
}
