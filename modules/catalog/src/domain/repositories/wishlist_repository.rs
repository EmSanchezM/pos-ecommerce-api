use async_trait::async_trait;

use crate::CatalogError;
use crate::domain::entities::Wishlist;
use crate::domain::value_objects::{ProductListingId, WishlistId};
use identity::StoreId;
use sales::CustomerId;

#[async_trait]
pub trait WishlistRepository: Send + Sync {
    async fn find_or_create(
        &self,
        customer_id: CustomerId,
        store_id: StoreId,
    ) -> Result<Wishlist, CatalogError>;
    async fn find_by_customer(
        &self,
        customer_id: CustomerId,
        store_id: StoreId,
    ) -> Result<Option<Wishlist>, CatalogError>;
    async fn add_item(
        &self,
        wishlist_id: WishlistId,
        listing_id: ProductListingId,
    ) -> Result<(), CatalogError>;
    async fn remove_item(
        &self,
        wishlist_id: WishlistId,
        listing_id: ProductListingId,
    ) -> Result<(), CatalogError>;
}
