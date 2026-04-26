use async_trait::async_trait;
use sqlx::PgPool;

use crate::ShippingError;
use crate::domain::entities::ShippingMethod;
use crate::domain::repositories::ShippingMethodRepository;
use crate::domain::value_objects::{ShippingMethodId, ShippingMethodType};
use identity::StoreId;

pub struct PgShippingMethodRepository {
    pool: PgPool,
}

impl PgShippingMethodRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShippingMethodRepository for PgShippingMethodRepository {
    async fn save(&self, m: &ShippingMethod) -> Result<(), ShippingError> {
        sqlx::query(
            r#"
            INSERT INTO shipping_methods
              (id, store_id, name, code, method_type, description,
               estimated_days_min, estimated_days_max, is_active, sort_order,
               created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            "#,
        )
        .bind(m.id().into_uuid())
        .bind(m.store_id().into_uuid())
        .bind(m.name())
        .bind(m.code())
        .bind(m.method_type().to_string())
        .bind(m.description())
        .bind(m.estimated_days_min())
        .bind(m.estimated_days_max())
        .bind(m.is_active())
        .bind(m.sort_order())
        .bind(m.created_at())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ShippingMethodId,
    ) -> Result<Option<ShippingMethod>, ShippingError> {
        let row = sqlx::query_as::<_, MethodRow>(SELECT_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<ShippingMethod>, ShippingError> {
        let rows = sqlx::query_as::<_, MethodRow>(
            r#"SELECT id, store_id, name, code, method_type, description,
                   estimated_days_min, estimated_days_max, is_active, sort_order,
                   created_at, updated_at
               FROM shipping_methods
               WHERE store_id = $1 ORDER BY sort_order ASC, name ASC"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_active_by_store(
        &self,
        store_id: StoreId,
    ) -> Result<Vec<ShippingMethod>, ShippingError> {
        let rows = sqlx::query_as::<_, MethodRow>(
            r#"SELECT id, store_id, name, code, method_type, description,
                   estimated_days_min, estimated_days_max, is_active, sort_order,
                   created_at, updated_at
               FROM shipping_methods
               WHERE store_id = $1 AND is_active = true
               ORDER BY sort_order ASC, name ASC"#,
        )
        .bind(store_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, m: &ShippingMethod) -> Result<(), ShippingError> {
        let result = sqlx::query(
            r#"UPDATE shipping_methods
               SET name=$2, description=$3,
                   estimated_days_min=$4, estimated_days_max=$5,
                   is_active=$6, sort_order=$7, updated_at=$8
               WHERE id=$1"#,
        )
        .bind(m.id().into_uuid())
        .bind(m.name())
        .bind(m.description())
        .bind(m.estimated_days_min())
        .bind(m.estimated_days_max())
        .bind(m.is_active())
        .bind(m.sort_order())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShippingMethodNotFound(m.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: ShippingMethodId) -> Result<(), ShippingError> {
        let result = sqlx::query("DELETE FROM shipping_methods WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShippingMethodNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_SQL: &str = r#"
SELECT id, store_id, name, code, method_type, description,
       estimated_days_min, estimated_days_max, is_active, sort_order,
       created_at, updated_at
FROM shipping_methods WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct MethodRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    name: String,
    code: String,
    method_type: String,
    description: Option<String>,
    estimated_days_min: Option<i32>,
    estimated_days_max: Option<i32>,
    is_active: bool,
    sort_order: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<MethodRow> for ShippingMethod {
    type Error = ShippingError;
    fn try_from(row: MethodRow) -> Result<Self, Self::Error> {
        let method_type: ShippingMethodType = row
            .method_type
            .parse()
            .map_err(|_| ShippingError::InvalidMethodType)?;
        Ok(ShippingMethod::reconstitute(
            ShippingMethodId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.name,
            row.code,
            method_type,
            row.description,
            row.estimated_days_min,
            row.estimated_days_max,
            row.is_active,
            row.sort_order,
            row.created_at,
            row.updated_at,
        ))
    }
}
