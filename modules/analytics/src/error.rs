use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("Dashboard not found: {0}")]
    DashboardNotFound(Uuid),

    #[error("Widget not found: {0}")]
    WidgetNotFound(Uuid),

    #[error("KPI snapshot not found for key '{0}'")]
    SnapshotNotFound(String),

    #[error("Unknown KPI key: {0}")]
    UnknownKpiKey(String),

    #[error("Unknown report kind: {0}")]
    UnknownReportKind(String),

    #[error("Invalid time window: {0}")]
    InvalidTimeWindow(String),

    #[error("Invalid widget kind: {0}")]
    InvalidWidgetKind(String),

    #[error("Invalid widget configuration: {0}")]
    InvalidWidgetConfig(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
