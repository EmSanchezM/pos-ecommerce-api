use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::ServiceOrder;
use crate::domain::value_objects::{AssetId, ServiceOrderId, ServiceOrderStatus};

#[derive(Debug, Clone, Default)]
pub struct ListServiceOrdersFilters {
    pub store_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub asset_id: Option<AssetId>,
    pub status: Option<ServiceOrderStatus>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[async_trait]
pub trait ServiceOrderRepository: Send + Sync {
    async fn save(&self, order: &ServiceOrder) -> Result<(), ServiceOrdersError>;
    async fn update(&self, order: &ServiceOrder) -> Result<(), ServiceOrdersError>;
    async fn find_by_id(
        &self,
        id: ServiceOrderId,
    ) -> Result<Option<ServiceOrder>, ServiceOrdersError>;
    async fn find_by_public_token(
        &self,
        token: &str,
    ) -> Result<Option<ServiceOrder>, ServiceOrdersError>;
    async fn list(
        &self,
        filters: ListServiceOrdersFilters,
    ) -> Result<Vec<ServiceOrder>, ServiceOrdersError>;
    async fn list_by_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<Vec<ServiceOrder>, ServiceOrdersError>;
}
