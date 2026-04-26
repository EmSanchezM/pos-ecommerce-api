use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::CatalogError;
use crate::domain::entities::{Wishlist, WishlistItem};
use crate::domain::repositories::WishlistRepository;
use crate::domain::value_objects::{ProductListingId, WishlistId, WishlistItemId};
use identity::StoreId;
use sales::CustomerId;

pub struct PgWishlistRepository {
    pool: PgPool,
}

impl PgWishlistRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WishlistRepository for PgWishlistRepository {
    async fn find_or_create(
        &self,
        customer_id: CustomerId,
        store_id: StoreId,
    ) -> Result<Wishlist, CatalogError> {
        // Try fetch first.
        if let Some(w) = self.find_by_customer(customer_id, store_id).await? {
            return Ok(w);
        }
        // Insert + fetch.
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        sqlx::query(
            "INSERT INTO wishlists (id, customer_id, store_id) VALUES ($1, $2, $3)
             ON CONFLICT (customer_id, store_id) DO NOTHING",
        )
        .bind(id)
        .bind(customer_id.into_uuid())
        .bind(store_id.into_uuid())
        .execute(&self.pool)
        .await?;
        self.find_by_customer(customer_id, store_id)
            .await?
            .ok_or_else(|| CatalogError::WishlistNotFound(id))
    }

    async fn find_by_customer(
        &self,
        customer_id: CustomerId,
        store_id: StoreId,
    ) -> Result<Option<Wishlist>, CatalogError> {
        let row: Option<WishlistRow> = sqlx::query_as(
            "SELECT id, customer_id, store_id, created_at, updated_at
             FROM wishlists WHERE customer_id = $1 AND store_id = $2 LIMIT 1",
        )
        .bind(customer_id.into_uuid())
        .bind(store_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        let Some(w) = row else { return Ok(None) };
        let item_rows: Vec<ItemRow> = sqlx::query_as(
            "SELECT id, wishlist_id, listing_id, added_at
             FROM wishlist_items WHERE wishlist_id = $1
             ORDER BY added_at DESC",
        )
        .bind(w.id)
        .fetch_all(&self.pool)
        .await?;

        let items = item_rows
            .into_iter()
            .map(|r| {
                WishlistItem::reconstitute(
                    WishlistItemId::from_uuid(r.id),
                    WishlistId::from_uuid(r.wishlist_id),
                    ProductListingId::from_uuid(r.listing_id),
                    r.added_at,
                )
            })
            .collect();

        Ok(Some(Wishlist::reconstitute(
            WishlistId::from_uuid(w.id),
            CustomerId::from_uuid(w.customer_id),
            StoreId::from_uuid(w.store_id),
            items,
            w.created_at,
            w.updated_at,
        )))
    }

    async fn add_item(
        &self,
        wishlist_id: WishlistId,
        listing_id: ProductListingId,
    ) -> Result<(), CatalogError> {
        let id = Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext));
        sqlx::query(
            "INSERT INTO wishlist_items (id, wishlist_id, listing_id) VALUES ($1, $2, $3)
             ON CONFLICT (wishlist_id, listing_id) DO NOTHING",
        )
        .bind(id)
        .bind(wishlist_id.into_uuid())
        .bind(listing_id.into_uuid())
        .execute(&self.pool)
        .await?;
        sqlx::query("UPDATE wishlists SET updated_at = NOW() WHERE id = $1")
            .bind(wishlist_id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn remove_item(
        &self,
        wishlist_id: WishlistId,
        listing_id: ProductListingId,
    ) -> Result<(), CatalogError> {
        let result =
            sqlx::query("DELETE FROM wishlist_items WHERE wishlist_id = $1 AND listing_id = $2")
                .bind(wishlist_id.into_uuid())
                .bind(listing_id.into_uuid())
                .execute(&self.pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::WishlistItemNotFound);
        }
        sqlx::query("UPDATE wishlists SET updated_at = NOW() WHERE id = $1")
            .bind(wishlist_id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct WishlistRow {
    id: uuid::Uuid,
    customer_id: uuid::Uuid,
    store_id: uuid::Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: uuid::Uuid,
    wishlist_id: uuid::Uuid,
    listing_id: uuid::Uuid,
    added_at: chrono::DateTime<chrono::Utc>,
}
