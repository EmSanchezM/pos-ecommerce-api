//! Update cart item use case

use std::sync::Arc;

use crate::application::dtos::{CartResponse, UpdateCartItemCommand};
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::{CartId, CartItemId};
use crate::SalesError;

/// Use case for updating a cart item quantity
pub struct UpdateCartItemUseCase {
    cart_repo: Arc<dyn CartRepository>,
}

impl UpdateCartItemUseCase {
    pub fn new(cart_repo: Arc<dyn CartRepository>) -> Self {
        Self { cart_repo }
    }

    pub async fn execute(&self, cmd: UpdateCartItemCommand) -> Result<CartResponse, SalesError> {
        let cart_id = CartId::from_uuid(cmd.cart_id);
        let item_id = CartItemId::from_uuid(cmd.item_id);

        let mut cart = self
            .cart_repo
            .find_by_id_with_items(cart_id)
            .await?
            .ok_or(SalesError::CartNotFound(cmd.cart_id))?;

        cart.update_item_quantity(item_id, cmd.quantity)?;

        // Find the updated item for persistence
        let item = cart
            .items()
            .iter()
            .find(|i| i.id() == item_id)
            .ok_or(SalesError::CartItemNotFound(cmd.item_id))?;

        self.cart_repo.update_item(item).await?;
        self.cart_repo.update(&cart).await?;

        Ok(CartResponse::from(cart))
    }
}
