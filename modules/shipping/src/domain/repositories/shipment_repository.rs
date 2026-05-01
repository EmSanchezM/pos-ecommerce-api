use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::ShippingError;
use crate::domain::entities::Shipment;
use crate::domain::value_objects::{DriverId, ShipmentId, ShipmentStatus, ShippingMethodType};
use identity::StoreId;
use sales::SaleId;

#[derive(Debug, Clone, Default)]
pub struct ShipmentFilter {
    pub store_id: Option<StoreId>,
    pub sale_id: Option<SaleId>,
    pub status: Option<ShipmentStatus>,
    pub method_type: Option<ShippingMethodType>,
    pub driver_id: Option<DriverId>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

#[async_trait]
pub trait ShipmentRepository: Send + Sync {
    async fn save(&self, shipment: &Shipment) -> Result<(), ShippingError>;
    async fn find_by_id(&self, id: ShipmentId) -> Result<Option<Shipment>, ShippingError>;
    async fn find_by_sale_id(&self, sale_id: SaleId) -> Result<Option<Shipment>, ShippingError>;
    async fn find_by_tracking(
        &self,
        tracking_number: &str,
    ) -> Result<Option<Shipment>, ShippingError>;
    async fn find_by_pickup_code(
        &self,
        store_id: StoreId,
        pickup_code: &str,
    ) -> Result<Option<Shipment>, ShippingError>;
    async fn update(&self, shipment: &Shipment) -> Result<(), ShippingError>;
    async fn find_paginated(
        &self,
        filter: ShipmentFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Shipment>, i64), ShippingError>;
    /// All `ready_for_pickup` shipments past their expiration window.
    async fn find_expired_pickups(&self) -> Result<Vec<Shipment>, ShippingError>;
}
