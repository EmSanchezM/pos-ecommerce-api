use async_trait::async_trait;
use sqlx::PgPool;

use crate::CatalogError;
use crate::domain::entities::ImageStorageProvider;
use crate::domain::repositories::ImageStorageProviderRepository;
use crate::domain::value_objects::{ImageStorageProviderId, StorageProviderType};
use identity::StoreId;

pub struct PgImageStorageProviderRepository {
    pool: PgPool,
}

impl PgImageStorageProviderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str = r#"
    id, store_id, name, provider_type, is_active, is_default,
    api_key_encrypted, secret_key_encrypted, config_json,
    created_at, updated_at
"#;

#[async_trait]
impl ImageStorageProviderRepository for PgImageStorageProviderRepository {
    async fn save(&self, p: &ImageStorageProvider) -> Result<(), CatalogError> {
        sqlx::query(
            r#"INSERT INTO image_storage_providers
              (id, store_id, name, provider_type, is_active, is_default,
               api_key_encrypted, secret_key_encrypted, config_json,
               created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#,
        )
        .bind(p.id().into_uuid())
        .bind(p.store_id().into_uuid())
        .bind(p.name())
        .bind(p.provider_type().to_string())
        .bind(p.is_active())
        .bind(p.is_default())
        .bind(p.api_key_encrypted())
        .bind(p.secret_key_encrypted())
        .bind(p.config_json())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ImageStorageProviderId,
    ) -> Result<Option<ImageStorageProvider>, CatalogError> {
        let sql =
            format!("SELECT {SELECT_COLUMNS} FROM image_storage_providers WHERE id = $1 LIMIT 1");
        let row = sqlx::query_as::<_, ProviderRow>(&sql)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<ImageStorageProvider>, CatalogError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM image_storage_providers
             WHERE store_id = $1 ORDER BY name"
        );
        let rows = sqlx::query_as::<_, ProviderRow>(&sql)
            .bind(store_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_default(
        &self,
        store_id: StoreId,
    ) -> Result<Option<ImageStorageProvider>, CatalogError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM image_storage_providers
             WHERE store_id = $1 AND is_default = true AND is_active = true
             LIMIT 1"
        );
        let row = sqlx::query_as::<_, ProviderRow>(&sql)
            .bind(store_id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, p: &ImageStorageProvider) -> Result<(), CatalogError> {
        let result = sqlx::query(
            r#"UPDATE image_storage_providers SET
                 name=$2, provider_type=$3, is_active=$4, is_default=$5,
                 api_key_encrypted=$6, secret_key_encrypted=$7, config_json=$8,
                 updated_at=$9
               WHERE id=$1"#,
        )
        .bind(p.id().into_uuid())
        .bind(p.name())
        .bind(p.provider_type().to_string())
        .bind(p.is_active())
        .bind(p.is_default())
        .bind(p.api_key_encrypted())
        .bind(p.secret_key_encrypted())
        .bind(p.config_json())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::StorageProviderNotFound(p.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: ImageStorageProviderId) -> Result<(), CatalogError> {
        let result = sqlx::query("DELETE FROM image_storage_providers WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::StorageProviderNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn unset_default_except(
        &self,
        store_id: StoreId,
        keep: ImageStorageProviderId,
    ) -> Result<(), CatalogError> {
        sqlx::query(
            "UPDATE image_storage_providers
             SET is_default = false, updated_at = NOW()
             WHERE store_id = $1 AND id <> $2 AND is_default = true",
        )
        .bind(store_id.into_uuid())
        .bind(keep.into_uuid())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ProviderRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    name: String,
    provider_type: String,
    is_active: bool,
    is_default: bool,
    api_key_encrypted: String,
    secret_key_encrypted: String,
    config_json: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ProviderRow> for ImageStorageProvider {
    type Error = CatalogError;
    fn try_from(row: ProviderRow) -> Result<Self, Self::Error> {
        let provider_type: StorageProviderType = row
            .provider_type
            .parse()
            .map_err(|_| CatalogError::InvalidStorageProviderType)?;
        Ok(ImageStorageProvider::reconstitute(
            ImageStorageProviderId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.name,
            provider_type,
            row.is_active,
            row.is_default,
            row.api_key_encrypted,
            row.secret_key_encrypted,
            row.config_json,
            row.created_at,
            row.updated_at,
        ))
    }
}
