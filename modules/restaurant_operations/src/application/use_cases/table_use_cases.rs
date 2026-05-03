use std::sync::Arc;

use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::application::dtos::{
    CreateRestaurantTableCommand, SetTableStatusCommand, UpdateRestaurantTableCommand,
};
use crate::domain::entities::RestaurantTable;
use crate::domain::repositories::RestaurantTableRepository;
use crate::domain::value_objects::RestaurantTableId;

pub struct CreateRestaurantTableUseCase {
    tables: Arc<dyn RestaurantTableRepository>,
}

impl CreateRestaurantTableUseCase {
    pub fn new(tables: Arc<dyn RestaurantTableRepository>) -> Self {
        Self { tables }
    }

    pub async fn execute(
        &self,
        cmd: CreateRestaurantTableCommand,
    ) -> Result<RestaurantTable, RestaurantOperationsError> {
        let table = RestaurantTable::new(cmd.store_id, cmd.label, cmd.capacity, cmd.notes)?;
        self.tables.save(&table).await?;
        Ok(table)
    }
}

pub struct UpdateRestaurantTableUseCase {
    tables: Arc<dyn RestaurantTableRepository>,
}

impl UpdateRestaurantTableUseCase {
    pub fn new(tables: Arc<dyn RestaurantTableRepository>) -> Self {
        Self { tables }
    }

    pub async fn execute(
        &self,
        id: RestaurantTableId,
        cmd: UpdateRestaurantTableCommand,
    ) -> Result<RestaurantTable, RestaurantOperationsError> {
        let mut table = self
            .tables
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TableNotFound(id.into_uuid()))?;
        table.rename(cmd.label, cmd.capacity, cmd.notes);
        self.tables.update(&table).await?;
        Ok(table)
    }
}

pub struct SetTableStatusUseCase {
    tables: Arc<dyn RestaurantTableRepository>,
}

impl SetTableStatusUseCase {
    pub fn new(tables: Arc<dyn RestaurantTableRepository>) -> Self {
        Self { tables }
    }

    pub async fn execute(
        &self,
        id: RestaurantTableId,
        cmd: SetTableStatusCommand,
    ) -> Result<RestaurantTable, RestaurantOperationsError> {
        let mut table = self
            .tables
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TableNotFound(id.into_uuid()))?;
        table.set_status(cmd.status, cmd.current_ticket_id);
        self.tables.update(&table).await?;
        Ok(table)
    }
}

pub struct DeactivateRestaurantTableUseCase {
    tables: Arc<dyn RestaurantTableRepository>,
}

impl DeactivateRestaurantTableUseCase {
    pub fn new(tables: Arc<dyn RestaurantTableRepository>) -> Self {
        Self { tables }
    }

    pub async fn execute(&self, id: RestaurantTableId) -> Result<(), RestaurantOperationsError> {
        let mut table = self
            .tables
            .find_by_id(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::TableNotFound(id.into_uuid()))?;
        table.deactivate();
        self.tables.update(&table).await?;
        Ok(())
    }
}

pub struct ListRestaurantTablesUseCase {
    tables: Arc<dyn RestaurantTableRepository>,
}

impl ListRestaurantTablesUseCase {
    pub fn new(tables: Arc<dyn RestaurantTableRepository>) -> Self {
        Self { tables }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<RestaurantTable>, RestaurantOperationsError> {
        self.tables.list_by_store(store_id, only_active).await
    }
}
