mod add_widget;
mod create_dashboard;
mod get_dashboard_overview;
mod get_kpi_snapshot;
mod list_dashboards;
mod recompute_kpi_snapshots;
mod remove_widget;
mod run_report;

pub use add_widget::AddWidgetUseCase;
pub use create_dashboard::CreateDashboardUseCase;
pub use get_dashboard_overview::GetDashboardOverviewUseCase;
pub use get_kpi_snapshot::GetKpiSnapshotUseCase;
pub use list_dashboards::ListDashboardsUseCase;
pub use recompute_kpi_snapshots::RecomputeKpiSnapshotsUseCase;
pub use remove_widget::RemoveWidgetUseCase;
pub use run_report::RunReportUseCase;
