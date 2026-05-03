use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::application::dtos::{RegisterAssetCommand, UpdateAssetCommand};
use crate::domain::entities::Asset;
use crate::domain::repositories::AssetRepository;
use crate::domain::value_objects::{AssetId, AssetType};

pub struct RegisterAssetUseCase {
    assets: Arc<dyn AssetRepository>,
}

impl RegisterAssetUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>) -> Self {
        Self { assets }
    }

    pub async fn execute(&self, cmd: RegisterAssetCommand) -> Result<Asset, ServiceOrdersError> {
        let asset = Asset::register(
            cmd.store_id,
            cmd.customer_id,
            cmd.asset_type,
            cmd.brand,
            cmd.model,
            cmd.identifier,
            cmd.year,
            cmd.color,
            cmd.description,
            cmd.attributes.unwrap_or_else(|| json!({})),
        )?;
        self.assets.save(&asset).await?;
        Ok(asset)
    }
}

pub struct UpdateAssetUseCase {
    assets: Arc<dyn AssetRepository>,
}

impl UpdateAssetUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>) -> Self {
        Self { assets }
    }

    pub async fn execute(
        &self,
        id: AssetId,
        cmd: UpdateAssetCommand,
    ) -> Result<Asset, ServiceOrdersError> {
        let mut asset = self
            .assets
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::AssetNotFound(id.into_uuid()))?;
        asset.update_details(
            cmd.brand,
            cmd.model,
            cmd.identifier,
            cmd.year,
            cmd.color,
            cmd.description,
            cmd.attributes.unwrap_or_else(|| json!({})),
        );
        self.assets.update(&asset).await?;
        Ok(asset)
    }
}

pub struct DeactivateAssetUseCase {
    assets: Arc<dyn AssetRepository>,
}

impl DeactivateAssetUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>) -> Self {
        Self { assets }
    }

    pub async fn execute(&self, id: AssetId) -> Result<(), ServiceOrdersError> {
        let mut asset = self
            .assets
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::AssetNotFound(id.into_uuid()))?;
        asset.deactivate();
        self.assets.update(&asset).await?;
        Ok(())
    }
}

pub struct GetAssetUseCase {
    assets: Arc<dyn AssetRepository>,
}

impl GetAssetUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>) -> Self {
        Self { assets }
    }

    pub async fn execute(&self, id: AssetId) -> Result<Asset, ServiceOrdersError> {
        self.assets
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceOrdersError::AssetNotFound(id.into_uuid()))
    }
}

pub struct ListAssetsUseCase {
    assets: Arc<dyn AssetRepository>,
}

impl ListAssetsUseCase {
    pub fn new(assets: Arc<dyn AssetRepository>) -> Self {
        Self { assets }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        only_active: bool,
        asset_type_filter: Option<AssetType>,
    ) -> Result<Vec<Asset>, ServiceOrdersError> {
        self.assets
            .list_by_store(store_id, only_active, asset_type_filter)
            .await
    }
}
