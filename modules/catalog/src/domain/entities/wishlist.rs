//! Wishlist + WishlistItem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{ProductListingId, WishlistId, WishlistItemId};
use identity::StoreId;
use sales::CustomerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wishlist {
    id: WishlistId,
    customer_id: CustomerId,
    store_id: StoreId,
    items: Vec<WishlistItem>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Wishlist {
    pub fn create(customer_id: CustomerId, store_id: StoreId) -> Self {
        let now = Utc::now();
        Self {
            id: WishlistId::new(),
            customer_id,
            store_id,
            items: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn reconstitute(
        id: WishlistId,
        customer_id: CustomerId,
        store_id: StoreId,
        items: Vec<WishlistItem>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            customer_id,
            store_id,
            items,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> WishlistId {
        self.id
    }
    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn items(&self) -> &[WishlistItem] {
        &self.items
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishlistItem {
    id: WishlistItemId,
    wishlist_id: WishlistId,
    listing_id: ProductListingId,
    added_at: DateTime<Utc>,
}

impl WishlistItem {
    pub fn create(wishlist_id: WishlistId, listing_id: ProductListingId) -> Self {
        Self {
            id: WishlistItemId::new(),
            wishlist_id,
            listing_id,
            added_at: Utc::now(),
        }
    }

    pub fn reconstitute(
        id: WishlistItemId,
        wishlist_id: WishlistId,
        listing_id: ProductListingId,
        added_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            wishlist_id,
            listing_id,
            added_at,
        }
    }

    pub fn id(&self) -> WishlistItemId {
        self.id
    }
    pub fn wishlist_id(&self) -> WishlistId {
        self.wishlist_id
    }
    pub fn listing_id(&self) -> ProductListingId {
        self.listing_id
    }
    pub fn added_at(&self) -> DateTime<Utc> {
        self.added_at
    }
}
