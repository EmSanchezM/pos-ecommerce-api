//! Widget entity — one panel inside a dashboard. Bound to a single `KpiKey`
//! (or a `ReportKind` via the `config` JSON for table widgets); the dashboard
//! overview use case resolves the latest snapshot/report for each widget at
//! read time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::domain::value_objects::{DashboardId, KpiKey, TimeWindow, WidgetId, WidgetKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    id: WidgetId,
    dashboard_id: DashboardId,
    title: String,
    kind: WidgetKind,
    kpi_key: Option<KpiKey>,
    time_window: TimeWindow,
    position: i32,
    config: JsonValue,
    created_at: DateTime<Utc>,
}

impl Widget {
    pub fn create(
        dashboard_id: DashboardId,
        title: impl Into<String>,
        kind: WidgetKind,
        kpi_key: Option<KpiKey>,
        time_window: TimeWindow,
        position: i32,
        config: JsonValue,
    ) -> Self {
        Self {
            id: WidgetId::new(),
            dashboard_id,
            title: title.into(),
            kind,
            kpi_key,
            time_window,
            position,
            config,
            created_at: Utc::now(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: WidgetId,
        dashboard_id: DashboardId,
        title: String,
        kind: WidgetKind,
        kpi_key: Option<KpiKey>,
        time_window: TimeWindow,
        position: i32,
        config: JsonValue,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            dashboard_id,
            title,
            kind,
            kpi_key,
            time_window,
            position,
            config,
            created_at,
        }
    }

    pub fn id(&self) -> WidgetId {
        self.id
    }
    pub fn dashboard_id(&self) -> DashboardId {
        self.dashboard_id
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn kind(&self) -> WidgetKind {
        self.kind
    }
    pub fn kpi_key(&self) -> Option<&KpiKey> {
        self.kpi_key.as_ref()
    }
    pub fn time_window(&self) -> TimeWindow {
        self.time_window
    }
    pub fn position(&self) -> i32 {
        self.position
    }
    pub fn config(&self) -> &JsonValue {
        &self.config
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_widget_with_kpi_card() {
        let dashboard_id = DashboardId::new();
        let w = Widget::create(
            dashboard_id,
            "Today's revenue",
            WidgetKind::KpiCard,
            Some(KpiKey::new(KpiKey::REVENUE_TOTAL)),
            TimeWindow::Today,
            0,
            json!({}),
        );
        assert_eq!(w.dashboard_id(), dashboard_id);
        assert_eq!(w.kind(), WidgetKind::KpiCard);
        assert_eq!(w.position(), 0);
        assert!(w.kpi_key().is_some());
    }
}
