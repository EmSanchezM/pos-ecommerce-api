use std::sync::Arc;

use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::application::dtos::IntakeServiceOrderCommand;
use crate::domain::entities::{Asset, ServiceOrder};
use crate::domain::repositories::{
    AssetRepository, ListServiceOrdersFilters, ServiceOrderRepository,
};
use crate::domain::value_objects::{AssetId, ServiceOrderId};

pub struct IntakeServiceOrderUseCase {
    assets: Arc<dyn AssetRepository>,
    orders: Arc<dyn ServiceOrderRepository>,
}

impl IntakeServiceOrderUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>, orders: Arc<dyn ServiceOrderRepository>) -> Self {
        Self { assets, orders }
    }

    pub async fn execute(
        &self,
        cmd: IntakeServiceOrderCommand,
        actor_id: Option<Uuid>,
    ) -> Result<ServiceOrder, ServiceOrdersError> {
        let asset_id = AssetId::from_uuid(cmd.asset_id);
        let asset = self
            .assets
            .find_by_id(asset_id)
            .await?
            .ok_or_else(|| ServiceOrdersError::AssetNotFound(cmd.asset_id))?;
        if !asset.is_active() {
            return Err(ServiceOrdersError::InactiveAsset(cmd.asset_id));
        }
        if asset.store_id() != cmd.store_id {
            return Err(ServiceOrdersError::Validation(
                "asset does not belong to the supplied store".to_string(),
            ));
        }
        let order = ServiceOrder::intake(
            cmd.store_id,
            asset_id,
            cmd.customer_id,
            cmd.customer_name,
            cmd.customer_email,
            cmd.customer_phone,
            cmd.priority.unwrap_or_default(),
            cmd.intake_notes,
            actor_id,
            cmd.promised_at,
        )?;
        self.orders.save(&order).await?;
        Ok(order)
    }
}

pub struct ListServiceOrdersUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
}

impl ListServiceOrdersUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>) -> Self {
        Self { orders }
    }

    pub async fn execute(
        &self,
        filters: ListServiceOrdersFilters,
    ) -> Result<Vec<ServiceOrder>, ServiceOrdersError> {
        self.orders.list(filters).await
    }
}

pub struct GetServiceOrderUseCase {
    orders: Arc<dyn ServiceOrderRepository>,
}

impl GetServiceOrderUseCase {
    pub fn new(orders: Arc<dyn ServiceOrderRepository>) -> Self {
        Self { orders }
    }

    pub async fn execute(&self, id: ServiceOrderId) -> Result<ServiceOrder, ServiceOrdersError> {
        self.orders
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::ServiceOrderNotFound(id.into_uuid()))
    }
}

pub struct GetAssetWithHistoryUseCase {
    assets: Arc<dyn AssetRepository>,
    orders: Arc<dyn ServiceOrderRepository>,
}

impl GetAssetWithHistoryUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>, orders: Arc<dyn ServiceOrderRepository>) -> Self {
        Self { assets, orders }
    }

    pub async fn execute(
        &self,
        id: AssetId,
    ) -> Result<(Asset, Vec<ServiceOrder>), ServiceOrdersError> {
        let asset = self
            .assets
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::AssetNotFound(id.into_uuid()))?;
        let history = self.orders.list_by_asset(id).await?;
        Ok((asset, history))
    }
}
