use async_trait::async_trait;

use crate::CatalogError;
use crate::domain::entities::ProductImage;
use crate::domain::value_objects::{ProductImageId, ProductListingId};

#[async_trait]
pub trait ProductImageRepository: Send + Sync {
    async fn save(&self, image: &ProductImage) -> Result<(), CatalogError>;
    async fn find_by_id(&self, id: ProductImageId) -> Result<Option<ProductImage>, CatalogError>;
    async fn find_by_listing(
        &self,
        listing_id: ProductListingId,
    ) -> Result<Vec<ProductImage>, CatalogError>;
    async fn count_by_listing(&self, listing_id: ProductListingId) -> Result<i64, CatalogError>;
    async fn delete(&self, id: ProductImageId) -> Result<(), CatalogError>;
    async fn reorder(
        &self,
        listing_id: ProductListingId,
        image_ids: Vec<ProductImageId>,
    ) -> Result<(), CatalogError>;
    /// Clears `is_primary=true` from every image of the listing other than `keep`.
    async fn unset_primary_except(
        &self,
        listing_id: ProductListingId,
        keep: ProductImageId,
    ) -> Result<(), CatalogError>;
}
