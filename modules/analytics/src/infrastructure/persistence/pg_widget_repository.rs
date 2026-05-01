use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AnalyticsError;
use crate::domain::entities::Widget;
use crate::domain::repositories::WidgetRepository;
use crate::domain::value_objects::{DashboardId, KpiKey, TimeWindow, WidgetId, WidgetKind};

pub struct PgWidgetRepository {
    pool: PgPool,
}

impl PgWidgetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WidgetRepository for PgWidgetRepository {
    async fn save(&self, w: &Widget) -> Result<(), AnalyticsError> {
        sqlx::query(
            r#"
            INSERT INTO dashboard_widgets (
                id, dashboard_id, title, kind, kpi_key, time_window,
                position, config, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(w.id().into_uuid())
        .bind(w.dashboard_id().into_uuid())
        .bind(w.title())
        .bind(w.kind().to_string())
        .bind(w.kpi_key().map(|k| k.as_str().to_string()))
        .bind(w.time_window().to_string())
        .bind(w.position())
        .bind(w.config())
        .bind(w.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: WidgetId) -> Result<Option<Widget>, AnalyticsError> {
        let row = sqlx::query_as::<_, WidgetRow>(SELECT_WIDGET)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Widget::try_from).transpose()
    }

    async fn list_by_dashboard(
        &self,
        dashboard_id: DashboardId,
    ) -> Result<Vec<Widget>, AnalyticsError> {
        let rows = sqlx::query_as::<_, WidgetRow>(
            r#"
            SELECT id, dashboard_id, title, kind, kpi_key, time_window,
                   position, config, created_at
            FROM dashboard_widgets
            WHERE dashboard_id = $1
            ORDER BY position ASC, created_at ASC
            "#,
        )
        .bind(dashboard_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Widget::try_from).collect()
    }

    async fn delete(&self, id: WidgetId) -> Result<(), AnalyticsError> {
        let result = sqlx::query("DELETE FROM dashboard_widgets WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AnalyticsError::WidgetNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_WIDGET: &str = r#"
SELECT id, dashboard_id, title, kind, kpi_key, time_window,
       position, config, created_at
FROM dashboard_widgets
WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct WidgetRow {
    id: Uuid,
    dashboard_id: Uuid,
    title: String,
    kind: String,
    kpi_key: Option<String>,
    time_window: String,
    position: i32,
    config: serde_json::Value,
    created_at: DateTime<Utc>,
}

impl TryFrom<WidgetRow> for Widget {
    type Error = AnalyticsError;

    fn try_from(row: WidgetRow) -> Result<Self, Self::Error> {
        let kind: WidgetKind = row.kind.parse()?;
        let window: TimeWindow = row.time_window.parse()?;
        Ok(Widget::reconstitute(
            WidgetId::from_uuid(row.id),
            DashboardId::from_uuid(row.dashboard_id),
            row.title,
            kind,
            row.kpi_key.map(KpiKey::new),
            window,
            row.position,
            row.config,
            row.created_at,
        ))
    }
}
