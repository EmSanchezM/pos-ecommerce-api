//! Add cart item use case

use std::sync::Arc;

use crate::application::dtos::{AddCartItemCommand, CartResponse};
use crate::domain::entities::CartItem;
use crate::domain::repositories::CartRepository;
use crate::domain::value_objects::CartId;
use crate::SalesError;
use inventory::{ProductId, UnitOfMeasure, VariantId};

/// Use case for adding an item to a cart
pub struct AddCartItemUseCase {
    cart_repo: Arc<dyn CartRepository>,
}

impl AddCartItemUseCase {
    pub fn new(cart_repo: Arc<dyn CartRepository>) -> Self {
        Self { cart_repo }
    }

    pub async fn execute(&self, cmd: AddCartItemCommand) -> Result<CartResponse, SalesError> {
        let cart_id = CartId::from_uuid(cmd.cart_id);

        let mut cart = self
            .cart_repo
            .find_by_id_with_items(cart_id)
            .await?
            .ok_or(SalesError::CartNotFound(cmd.cart_id))?;

        let uom: UnitOfMeasure = cmd.unit_of_measure.parse()
            .map_err(|_| SalesError::InvalidUnitOfMeasure)?;

        let item = CartItem::create(
            cart_id,
            ProductId::from_uuid(cmd.product_id),
            cmd.variant_id.map(VariantId::from_uuid),
            cmd.sku,
            cmd.name,
            cmd.quantity,
            uom,
            cmd.unit_price,
            cmd.tax_rate,
        )?;

        cart.add_item(item.clone())?;

        self.cart_repo.save_item(&item).await?;
        self.cart_repo.update(&cart).await?;

        Ok(CartResponse::from(cart))
    }
}
