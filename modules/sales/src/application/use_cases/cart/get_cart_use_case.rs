//! Get cart use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CartResponse;
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::CartId;
use crate::SalesError;

/// Use case for retrieving a cart by ID
pub struct GetCartUseCase {
    cart_repo: Arc<dyn CartRepository>,
}

impl GetCartUseCase {
    pub fn new(cart_repo: Arc<dyn CartRepository>) -> Self {
        Self { cart_repo }
    }

    pub async fn execute(&self, cart_id: Uuid) -> Result<CartResponse, SalesError> {
        let id = CartId::from_uuid(cart_id);

        let cart = self
            .cart_repo
            .find_by_id_with_items(id)
            .await?
            .ok_or(SalesError::CartNotFound(cart_id))?;

        Ok(CartResponse::from(cart))
    }
}
