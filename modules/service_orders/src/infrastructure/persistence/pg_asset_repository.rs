use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::Asset;
use crate::domain::repositories::AssetRepository;
use crate::domain::value_objects::{AssetId, AssetType};

pub struct PgAssetRepository {
    pool: PgPool,
}

impl PgAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for PgAssetRepository {
    async fn save(&self, a: &Asset) -> Result<(), ServiceOrdersError> {
        sqlx::query(
            r#"
            INSERT INTO service_assets (
                id, store_id, customer_id, asset_type,
                brand, model, identifier, year, color, description,
                attributes, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.store_id())
        .bind(a.customer_id())
        .bind(a.asset_type().as_str())
        .bind(a.brand())
        .bind(a.model())
        .bind(a.identifier())
        .bind(a.year())
        .bind(a.color())
        .bind(a.description())
        .bind(a.attributes())
        .bind(a.is_active())
        .bind(a.created_at())
        .bind(a.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, a: &Asset) -> Result<(), ServiceOrdersError> {
        let result = sqlx::query(
            r#"
            UPDATE service_assets
               SET customer_id = $2,
                   brand       = $3,
                   model       = $4,
                   identifier  = $5,
                   year        = $6,
                   color       = $7,
                   description = $8,
                   attributes  = $9,
                   is_active   = $10,
                   updated_at  = $11
             WHERE id = $1
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.customer_id())
        .bind(a.brand())
        .bind(a.model())
        .bind(a.identifier())
        .bind(a.year())
        .bind(a.color())
        .bind(a.description())
        .bind(a.attributes())
        .bind(a.is_active())
        .bind(a.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ServiceOrdersError::AssetNotFound(a.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: AssetId) -> Result<Option<Asset>, ServiceOrdersError> {
        let row = sqlx::query_as::<_, AssetRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Asset::try_from).transpose()
    }

    async fn list_by_store(
        &self,
        store_id: Uuid,
        only_active: bool,
        asset_type_filter: Option<AssetType>,
    ) -> Result<Vec<Asset>, ServiceOrdersError> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, store_id, customer_id, asset_type, brand, model, \
             identifier, year, color, description, attributes, is_active, \
             created_at, updated_at FROM service_assets WHERE store_id = ",
        );
        qb.push_bind(store_id);
        if only_active {
            qb.push(" AND is_active = TRUE");
        }
        if let Some(t) = asset_type_filter {
            qb.push(" AND asset_type = ")
                .push_bind(t.as_str().to_string());
        }
        qb.push(" ORDER BY created_at DESC");
        let rows: Vec<AssetRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(Asset::try_from).collect()
    }

    async fn list_by_customer(&self, customer_id: Uuid) -> Result<Vec<Asset>, ServiceOrdersError> {
        let rows = sqlx::query_as::<_, AssetRow>(
            r#"
            SELECT id, store_id, customer_id, asset_type, brand, model,
                   identifier, year, color, description, attributes, is_active,
                   created_at, updated_at
            FROM service_assets
            WHERE customer_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Asset::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, customer_id, asset_type, brand, model,
       identifier, year, color, description, attributes, is_active,
       created_at, updated_at
FROM service_assets
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct AssetRow {
    id: Uuid,
    store_id: Uuid,
    customer_id: Option<Uuid>,
    asset_type: String,
    brand: Option<String>,
    model: Option<String>,
    identifier: Option<String>,
    year: Option<i32>,
    color: Option<String>,
    description: Option<String>,
    attributes: JsonValue,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<AssetRow> for Asset {
    type Error = ServiceOrdersError;
    fn try_from(r: AssetRow) -> Result<Self, ServiceOrdersError> {
        Ok(Asset::reconstitute(
            AssetId::from_uuid(r.id),
            r.store_id,
            r.customer_id,
            AssetType::from_str(&r.asset_type)?,
            r.brand,
            r.model,
            r.identifier,
            r.year,
            r.color,
            r.description,
            r.attributes,
            r.is_active,
            r.created_at,
            r.updated_at,
        ))
    }
}
