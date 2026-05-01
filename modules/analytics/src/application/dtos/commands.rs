use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::value_objects::{KpiKey, ReportKind, TimeWindow, WidgetKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardCommand {
    pub store_id: Option<Uuid>,
    pub owner_user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWidgetCommand {
    pub title: String,
    pub kind: WidgetKind,
    pub kpi_key: Option<KpiKey>,
    pub time_window: TimeWindow,
    pub position: i32,
    #[serde(default)]
    pub config: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReportCommand {
    pub kind: ReportKind,
    pub store_id: Option<Uuid>,
    pub time_window: TimeWindow,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}
