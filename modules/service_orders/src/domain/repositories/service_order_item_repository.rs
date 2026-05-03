use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::ServiceOrdersError;
use crate::domain::entities::ServiceOrderItem;
use crate::domain::value_objects::{ServiceOrderId, ServiceOrderItemId};

#[async_trait]
pub trait ServiceOrderItemRepository: Send + Sync {
    async fn save(&self, item: &ServiceOrderItem) -> Result<(), ServiceOrdersError>;
    async fn update(&self, item: &ServiceOrderItem) -> Result<(), ServiceOrdersError>;
    async fn delete(&self, id: ServiceOrderItemId) -> Result<(), ServiceOrdersError>;
    async fn find_by_id(
        &self,
        id: ServiceOrderItemId,
    ) -> Result<Option<ServiceOrderItem>, ServiceOrdersError>;
    async fn list_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<ServiceOrderItem>, ServiceOrdersError>;
    /// SUM(total) for the items of an order — used by use cases to refresh
    /// the cached `service_orders.total_amount` after each mutation.
    async fn subtotal_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Decimal, ServiceOrdersError>;
}
