//! Clear cart use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CartResponse;
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::CartId;
use crate::SalesError;

/// Use case for clearing all items from a cart
pub struct ClearCartUseCase {
    cart_repo: Arc<dyn CartRepository>,
}

impl ClearCartUseCase {
    pub fn new(cart_repo: Arc<dyn CartRepository>) -> Self {
        Self { cart_repo }
    }

    pub async fn execute(&self, cart_id: Uuid) -> Result<CartResponse, SalesError> {
        let cart_id_vo = CartId::from_uuid(cart_id);

        let mut cart = self
            .cart_repo
            .find_by_id_with_items(cart_id_vo)
            .await?
            .ok_or(SalesError::CartNotFound(cart_id))?;

        cart.clear()?;

        self.cart_repo.delete_items_by_cart(cart_id_vo).await?;
        self.cart_repo.update(&cart).await?;

        Ok(CartResponse::from(cart))
    }
}
