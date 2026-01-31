//! Cart repository trait

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::entities::{Cart, CartItem};
use crate::domain::value_objects::{CartId, CartItemId, CustomerId};
use crate::SalesError;
use identity::StoreId;

/// Filter for querying carts
#[derive(Debug, Clone, Default)]
pub struct CartFilter {
    pub store_id: Option<StoreId>,
    pub customer_id: Option<CustomerId>,
    pub session_id: Option<String>,
    pub include_expired: bool,
}

/// Repository trait for Cart persistence
#[async_trait]
pub trait CartRepository: Send + Sync {
    /// Saves a new cart
    async fn save(&self, cart: &Cart) -> Result<(), SalesError>;

    /// Finds a cart by ID
    async fn find_by_id(&self, id: CartId) -> Result<Option<Cart>, SalesError>;

    /// Finds a cart by ID with items
    async fn find_by_id_with_items(&self, id: CartId) -> Result<Option<Cart>, SalesError>;

    /// Finds an active cart by session ID
    async fn find_by_session(
        &self,
        store_id: StoreId,
        session_id: &str,
    ) -> Result<Option<Cart>, SalesError>;

    /// Finds an active cart by customer ID
    async fn find_by_customer(
        &self,
        store_id: StoreId,
        customer_id: CustomerId,
    ) -> Result<Option<Cart>, SalesError>;

    /// Updates an existing cart
    async fn update(&self, cart: &Cart) -> Result<(), SalesError>;

    /// Deletes a cart and its items
    async fn delete(&self, id: CartId) -> Result<(), SalesError>;

    /// Finds expired carts
    async fn find_expired(&self, before: DateTime<Utc>) -> Result<Vec<Cart>, SalesError>;

    /// Deletes expired carts
    async fn delete_expired(&self, before: DateTime<Utc>) -> Result<i64, SalesError>;

    // -------------------------------------------------------------------------
    // Cart Item operations
    // -------------------------------------------------------------------------

    /// Saves a cart item
    async fn save_item(&self, item: &CartItem) -> Result<(), SalesError>;

    /// Updates a cart item
    async fn update_item(&self, item: &CartItem) -> Result<(), SalesError>;

    /// Deletes a cart item
    async fn delete_item(&self, item_id: CartItemId) -> Result<(), SalesError>;

    /// Finds items for a cart
    async fn find_items_by_cart(&self, cart_id: CartId) -> Result<Vec<CartItem>, SalesError>;

    /// Finds a cart item by ID
    async fn find_item_by_id(&self, item_id: CartItemId) -> Result<Option<CartItem>, SalesError>;

    /// Deletes all items for a cart
    async fn delete_items_by_cart(&self, cart_id: CartId) -> Result<(), SalesError>;
}
