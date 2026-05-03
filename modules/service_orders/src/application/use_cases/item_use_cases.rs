//! Item lifecycle. Every mutation re-syncs the cached `total_amount` on the
//! parent `ServiceOrder` so list views can show running totals without an
//! extra aggregate query.

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::ServiceOrdersError;
use crate::application::dtos::{AddItemCommand, UpdateItemCommand};
use crate::domain::entities::ServiceOrderItem;
use crate::domain::repositories::{ServiceOrderItemRepository, ServiceOrderRepository};
use crate::domain::value_objects::{ServiceOrderId, ServiceOrderItemId};

pub struct AddItemUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    items: Arc<dyn ServiceOrderItemRepository>,
}

impl AddItemUseCase {
    pub fn new(
        orders: Arc<dyn ServiceOrderRepository>,
        items: Arc<dyn ServiceOrderItemRepository>,
    ) -> Self {
        Self { orders, items }
    }

    pub async fn execute(
        &self,
        order_id: ServiceOrderId,
        cmd: AddItemCommand,
    ) -> Result<ServiceOrderItem, ServiceOrdersError> {
        let mut order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;
        if order.status().is_terminal() {
            return Err(ServiceOrdersError::CannotModifyTerminalOrder);
        }

        let item = ServiceOrderItem::new(
            order_id,
            cmd.item_type,
            cmd.description,
            cmd.quantity,
            cmd.unit_price,
            cmd.product_id,
            cmd.variant_id,
            cmd.tax_rate.unwrap_or(Decimal::ZERO),
        )?;
        self.items.save(&item).await?;

        let new_total = self.items.subtotal_by_order(order_id).await?;
        order.recompute_total(new_total);
        self.orders.update(&order).await?;
        Ok(item)
    }
}

pub struct UpdateItemUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    items: Arc<dyn ServiceOrderItemRepository>,
}

impl UpdateItemUseCase {
    pub fn new(
        orders: Arc<dyn ServiceOrderRepository>,
        items: Arc<dyn ServiceOrderItemRepository>,
    ) -> Self {
        Self { orders, items }
    }

    pub async fn execute(
        &self,
        item_id: ServiceOrderItemId,
        cmd: UpdateItemCommand,
    ) -> Result<ServiceOrderItem, ServiceOrdersError> {
        let existing = self
            .items
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ItemNotFound(item_id.into_uuid()))?;
        let order_id = existing.service_order_id();
        let mut order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;
        if order.status().is_terminal() {
            return Err(ServiceOrdersError::CannotModifyTerminalOrder);
        }

        // Build a brand-new item with the same id/type/created_at.
        let updated = ServiceOrderItem::new(
            order_id,
            existing.item_type(),
            cmd.description,
            cmd.quantity,
            cmd.unit_price,
            cmd.product_id,
            cmd.variant_id,
            cmd.tax_rate.unwrap_or(Decimal::ZERO),
        )?;
        // Re-stamp the original id and created_at so we mutate in place.
        let updated = ServiceOrderItem::reconstitute(
            existing.id(),
            order_id,
            existing.item_type(),
            updated.description().to_string(),
            updated.quantity(),
            updated.unit_price(),
            updated.total(),
            updated.product_id(),
            updated.variant_id(),
            updated.tax_rate(),
            updated.tax_amount(),
            existing.created_at(),
        );
        self.items.update(&updated).await?;

        let new_total = self.items.subtotal_by_order(order_id).await?;
        order.recompute_total(new_total);
        self.orders.update(&order).await?;
        Ok(updated)
    }
}

pub struct RemoveItemUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
    items: Arc<dyn ServiceOrderItemRepository>,
}

impl RemoveItemUseCase {
    pub fn new(
        orders: Arc<dyn ServiceOrderRepository>,
        items: Arc<dyn ServiceOrderItemRepository>,
    ) -> Self {
        Self { orders, items }
    }

    pub async fn execute(&self, item_id: ServiceOrderItemId) -> Result<(), ServiceOrdersError> {
        let existing = self
            .items
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ItemNotFound(item_id.into_uuid()))?;
        let order_id = existing.service_order_id();
        let mut order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(order_id.into_uuid()))?;
        if order.status().is_terminal() {
            return Err(ServiceOrdersError::CannotModifyTerminalOrder);
        }
        self.items.delete(item_id).await?;
        let new_total = self.items.subtotal_by_order(order_id).await?;
        order.recompute_total(new_total);
        self.orders.update(&order).await?;
        Ok(())
    }
}

pub struct ListItemsUseCase {
    items: Arc<dyn ServiceOrderItemRepository>,
}

impl ListItemsUseCase {
    pub fn new(items: Arc<dyn ServiceOrderItemRepository>) -> Self {
        Self { items }
    }

    pub async fn execute(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<ServiceOrderItem>, ServiceOrdersError> {
        self.items.list_by_order(order_id).await
    }
}
