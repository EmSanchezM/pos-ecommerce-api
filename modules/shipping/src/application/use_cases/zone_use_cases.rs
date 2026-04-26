//! ShippingZone CRUD use cases.

use std::sync::Arc;

use uuid::Uuid;

use crate::ShippingError;
use crate::application::dtos::{
    CreateShippingZoneCommand, ShippingZoneResponse, UpdateShippingZoneCommand,
};
use crate::domain::entities::ShippingZone;
use crate::domain::repositories::ShippingZoneRepository;
use crate::domain::value_objects::ShippingZoneId;
use identity::StoreId;

pub struct CreateShippingZoneUseCase {
    zone_repo: Arc<dyn ShippingZoneRepository>,
}

impl CreateShippingZoneUseCase {
    pub fn new(zone_repo: Arc<dyn ShippingZoneRepository>) -> Self {
        Self { zone_repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateShippingZoneCommand,
    ) -> Result<ShippingZoneResponse, ShippingError> {
        let zone = ShippingZone::create(
            StoreId::from_uuid(cmd.store_id),
            cmd.name,
            cmd.countries,
            cmd.states,
            cmd.zip_codes,
        );
        self.zone_repo.save(&zone).await?;
        Ok(ShippingZoneResponse::from(zone))
    }
}

pub struct UpdateShippingZoneUseCase {
    zone_repo: Arc<dyn ShippingZoneRepository>,
}

impl UpdateShippingZoneUseCase {
    pub fn new(zone_repo: Arc<dyn ShippingZoneRepository>) -> Self {
        Self { zone_repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateShippingZoneCommand,
    ) -> Result<ShippingZoneResponse, ShippingError> {
        let id = ShippingZoneId::from_uuid(cmd.zone_id);
        let mut zone = self
            .zone_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShippingZoneNotFound(cmd.zone_id))?;

        if let Some(name) = cmd.name {
            zone.set_name(name);
        }
        if let Some(c) = cmd.countries {
            zone.set_countries(c);
        }
        if let Some(s) = cmd.states {
            zone.set_states(s);
        }
        if let Some(z) = cmd.zip_codes {
            zone.set_zip_codes(z);
        }
        if let Some(active) = cmd.is_active {
            if active {
                zone.activate();
            } else {
                zone.deactivate();
            }
        }

        self.zone_repo.update(&zone).await?;
        Ok(ShippingZoneResponse::from(zone))
    }
}

pub struct DeleteShippingZoneUseCase {
    zone_repo: Arc<dyn ShippingZoneRepository>,
}

impl DeleteShippingZoneUseCase {
    pub fn new(zone_repo: Arc<dyn ShippingZoneRepository>) -> Self {
        Self { zone_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), ShippingError> {
        let zid = ShippingZoneId::from_uuid(id);
        if self.zone_repo.find_by_id(zid).await?.is_none() {
            return Err(ShippingError::ShippingZoneNotFound(id));
        }
        self.zone_repo.delete(zid).await
    }
}

pub struct ListShippingZonesUseCase {
    zone_repo: Arc<dyn ShippingZoneRepository>,
}

impl ListShippingZonesUseCase {
    pub fn new(zone_repo: Arc<dyn ShippingZoneRepository>) -> Self {
        Self { zone_repo }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
    ) -> Result<Vec<ShippingZoneResponse>, ShippingError> {
        let zones = self
            .zone_repo
            .find_by_store(StoreId::from_uuid(store_id))
            .await?;
        Ok(zones.into_iter().map(ShippingZoneResponse::from).collect())
    }
}
