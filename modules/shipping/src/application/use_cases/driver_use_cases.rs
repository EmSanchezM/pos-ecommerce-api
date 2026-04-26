//! Driver CRUD.

use std::str::FromStr;
use std::sync::Arc;

use uuid::Uuid;

use crate::ShippingError;
use crate::application::dtos::{CreateDriverCommand, DriverResponse, UpdateDriverCommand};
use crate::domain::entities::Driver;
use crate::domain::repositories::DriverRepository;
use crate::domain::value_objects::{DriverId, DriverStatus, VehicleType};
use identity::{StoreId, UserId};

pub struct CreateDriverUseCase {
    driver_repo: Arc<dyn DriverRepository>,
}

impl CreateDriverUseCase {
    pub fn new(driver_repo: Arc<dyn DriverRepository>) -> Self {
        Self { driver_repo }
    }

    pub async fn execute(&self, cmd: CreateDriverCommand) -> Result<DriverResponse, ShippingError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let vehicle_type = VehicleType::from_str(&cmd.vehicle_type)?;

        // Reject duplicate phone in this store.
        let existing = self.driver_repo.find_by_store(store_id).await?;
        if existing.iter().any(|d| d.phone() == cmd.phone) {
            return Err(ShippingError::DuplicateDriverPhone(cmd.phone));
        }

        let driver = Driver::create(
            store_id,
            cmd.user_id.map(UserId::from_uuid),
            cmd.name,
            cmd.phone,
            vehicle_type,
            cmd.license_plate,
        );
        self.driver_repo.save(&driver).await?;
        Ok(DriverResponse::from(driver))
    }
}

pub struct UpdateDriverUseCase {
    driver_repo: Arc<dyn DriverRepository>,
}

impl UpdateDriverUseCase {
    pub fn new(driver_repo: Arc<dyn DriverRepository>) -> Self {
        Self { driver_repo }
    }

    pub async fn execute(&self, cmd: UpdateDriverCommand) -> Result<DriverResponse, ShippingError> {
        let id = DriverId::from_uuid(cmd.driver_id);
        let mut driver = self
            .driver_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::DriverNotFound(cmd.driver_id))?;

        if let Some(name) = cmd.name {
            driver.set_name(name);
        }
        if let Some(phone) = cmd.phone {
            driver.set_phone(phone);
        }
        if let Some(vt) = cmd.vehicle_type {
            let parsed = VehicleType::from_str(&vt)?;
            let plate = match cmd.license_plate.clone() {
                Some(p) => p,
                None => driver.license_plate().map(str::to_string),
            };
            driver.set_vehicle(parsed, plate);
        } else if let Some(plate) = cmd.license_plate {
            driver.set_vehicle(driver.vehicle_type(), plate);
        }
        if let Some(active) = cmd.is_active {
            if active {
                driver.activate();
            } else {
                driver.deactivate();
            }
        }
        if let Some(status) = cmd.current_status {
            driver.set_status(DriverStatus::from_str(&status)?);
        }

        self.driver_repo.update(&driver).await?;
        Ok(DriverResponse::from(driver))
    }
}

pub struct DeleteDriverUseCase {
    driver_repo: Arc<dyn DriverRepository>,
}

impl DeleteDriverUseCase {
    pub fn new(driver_repo: Arc<dyn DriverRepository>) -> Self {
        Self { driver_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), ShippingError> {
        let did = DriverId::from_uuid(id);
        if self.driver_repo.find_by_id(did).await?.is_none() {
            return Err(ShippingError::DriverNotFound(id));
        }
        self.driver_repo.delete(did).await
    }
}

pub struct ListDriversUseCase {
    driver_repo: Arc<dyn DriverRepository>,
}

impl ListDriversUseCase {
    pub fn new(driver_repo: Arc<dyn DriverRepository>) -> Self {
        Self { driver_repo }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_available: bool,
    ) -> Result<Vec<DriverResponse>, ShippingError> {
        let store_id = StoreId::from_uuid(store_id);
        let drivers = if only_available {
            self.driver_repo.find_available_by_store(store_id).await?
        } else {
            self.driver_repo.find_by_store(store_id).await?
        };
        Ok(drivers.into_iter().map(DriverResponse::from).collect())
    }
}
