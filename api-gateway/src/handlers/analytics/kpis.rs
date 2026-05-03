//! GET /api/v1/analytics/kpis/{kpi_key} — current snapshot for a KPI.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use analytics::{GetKpiSnapshotUseCase, KpiKey, KpiSnapshotResponse, TimeWindow};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct KpiQuery {
    pub store_id: Option<Uuid>,
    #[serde(default = "default_window")]
    pub time_window: TimeWindow,
}

fn default_window() -> TimeWindow {
    TimeWindow::Today
}

pub async fn get_kpi_snapshot_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(kpi_key): Path<String>,
    Query(params): Query<KpiQuery>,
) -> Result<Json<KpiSnapshotResponse>, Response> {
    require_permission(&ctx, "reports:analytics")?;
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }

    let use_case = GetKpiSnapshotUseCase::new(state.kpi_snapshot_repo());
    let snapshot = use_case
        .execute(&KpiKey::new(kpi_key), params.store_id, params.time_window)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(KpiSnapshotResponse::from(&snapshot)))
}
