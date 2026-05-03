use std::sync::Arc;

use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::application::dtos::{CreateKitchenStationCommand, UpdateKitchenStationCommand};
use crate::domain::entities::KitchenStation;
use crate::domain::repositories::KitchenStationRepository;
use crate::domain::value_objects::KitchenStationId;

pub struct CreateKitchenStationUseCase {
    stations: Arc<dyn KitchenStationRepository>,
}

impl CreateKitchenStationUseCase {
    pub fn new(stations: Arc<dyn KitchenStationRepository>) -> Self {
        Self { stations }
    }

    pub async fn execute(
        &self,
        cmd: CreateKitchenStationCommand,
    ) -> Result<KitchenStation, RestaurantOperationsError> {
        let station = KitchenStation::new(
            cmd.store_id,
            cmd.name,
            cmd.color,
            cmd.sort_order.unwrap_or(0),
        )?;
        self.stations.save(&station).await?;
        Ok(station)
    }
}

pub struct UpdateKitchenStationUseCase {
    stations: Arc<dyn KitchenStationRepository>,
}

impl UpdateKitchenStationUseCase {
    pub fn new(stations: Arc<dyn KitchenStationRepository>) -> Self {
        Self { stations }
    }

    pub async fn execute(
        &self,
        id: KitchenStationId,
        cmd: UpdateKitchenStationCommand,
    ) -> Result<KitchenStation, RestaurantOperationsError> {
        let mut station = self
            .stations
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::StationNotFound(id.into_uuid()))?;
        station.rename(cmd.name, cmd.color, cmd.sort_order.unwrap_or(0));
        self.stations.update(&station).await?;
        Ok(station)
    }
}

pub struct DeactivateKitchenStationUseCase {
    stations: Arc<dyn KitchenStationRepository>,
}

impl DeactivateKitchenStationUseCase {
    pub fn new(stations: Arc<dyn KitchenStationRepository>) -> Self {
        Self { stations }
    }

    pub async fn execute(&self, id: KitchenStationId) -> Result<(), RestaurantOperationsError> {
        let mut station = self
            .stations
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::StationNotFound(id.into_uuid()))?;
        station.deactivate();
        self.stations.update(&station).await?;
        Ok(())
    }
}

pub struct ListKitchenStationsUseCase {
    stations: Arc<dyn KitchenStationRepository>,
}

impl ListKitchenStationsUseCase {
    pub fn new(stations: Arc<dyn KitchenStationRepository>) -> Self {
        Self { stations }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<KitchenStation>, RestaurantOperationsError> {
        self.stations.list_by_store(store_id, only_active).await
    }
}
