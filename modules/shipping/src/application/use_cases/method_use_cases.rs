//! ShippingMethod CRUD use cases.

use std::str::FromStr;
use std::sync::Arc;

use uuid::Uuid;

use crate::ShippingError;
use crate::application::dtos::{
    CreateShippingMethodCommand, ShippingMethodResponse, UpdateShippingMethodCommand,
};
use crate::domain::entities::ShippingMethod;
use crate::domain::repositories::ShippingMethodRepository;
use crate::domain::value_objects::{ShippingMethodId, ShippingMethodType};
use identity::StoreId;

pub struct CreateShippingMethodUseCase {
    method_repo: Arc<dyn ShippingMethodRepository>,
}

impl CreateShippingMethodUseCase {
    pub fn new(method_repo: Arc<dyn ShippingMethodRepository>) -> Self {
        Self { method_repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateShippingMethodCommand,
    ) -> Result<ShippingMethodResponse, ShippingError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let method_type = ShippingMethodType::from_str(&cmd.method_type)?;

        // Reject duplicate code in this store.
        let existing = self.method_repo.find_by_store(store_id).await?;
        if existing.iter().any(|m| m.code() == cmd.code) {
            return Err(ShippingError::DuplicateMethodCode(cmd.code));
        }

        let method = ShippingMethod::create(
            store_id,
            cmd.name,
            cmd.code,
            method_type,
            cmd.description,
            cmd.estimated_days_min,
            cmd.estimated_days_max,
            cmd.sort_order,
        );
        self.method_repo.save(&method).await?;
        Ok(ShippingMethodResponse::from(method))
    }
}

pub struct UpdateShippingMethodUseCase {
    method_repo: Arc<dyn ShippingMethodRepository>,
}

impl UpdateShippingMethodUseCase {
    pub fn new(method_repo: Arc<dyn ShippingMethodRepository>) -> Self {
        Self { method_repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateShippingMethodCommand,
    ) -> Result<ShippingMethodResponse, ShippingError> {
        let id = ShippingMethodId::from_uuid(cmd.method_id);
        let mut method = self
            .method_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShippingMethodNotFound(cmd.method_id))?;

        if let Some(name) = cmd.name {
            method.set_name(name);
        }
        if let Some(desc) = cmd.description {
            method.set_description(desc);
        }
        if cmd.estimated_days_min.is_some() || cmd.estimated_days_max.is_some() {
            let min = cmd
                .estimated_days_min
                .unwrap_or_else(|| method.estimated_days_min());
            let max = cmd
                .estimated_days_max
                .unwrap_or_else(|| method.estimated_days_max());
            method.set_estimated_days(min, max);
        }
        if let Some(order) = cmd.sort_order {
            method.set_sort_order(order);
        }
        if let Some(active) = cmd.is_active {
            if active {
                method.activate();
            } else {
                method.deactivate();
            }
        }

        self.method_repo.update(&method).await?;
        Ok(ShippingMethodResponse::from(method))
    }
}

pub struct DeleteShippingMethodUseCase {
    method_repo: Arc<dyn ShippingMethodRepository>,
}

impl DeleteShippingMethodUseCase {
    pub fn new(method_repo: Arc<dyn ShippingMethodRepository>) -> Self {
        Self { method_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), ShippingError> {
        let mid = ShippingMethodId::from_uuid(id);
        if self.method_repo.find_by_id(mid).await?.is_none() {
            return Err(ShippingError::ShippingMethodNotFound(id));
        }
        self.method_repo.delete(mid).await
    }
}

pub struct ListShippingMethodsUseCase {
    method_repo: Arc<dyn ShippingMethodRepository>,
}

impl ListShippingMethodsUseCase {
    pub fn new(method_repo: Arc<dyn ShippingMethodRepository>) -> Self {
        Self { method_repo }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
    ) -> Result<Vec<ShippingMethodResponse>, ShippingError> {
        let methods = self
            .method_repo
            .find_by_store(StoreId::from_uuid(store_id))
            .await?;
        Ok(methods
            .into_iter()
            .map(ShippingMethodResponse::from)
            .collect())
    }
}
