//! Driver - store-owned (or subcontracted) delivery person.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{DriverId, DriverStatus, VehicleType};
use identity::{StoreId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    id: DriverId,
    store_id: StoreId,
    user_id: Option<UserId>,
    name: String,
    phone: String,
    vehicle_type: VehicleType,
    license_plate: Option<String>,
    is_active: bool,
    current_status: DriverStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Driver {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        user_id: Option<UserId>,
        name: String,
        phone: String,
        vehicle_type: VehicleType,
        license_plate: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DriverId::new(),
            store_id,
            user_id,
            name,
            phone,
            vehicle_type,
            license_plate,
            is_active: true,
            current_status: DriverStatus::Offline,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: DriverId,
        store_id: StoreId,
        user_id: Option<UserId>,
        name: String,
        phone: String,
        vehicle_type: VehicleType,
        license_plate: Option<String>,
        is_active: bool,
        current_status: DriverStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            user_id,
            name,
            phone,
            vehicle_type,
            license_plate,
            is_active,
            current_status,
            created_at,
            updated_at,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.touch();
    }
    pub fn set_phone(&mut self, phone: String) {
        self.phone = phone;
        self.touch();
    }
    pub fn set_vehicle(&mut self, vehicle_type: VehicleType, license_plate: Option<String>) {
        self.vehicle_type = vehicle_type;
        self.license_plate = license_plate;
        self.touch();
    }
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }
    pub fn set_status(&mut self, status: DriverStatus) {
        self.current_status = status;
        self.touch();
    }

    pub fn is_assignable(&self) -> bool {
        self.is_active && matches!(self.current_status, DriverStatus::Available)
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> DriverId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn user_id(&self) -> Option<UserId> {
        self.user_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn phone(&self) -> &str {
        &self.phone
    }
    pub fn vehicle_type(&self) -> VehicleType {
        self.vehicle_type
    }
    pub fn license_plate(&self) -> Option<&str> {
        self.license_plate.as_deref()
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn current_status(&self) -> DriverStatus {
        self.current_status
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
