use async_trait::async_trait;
use sqlx::PgPool;

use crate::ShippingError;
use crate::domain::entities::DeliveryProvider;
use crate::domain::repositories::DeliveryProviderRepository;
use crate::domain::value_objects::{DeliveryProviderId, DeliveryProviderType};
use identity::StoreId;

pub struct PgDeliveryProviderRepository {
    pool: PgPool,
}

impl PgDeliveryProviderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeliveryProviderRepository for PgDeliveryProviderRepository {
    async fn save(&self, p: &DeliveryProvider) -> Result<(), ShippingError> {
        sqlx::query(
            r#"INSERT INTO delivery_providers
              (id, store_id, name, provider_type, is_active, is_default,
               api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
               coverage_zone_ids, webhook_secret, created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)"#,
        )
        .bind(p.id().into_uuid())
        .bind(p.store_id().into_uuid())
        .bind(p.name())
        .bind(p.provider_type().to_string())
        .bind(p.is_active())
        .bind(p.is_default())
        .bind(p.api_key_encrypted())
        .bind(p.secret_key_encrypted())
        .bind(p.merchant_id())
        .bind(p.is_sandbox())
        .bind(p.coverage_zone_ids())
        .bind(p.webhook_secret())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: DeliveryProviderId,
    ) -> Result<Option<DeliveryProvider>, ShippingError> {
        let row = sqlx::query_as::<_, ProviderRow>(SELECT_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<DeliveryProvider>, ShippingError> {
        let rows = sqlx::query_as::<_, ProviderRow>(
            r#"SELECT id, store_id, name, provider_type, is_active, is_default,
                   api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
                   coverage_zone_ids, webhook_secret, created_at, updated_at
               FROM delivery_providers WHERE store_id = $1 ORDER BY name"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_default(
        &self,
        store_id: StoreId,
    ) -> Result<Option<DeliveryProvider>, ShippingError> {
        let row = sqlx::query_as::<_, ProviderRow>(
            r#"SELECT id, store_id, name, provider_type, is_active, is_default,
                   api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
                   coverage_zone_ids, webhook_secret, created_at, updated_at
               FROM delivery_providers
               WHERE store_id = $1 AND is_default = true AND is_active = true
               LIMIT 1"#,
        )
        .bind(store_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, p: &DeliveryProvider) -> Result<(), ShippingError> {
        let result = sqlx::query(
            r#"UPDATE delivery_providers SET
                 name=$2, provider_type=$3, is_active=$4, is_default=$5,
                 api_key_encrypted=$6, secret_key_encrypted=$7,
                 merchant_id=$8, is_sandbox=$9, coverage_zone_ids=$10,
                 webhook_secret=$11, updated_at=$12
               WHERE id=$1"#,
        )
        .bind(p.id().into_uuid())
        .bind(p.name())
        .bind(p.provider_type().to_string())
        .bind(p.is_active())
        .bind(p.is_default())
        .bind(p.api_key_encrypted())
        .bind(p.secret_key_encrypted())
        .bind(p.merchant_id())
        .bind(p.is_sandbox())
        .bind(p.coverage_zone_ids())
        .bind(p.webhook_secret())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::DeliveryProviderNotFound(p.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: DeliveryProviderId) -> Result<(), ShippingError> {
        let result = sqlx::query("DELETE FROM delivery_providers WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::DeliveryProviderNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn unset_default_except(
        &self,
        store_id: StoreId,
        keep: DeliveryProviderId,
    ) -> Result<(), ShippingError> {
        sqlx::query(
            r#"UPDATE delivery_providers
               SET is_default = false, updated_at = NOW()
               WHERE store_id = $1 AND id <> $2 AND is_default = true"#,
        )
        .bind(store_id.into_uuid())
        .bind(keep.into_uuid())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

const SELECT_SQL: &str = r#"
SELECT id, store_id, name, provider_type, is_active, is_default,
       api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
       coverage_zone_ids, webhook_secret, created_at, updated_at
FROM delivery_providers WHERE id = $1
"#;

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
    merchant_id: Option<String>,
    is_sandbox: bool,
    coverage_zone_ids: Vec<uuid::Uuid>,
    webhook_secret: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ProviderRow> for DeliveryProvider {
    type Error = ShippingError;
    fn try_from(row: ProviderRow) -> Result<Self, Self::Error> {
        let provider_type: DeliveryProviderType = row
            .provider_type
            .parse()
            .map_err(|_| ShippingError::InvalidProviderType)?;
        Ok(DeliveryProvider::reconstitute(
            DeliveryProviderId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.name,
            provider_type,
            row.is_active,
            row.is_default,
            row.api_key_encrypted,
            row.secret_key_encrypted,
            row.merchant_id,
            row.is_sandbox,
            row.coverage_zone_ids,
            row.webhook_secret,
            row.created_at,
            row.updated_at,
        ))
    }
}
