use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{Wishlist, WishlistItem};

#[derive(Debug, Deserialize)]
pub struct AddWishlistItemCommand {
    pub customer_id: Uuid,
    pub store_id: Uuid,
    pub listing_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RemoveWishlistItemCommand {
    pub customer_id: Uuid,
    pub store_id: Uuid,
    pub listing_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct WishlistItemResponse {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub added_at: DateTime<Utc>,
}

impl From<WishlistItem> for WishlistItemResponse {
    fn from(i: WishlistItem) -> Self {
        Self {
            id: i.id().into_uuid(),
            listing_id: i.listing_id().into_uuid(),
            added_at: i.added_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WishlistResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub store_id: Uuid,
    pub items: Vec<WishlistItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Wishlist> for WishlistResponse {
    fn from(w: Wishlist) -> Self {
        Self {
            id: w.id().into_uuid(),
            customer_id: w.customer_id().into_uuid(),
            store_id: w.store_id().into_uuid(),
            items: w
                .items()
                .iter()
                .cloned()
                .map(WishlistItemResponse::from)
                .collect(),
            created_at: w.created_at(),
            updated_at: w.updated_at(),
        }
    }
}
