use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::CatalogError;
use crate::domain::entities::ProductReview;
use crate::domain::value_objects::{ProductListingId, ProductReviewId};
use sales::CustomerId;

#[async_trait]
pub trait ProductReviewRepository: Send + Sync {
    async fn save(&self, review: &ProductReview) -> Result<(), CatalogError>;
    async fn find_by_id(&self, id: ProductReviewId) -> Result<Option<ProductReview>, CatalogError>;
    /// Returns approved reviews paginated.
    async fn find_approved_by_listing(
        &self,
        listing_id: ProductListingId,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ProductReview>, i64), CatalogError>;
    /// Returns reviews pending moderation.
    async fn find_pending(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ProductReview>, i64), CatalogError>;
    async fn find_by_customer_and_listing(
        &self,
        customer_id: CustomerId,
        listing_id: ProductListingId,
    ) -> Result<Option<ProductReview>, CatalogError>;
    async fn update(&self, review: &ProductReview) -> Result<(), CatalogError>;
    /// Returns (avg_rating, count) for the listing's approved reviews.
    async fn average_rating(
        &self,
        listing_id: ProductListingId,
    ) -> Result<(Decimal, i64), CatalogError>;
    async fn delete(&self, id: ProductReviewId) -> Result<(), CatalogError>;
    /// True when the customer has at least one completed sale containing the
    /// underlying product. Used to set `is_verified_purchase` on submit.
    async fn customer_purchased_product(
        &self,
        customer_id: CustomerId,
        listing_id: ProductListingId,
    ) -> Result<bool, CatalogError>;
}
