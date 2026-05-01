use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::Dashboard;
use crate::domain::repositories::DashboardRepository;
use crate::domain::value_objects::DashboardId;

pub struct PgDashboardRepository {
    pool: PgPool,
}

impl PgDashboardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DashboardRepository for PgDashboardRepository {
    async fn save(&self, d: &Dashboard) -> Result<(), AnalyticsError> {
        sqlx::query(
            r#"
            INSERT INTO dashboards (
                id, store_id, owner_user_id, name, description, layout,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.store_id())
        .bind(d.owner_user_id())
        .bind(d.name())
        .bind(d.description())
        .bind(d.layout())
        .bind(d.created_at())
        .bind(d.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, d: &Dashboard) -> Result<(), AnalyticsError> {
        let result = sqlx::query(
            r#"
            UPDATE dashboards
            SET store_id = $2,
                name = $3,
                description = $4,
                layout = $5,
                updated_at = $6
            WHERE id = $1
            "#,
        )
        .bind(d.id().into_uuid())
        .bind(d.store_id())
        .bind(d.name())
        .bind(d.description())
        .bind(d.layout())
        .bind(d.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AnalyticsError::DashboardNotFound(d.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(&self, id: DashboardId) -> Result<Option<Dashboard>, AnalyticsError> {
        let row = sqlx::query_as::<_, DashboardRow>(SELECT_DASHBOARD)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Dashboard::from))
    }

    async fn list_for_owner(&self, owner_user_id: Uuid) -> Result<Vec<Dashboard>, AnalyticsError> {
        let rows = sqlx::query_as::<_, DashboardRow>(
            r#"
            SELECT id, store_id, owner_user_id, name, description, layout,
                   created_at, updated_at
            FROM dashboards
            WHERE owner_user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Dashboard::from).collect())
    }

    async fn delete(&self, id: DashboardId) -> Result<(), AnalyticsError> {
        let result = sqlx::query("DELETE FROM dashboards WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AnalyticsError::DashboardNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_DASHBOARD: &str = r#"
SELECT id, store_id, owner_user_id, name, description, layout,
       created_at, updated_at
FROM dashboards
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct DashboardRow {
    id: Uuid,
    store_id: Option<Uuid>,
    owner_user_id: Uuid,
    name: String,
    description: Option<String>,
    layout: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<DashboardRow> for Dashboard {
    fn from(row: DashboardRow) -> Self {
        Dashboard::reconstitute(
            DashboardId::from_uuid(row.id),
            row.store_id,
            row.owner_user_id,
            row.name,
            row.description,
            row.layout,
            row.created_at,
            row.updated_at,
        )
    }
}
