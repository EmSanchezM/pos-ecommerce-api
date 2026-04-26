//! Wishlist use cases.

use std::sync::Arc;

use crate::CatalogError;
use crate::application::dtos::{
    AddWishlistItemCommand, RemoveWishlistItemCommand, WishlistResponse,
};
use crate::domain::repositories::WishlistRepository;
use crate::domain::value_objects::ProductListingId;
use identity::StoreId;
use sales::CustomerId;

pub struct GetWishlistUseCase {
    wishlist_repo: Arc<dyn WishlistRepository>,
}

impl GetWishlistUseCase {
    pub fn new(wishlist_repo: Arc<dyn WishlistRepository>) -> Self {
        Self { wishlist_repo }
    }
    pub async fn execute(
        &self,
        customer_id: uuid::Uuid,
        store_id: uuid::Uuid,
    ) -> Result<WishlistResponse, CatalogError> {
        let wishlist = self
            .wishlist_repo
            .find_or_create(
                CustomerId::from_uuid(customer_id),
                StoreId::from_uuid(store_id),
            )
            .await?;
        Ok(WishlistResponse::from(wishlist))
    }
}

pub struct AddToWishlistUseCase {
    wishlist_repo: Arc<dyn WishlistRepository>,
}

impl AddToWishlistUseCase {
    pub fn new(wishlist_repo: Arc<dyn WishlistRepository>) -> Self {
        Self { wishlist_repo }
    }
    pub async fn execute(
        &self,
        cmd: AddWishlistItemCommand,
    ) -> Result<WishlistResponse, CatalogError> {
        let customer_id = CustomerId::from_uuid(cmd.customer_id);
        let store_id = StoreId::from_uuid(cmd.store_id);
        let listing_id = ProductListingId::from_uuid(cmd.listing_id);
        let wishlist = self
            .wishlist_repo
            .find_or_create(customer_id, store_id)
            .await?;
        self.wishlist_repo
            .add_item(wishlist.id(), listing_id)
            .await?;
        // Re-fetch to include the new item.
        let updated = self
            .wishlist_repo
            .find_by_customer(customer_id, store_id)
            .await?
            .ok_or_else(|| CatalogError::WishlistNotFound(wishlist.id().into_uuid()))?;
        Ok(WishlistResponse::from(updated))
    }
}

pub struct RemoveFromWishlistUseCase {
    wishlist_repo: Arc<dyn WishlistRepository>,
}

impl RemoveFromWishlistUseCase {
    pub fn new(wishlist_repo: Arc<dyn WishlistRepository>) -> Self {
        Self { wishlist_repo }
    }
    pub async fn execute(
        &self,
        cmd: RemoveWishlistItemCommand,
    ) -> Result<WishlistResponse, CatalogError> {
        let customer_id = CustomerId::from_uuid(cmd.customer_id);
        let store_id = StoreId::from_uuid(cmd.store_id);
        let wishlist = self
            .wishlist_repo
            .find_by_customer(customer_id, store_id)
            .await?
            .ok_or_else(|| CatalogError::WishlistNotFound(uuid::Uuid::nil()))?;
        self.wishlist_repo
            .remove_item(wishlist.id(), ProductListingId::from_uuid(cmd.listing_id))
            .await?;
        let updated = self
            .wishlist_repo
            .find_by_customer(customer_id, store_id)
            .await?
            .ok_or_else(|| CatalogError::WishlistNotFound(wishlist.id().into_uuid()))?;
        Ok(WishlistResponse::from(updated))
    }
}
