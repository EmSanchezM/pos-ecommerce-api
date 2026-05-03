//! POST /api/v1/analytics/reports/run — execute a registered report.

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};

use analytics::{ReportRows, RunReportCommand, RunReportUseCase};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn run_report_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<RunReportCommand>,
) -> Result<Json<ReportRows>, Response> {
    require_permission(&ctx, "reports:analytics")?;
    if let Some(sid) = cmd.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }

    let use_case = RunReportUseCase::new(state.analytics_query_repo());
    let rows = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(rows))
}
