use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::entities::{Dashboard, KpiSnapshot, Widget};
use crate::domain::value_objects::{KpiKey, TimeWindow, WidgetKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    pub id: Uuid,
    pub store_id: Option<Uuid>,
    pub owner_user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub layout: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Dashboard> for DashboardResponse {
    fn from(d: &Dashboard) -> Self {
        Self {
            id: d.id().into_uuid(),
            store_id: d.store_id(),
            owner_user_id: d.owner_user_id(),
            name: d.name().to_string(),
            description: d.description().map(|s| s.to_string()),
            layout: d.layout().clone(),
            created_at: d.created_at(),
            updated_at: d.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetResponse {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub title: String,
    pub kind: WidgetKind,
    pub kpi_key: Option<KpiKey>,
    pub time_window: TimeWindow,
    pub position: i32,
    pub config: JsonValue,
    pub created_at: DateTime<Utc>,
}

impl From<&Widget> for WidgetResponse {
    fn from(w: &Widget) -> Self {
        Self {
            id: w.id().into_uuid(),
            dashboard_id: w.dashboard_id().into_uuid(),
            title: w.title().to_string(),
            kind: w.kind(),
            kpi_key: w.kpi_key().cloned(),
            time_window: w.time_window(),
            position: w.position(),
            config: w.config().clone(),
            created_at: w.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSnapshotResponse {
    pub kpi_key: KpiKey,
    pub store_id: Option<Uuid>,
    pub time_window: TimeWindow,
    pub value: Decimal,
    pub metadata: JsonValue,
    pub computed_at: DateTime<Utc>,
}

impl From<&KpiSnapshot> for KpiSnapshotResponse {
    fn from(s: &KpiSnapshot) -> Self {
        Self {
            kpi_key: s.kpi_key().clone(),
            store_id: s.store_id(),
            time_window: s.time_window(),
            value: s.value(),
            metadata: s.metadata().clone(),
            computed_at: s.computed_at(),
        }
    }
}

/// A widget bundled with its currently resolved snapshot (or `None` if
/// the recompute job has not yet run for that key/window).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetOverviewResponse {
    pub widget: WidgetResponse,
    pub snapshot: Option<KpiSnapshotResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverviewResponse {
    pub dashboard: DashboardResponse,
    pub widgets: Vec<WidgetOverviewResponse>,
}
