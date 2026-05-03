use std::sync::Arc;

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::application::dtos::{
    AssignProductModifierGroupsCommand, CreateModifierCommand, CreateModifierGroupCommand,
    UpdateModifierCommand, UpdateModifierGroupCommand,
};
use crate::domain::entities::{MenuModifier, MenuModifierGroup};
use crate::domain::repositories::{MenuModifierRepository, ModifierGroupWithModifiers};
use crate::domain::value_objects::{MenuModifierGroupId, MenuModifierId};

pub struct CreateModifierGroupUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl CreateModifierGroupUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateModifierGroupCommand,
    ) -> Result<MenuModifierGroup, RestaurantOperationsError> {
        let group = MenuModifierGroup::new(
            cmd.store_id,
            cmd.name,
            cmd.min_select,
            cmd.max_select,
            cmd.sort_order.unwrap_or(0),
        )?;
        self.repo.save_group(&group).await?;
        Ok(group)
    }
}

pub struct UpdateModifierGroupUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl UpdateModifierGroupUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: MenuModifierGroupId,
        cmd: UpdateModifierGroupCommand,
    ) -> Result<MenuModifierGroup, RestaurantOperationsError> {
        let mut group = self
            .repo
            .find_group(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::ModifierGroupNotFound(id.into_uuid()))?;
        group.update(
            cmd.name,
            cmd.min_select,
            cmd.max_select,
            cmd.sort_order.unwrap_or(0),
        )?;
        self.repo.update_group(&group).await?;
        Ok(group)
    }
}

pub struct AddModifierUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl AddModifierUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        group_id: MenuModifierGroupId,
        cmd: CreateModifierCommand,
    ) -> Result<MenuModifier, RestaurantOperationsError> {
        if self.repo.find_group(group_id).await?.is_none() {
            return Err(RestaurantOperationsError::ModifierGroupNotFound(
                group_id.into_uuid(),
            ));
        }
        let modifier = MenuModifier::new(
            group_id,
            cmd.name,
            cmd.price_delta.unwrap_or(Decimal::ZERO),
            cmd.sort_order.unwrap_or(0),
        )?;
        self.repo.save_modifier(&modifier).await?;
        Ok(modifier)
    }
}

pub struct UpdateModifierUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl UpdateModifierUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        id: MenuModifierId,
        cmd: UpdateModifierCommand,
    ) -> Result<MenuModifier, RestaurantOperationsError> {
        let mut modifier = self
            .repo
            .find_modifier(id)
            .await?
            .ok_or_else(|| RestaurantOperationsError::ModifierNotFound(id.into_uuid()))?;
        modifier.update(
            cmd.name,
            cmd.price_delta.unwrap_or(Decimal::ZERO),
            cmd.sort_order.unwrap_or(0),
            cmd.is_active,
        )?;
        self.repo.update_modifier(&modifier).await?;
        Ok(modifier)
    }
}

pub struct ListGroupsWithModifiersUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl ListGroupsWithModifiersUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<ModifierGroupWithModifiers>, RestaurantOperationsError> {
        self.repo
            .list_groups_with_modifiers(store_id, only_active)
            .await
    }
}

pub struct AssignProductModifierGroupsUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl AssignProductModifierGroupsUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        product_id: Uuid,
        cmd: AssignProductModifierGroupsCommand,
    ) -> Result<(), RestaurantOperationsError> {
        let typed: Vec<MenuModifierGroupId> = cmd
            .group_ids
            .iter()
            .copied()
            .map(MenuModifierGroupId::from_uuid)
            .collect();
        // Verify each group exists before binding (caller mistakes would
        // otherwise produce a confusing FK error).
        for gid in &typed {
            if self.repo.find_group(*gid).await?.is_none() {
                return Err(RestaurantOperationsError::ModifierGroupNotFound(
                    gid.into_uuid(),
                ));
            }
        }
        self.repo.assign_groups_to_product(product_id, &typed).await
    }
}

pub struct GetProductModifierGroupsUseCase {
    repo: Arc<dyn MenuModifierRepository>,
}

impl GetProductModifierGroupsUseCase {
    pub fn new(repo: Arc<dyn MenuModifierRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        product_id: Uuid,
    ) -> Result<Vec<ModifierGroupWithModifiers>, RestaurantOperationsError> {
        self.repo.list_groups_for_product(product_id).await
    }
}
