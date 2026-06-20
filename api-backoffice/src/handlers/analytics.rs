// Cross-org analytics handlers (Phase 6 — Slice D)
//
//   GET /backoffice/analytics/kpis/{kpi_key}?window=this_month — platform:analytics.read
//   GET /backoffice/analytics/overview?window=this_month       — platform:analytics.read
//
// Reads SYSTEM-WIDE (store_id = NULL) KPI snapshots produced by the analytics
// recompute job's canonical pass (revenue, sales count, average ticket, unique
// customers across Today/ThisWeek/ThisMonth). This is the platform-owner view:
// metrics aggregated across every org, not tenant-scoped. Read-only — no audit.

use std::str::FromStr;

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use analytics::{AnalyticsError, GetKpiSnapshotUseCase, KpiKey, KpiSnapshotResponse, TimeWindow};

use crate::error::AppError;
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// Canonical system-wide KPIs maintained by the recompute job — the set the
/// overview bundles.
const OVERVIEW_KPIS: &[&str] = &[
    KpiKey::REVENUE_TOTAL,
    KpiKey::SALES_COUNT,
    KpiKey::AVERAGE_TICKET,
    KpiKey::UNIQUE_CUSTOMERS,
];

/// `?window=` — defaults to `this_month` when omitted.
#[derive(Debug, Deserialize)]
pub struct WindowQuery {
    pub window: Option<String>,
}

#[allow(clippy::result_large_err)]
fn parse_window(q: &WindowQuery) -> Result<TimeWindow, Response> {
    match &q.window {
        Some(w) => TimeWindow::from_str(w).map_err(|e| AppError::from(e).into_response()),
        None => Ok(TimeWindow::ThisMonth),
    }
}

/// One KPI in the overview — `snapshot` is null when the recompute job has not
/// produced that key/window yet.
#[derive(Debug, Serialize)]
pub struct KpiEntry {
    pub kpi_key: String,
    pub snapshot: Option<KpiSnapshotResponse>,
}

#[derive(Debug, Serialize)]
pub struct GlobalAnalyticsOverview {
    pub window: TimeWindow,
    pub kpis: Vec<KpiEntry>,
}

/// GET /backoffice/analytics/kpis/{kpi_key} — a single system-wide KPI snapshot.
pub async fn get_kpi_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(kpi_key): Path<String>,
    Query(query): Query<WindowQuery>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:analytics.read")?;
    let window = parse_window(&query)?;

    let use_case = GetKpiSnapshotUseCase::new(state.kpi_snapshot_repo());
    let snapshot = use_case
        .execute(&KpiKey::new(kpi_key), None, window)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(KpiSnapshotResponse::from(&snapshot)))
}

/// GET /backoffice/analytics/overview — the canonical system-wide KPIs bundled
/// for one time window. KPIs not yet computed come back with `snapshot: null`.
pub async fn overview_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Query(query): Query<WindowQuery>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:analytics.read")?;
    let window = parse_window(&query)?;

    let use_case = GetKpiSnapshotUseCase::new(state.kpi_snapshot_repo());
    let mut kpis = Vec::with_capacity(OVERVIEW_KPIS.len());
    for key in OVERVIEW_KPIS {
        let snapshot = match use_case.execute(&KpiKey::new(*key), None, window).await {
            Ok(s) => Some(KpiSnapshotResponse::from(&s)),
            // A missing snapshot is expected (job hasn't run for it yet) — null.
            Err(AnalyticsError::SnapshotNotFound(_)) => None,
            Err(e) => return Err(AppError::from(e).into_response()),
        };
        kpis.push(KpiEntry {
            kpi_key: (*key).to_string(),
            snapshot,
        });
    }

    Ok(Json(GlobalAnalyticsOverview { window, kpis }))
}
