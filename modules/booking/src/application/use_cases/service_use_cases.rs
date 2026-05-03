use std::sync::Arc;

use uuid::Uuid;

use crate::BookingError;
use crate::application::dtos::{
    AssignServiceResourcesCommand, CreateServiceCommand, UpdateServiceCommand,
};
use crate::domain::entities::Service;
use crate::domain::repositories::{ResourceRepository, ServiceRepository};
use crate::domain::value_objects::{ResourceId, ServiceId};

pub struct CreateServiceUseCase {
    services: Arc<dyn ServiceRepository>,
}

impl CreateServiceUseCase {
    pub fn new(services: Arc<dyn ServiceRepository>) -> Self {
        Self { services }
    }

    pub async fn execute(&self, cmd: CreateServiceCommand) -> Result<Service, BookingError> {
        let service = Service::new(
            cmd.store_id,
            cmd.name,
            cmd.description,
            cmd.duration_minutes,
            cmd.price,
            cmd.buffer_minutes_before.unwrap_or(0),
            cmd.buffer_minutes_after.unwrap_or(0),
            cmd.requires_deposit.unwrap_or(false),
            cmd.deposit_amount,
        )?;
        self.services.save(&service).await?;
        Ok(service)
    }
}

pub struct UpdateServiceUseCase {
    services: Arc<dyn ServiceRepository>,
}

impl UpdateServiceUseCase {
    pub fn new(services: Arc<dyn ServiceRepository>) -> Self {
        Self { services }
    }

    pub async fn execute(
        &self,
        id: ServiceId,
        cmd: UpdateServiceCommand,
    ) -> Result<Service, BookingError> {
        let existing = self
            .services
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::ServiceNotFound(id.into_uuid()))?;
        // Reconstitute with the same id/store/timestamps but new mutable fields.
        let updated = Service::reconstitute(
            existing.id(),
            existing.store_id(),
            cmd.name,
            cmd.description,
            cmd.duration_minutes,
            cmd.price,
            cmd.buffer_minutes_before.unwrap_or(0),
            cmd.buffer_minutes_after.unwrap_or(0),
            cmd.requires_deposit.unwrap_or(false),
            cmd.deposit_amount,
            existing.is_active(),
            existing.created_at(),
            chrono::Utc::now(),
        );
        self.services.update(&updated).await?;
        Ok(updated)
    }
}

pub struct DeactivateServiceUseCase {
    services: Arc<dyn ServiceRepository>,
}

impl DeactivateServiceUseCase {
    pub fn new(services: Arc<dyn ServiceRepository>) -> Self {
        Self { services }
    }

    pub async fn execute(&self, id: ServiceId) -> Result<(), BookingError> {
        let mut service = self
            .services
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::ServiceNotFound(id.into_uuid()))?;
        service.deactivate();
        self.services.update(&service).await?;
        Ok(())
    }
}

pub struct ListServicesUseCase {
    services: Arc<dyn ServiceRepository>,
}

impl ListServicesUseCase {
    pub fn new(services: Arc<dyn ServiceRepository>) -> Self {
        Self { services }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<Service>, BookingError> {
        self.services.list_by_store(store_id, only_active).await
    }
}

pub struct AssignServiceResourcesUseCase {
    services: Arc<dyn ServiceRepository>,
    resources: Arc<dyn ResourceRepository>,
}

impl AssignServiceResourcesUseCase {
    pub fn new(
        services: Arc<dyn ServiceRepository>,
        resources: Arc<dyn ResourceRepository>,
    ) -> Self {
        Self {
            services,
            resources,
        }
    }

    pub async fn execute(
        &self,
        service_id: ServiceId,
        cmd: AssignServiceResourcesCommand,
    ) -> Result<(), BookingError> {
        if self.services.find_by_id(service_id).await?.is_none() {
            return Err(BookingError::ServiceNotFound(service_id.into_uuid()));
        }
        let mut typed_ids = Vec::with_capacity(cmd.resource_ids.len());
        for rid in &cmd.resource_ids {
            let resource_id = ResourceId::from_uuid(*rid);
            if self.resources.find_by_id(resource_id).await?.is_none() {
                return Err(BookingError::ResourceNotFound(*rid));
            }
            typed_ids.push(resource_id);
        }
        self.services.assign_resources(service_id, &typed_ids).await
    }
}
