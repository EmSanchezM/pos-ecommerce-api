//! # Analytics Module
//!
//! Cross-module BI: KPI snapshots, dashboards/widgets, and registered reports.
//!
//! - **Domain**: `KpiSnapshot`, `Dashboard`, `Widget`, value objects (`KpiKey`,
//!   `TimeWindow`, `WidgetKind`, `ReportKind`), repository traits.
//! - **Application**: use cases for dashboard CRUD, report execution, KPI
//!   recompute, and the `AnalyticsEventSubscriber` that hooks into the events
//!   module so the recompute job can react to upstream changes.
//! - **Infrastructure**: `Pg*Repository` implementations.
//!
//! See `docs/roadmap-modulos.md` (Fase 1.1) for the broader plan.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::AnalyticsError;

// Domain
pub use domain::entities::{
    CashierPerformanceRow, Dashboard, DeadStockRow, KpiSnapshot, PeakHourRow,
    ProductProfitabilityRow, ReportRows, Widget,
};
pub use domain::repositories::{
    AnalyticsQueryRepository, DashboardRepository, KpiSnapshotRepository, ReportFilters,
    WidgetRepository,
};
pub use domain::value_objects::{
    DashboardId, KpiKey, KpiSnapshotId, ReportKind, TimeWindow, WidgetId, WidgetKind,
};

// Application
pub use application::dtos::{
    AddWidgetCommand, CreateDashboardCommand, DashboardOverviewResponse, DashboardResponse,
    KpiSnapshotResponse, RunReportCommand, WidgetOverviewResponse, WidgetResponse,
};
pub use application::subscriber::AnalyticsEventSubscriber;
pub use application::use_cases::{
    AddWidgetUseCase, CreateDashboardUseCase, GetDashboardOverviewUseCase, GetKpiSnapshotUseCase,
    ListDashboardsUseCase, RecomputeKpiSnapshotsUseCase, RemoveWidgetUseCase, RunReportUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgAnalyticsQueryRepository, PgDashboardRepository, PgKpiSnapshotRepository, PgWidgetRepository,
};
