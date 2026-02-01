//! Create cart use case

use std::sync::Arc;

use crate::application::dtos::{CartResponse, CreateCartCommand};
use crate::domain::entities::Cart;
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::CustomerId;
use crate::SalesError;
use identity::StoreId;
use inventory::Currency;

/// Use case for creating a new shopping cart
pub struct CreateCartUseCase {
    cart_repo: Arc<dyn CartRepository>,
}

impl CreateCartUseCase {
    pub fn new(cart_repo: Arc<dyn CartRepository>) -> Self {
        Self { cart_repo }
    }

    pub async fn execute(&self, cmd: CreateCartCommand) -> Result<CartResponse, SalesError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let customer_id = cmd.customer_id.map(CustomerId::from_uuid);
        let currency = Currency::new(&cmd.currency).map_err(|_| SalesError::InvalidCurrency)?;

        let cart = Cart::create(store_id, customer_id, cmd.session_id, currency);

        self.cart_repo.save(&cart).await?;

        Ok(CartResponse::from(cart))
    }
}
