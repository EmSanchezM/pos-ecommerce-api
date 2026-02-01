//! Remove cart item use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CartResponse;
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::{CartId, CartItemId};
use crate::SalesError;

/// Use case for removing an item from a cart
pub struct RemoveCartItemUseCase {
    cart_repo: Arc<dyn CartRepository>,
}

impl RemoveCartItemUseCase {
    pub fn new(cart_repo: Arc<dyn CartRepository>) -> Self {
        Self { cart_repo }
    }

    /// Removes an item from a cart. Takes cart_id and item_id.
    pub async fn execute(&self, cart_id: Uuid, item_id: Uuid) -> Result<CartResponse, SalesError> {
        let cart_id_vo = CartId::from_uuid(cart_id);
        let item_id_vo = CartItemId::from_uuid(item_id);

        let mut cart = self
            .cart_repo
            .find_by_id_with_items(cart_id_vo)
            .await?
            .ok_or(SalesError::CartNotFound(cart_id))?;

        cart.remove_item(item_id_vo)?;

        self.cart_repo.delete_item(item_id_vo).await?;
        self.cart_repo.update(&cart).await?;

        Ok(CartResponse::from(cart))
    }
}
