use std::sync::Arc;

use uuid::Uuid;

use crate::BookingError;
use crate::application::dtos::{
    CreateResourceCommand, SetResourceCalendarCommand, UpdateResourceCommand,
};
use crate::domain::entities::{Resource, ResourceCalendar};
use crate::domain::repositories::{ResourceCalendarRepository, ResourceRepository};
use crate::domain::value_objects::ResourceId;

pub struct CreateResourceUseCase {
    resources: Arc<dyn ResourceRepository>,
}

impl CreateResourceUseCase {
    pub fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    pub async fn execute(&self, cmd: CreateResourceCommand) -> Result<Resource, BookingError> {
        if cmd.name.trim().is_empty() {
            return Err(BookingError::Validation("name is required".to_string()));
        }
        let resource = Resource::new(cmd.store_id, cmd.resource_type, cmd.name, cmd.color);
        self.resources.save(&resource).await?;
        Ok(resource)
    }
}

pub struct UpdateResourceUseCase {
    resources: Arc<dyn ResourceRepository>,
}

impl UpdateResourceUseCase {
    pub fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    pub async fn execute(
        &self,
        id: ResourceId,
        cmd: UpdateResourceCommand,
    ) -> Result<Resource, BookingError> {
        let mut resource = self
            .resources
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::ResourceNotFound(id.into_uuid()))?;
        resource.rename(cmd.name, cmd.color);
        self.resources.update(&resource).await?;
        Ok(resource)
    }
}

pub struct DeactivateResourceUseCase {
    resources: Arc<dyn ResourceRepository>,
}

impl DeactivateResourceUseCase {
    pub fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    pub async fn execute(&self, id: ResourceId) -> Result<(), BookingError> {
        let mut resource = self
            .resources
            .find_by_id(id)
            .await?
            .ok_or_else(|| BookingError::ResourceNotFound(id.into_uuid()))?;
        resource.deactivate();
        self.resources.update(&resource).await?;
        Ok(())
    }
}

pub struct ListResourcesUseCase {
    resources: Arc<dyn ResourceRepository>,
}

impl ListResourcesUseCase {
    pub fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<Resource>, BookingError> {
        self.resources.list_by_store(store_id, only_active).await
    }
}

pub struct SetResourceCalendarUseCase {
    resources: Arc<dyn ResourceRepository>,
    calendars: Arc<dyn ResourceCalendarRepository>,
}

impl SetResourceCalendarUseCase {
    pub fn new(
        resources: Arc<dyn ResourceRepository>,
        calendars: Arc<dyn ResourceCalendarRepository>,
    ) -> Self {
        Self {
            resources,
            calendars,
        }
    }

    pub async fn execute(
        &self,
        resource_id: ResourceId,
        cmd: SetResourceCalendarCommand,
    ) -> Result<Vec<ResourceCalendar>, BookingError> {
        if self.resources.find_by_id(resource_id).await?.is_none() {
            return Err(BookingError::ResourceNotFound(resource_id.into_uuid()));
        }
        let mut entries = Vec::with_capacity(cmd.windows.len());
        for w in cmd.windows {
            entries.push(ResourceCalendar::new(
                resource_id,
                w.day_of_week,
                w.start_time,
                w.end_time,
            )?);
        }
        self.calendars
            .replace_for_resource(resource_id, &entries)
            .await?;
        Ok(entries)
    }
}

pub struct GetResourceCalendarUseCase {
    calendars: Arc<dyn ResourceCalendarRepository>,
}

impl GetResourceCalendarUseCase {
    pub fn new(calendars: Arc<dyn ResourceCalendarRepository>) -> Self {
        Self { calendars }
    }

    pub async fn execute(
        &self,
        resource_id: ResourceId,
    ) -> Result<Vec<ResourceCalendar>, BookingError> {
        self.calendars.find_by_resource(resource_id).await
    }
}
