//! Dashboard CRUD + widget endpoints.
//!
//! Read endpoints require `reports:analytics`; write endpoints require
//! `analytics:dashboards:write`. Dashboards are scoped to their owner — the
//! list endpoint always filters by `CurrentUser.user_id()`, so users only
//! see what they created.

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use analytics::{
    AddWidgetCommand, AddWidgetUseCase, CreateDashboardCommand, CreateDashboardUseCase,
    DashboardId, DashboardOverviewResponse, DashboardResponse, GetDashboardOverviewUseCase,
    ListDashboardsUseCase, RemoveWidgetUseCase, WidgetId, WidgetResponse,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

// =============================================================================
// Request DTOs (HTTP layer)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDashboardBody {
    pub store_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
}

// =============================================================================
// GET /api/v1/analytics/dashboards
// =============================================================================

pub async fn list_dashboards_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> Result<Json<Vec<DashboardResponse>>, Response> {
    require_permission(&ctx, "reports:analytics")?;

    let use_case = ListDashboardsUseCase::new(state.dashboard_repo());
    let dashboards = use_case
        .execute(*ctx.user_id().as_uuid())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(
        dashboards.iter().map(DashboardResponse::from).collect(),
    ))
}

// =============================================================================
// POST /api/v1/analytics/dashboards
// =============================================================================

pub async fn create_dashboard_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(body): Json<CreateDashboardBody>,
) -> Result<Json<DashboardResponse>, Response> {
    require_permission(&ctx, "analytics:dashboards:write")?;

    let use_case = CreateDashboardUseCase::new(state.dashboard_repo());
    let dashboard = use_case
        .execute(CreateDashboardCommand {
            store_id: body.store_id,
            owner_user_id: *ctx.user_id().as_uuid(),
            name: body.name,
            description: body.description,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(DashboardResponse::from(&dashboard)))
}

// =============================================================================
// GET /api/v1/analytics/dashboards/{id}/overview
// =============================================================================

pub async fn get_dashboard_overview_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DashboardOverviewResponse>, Response> {
    require_permission(&ctx, "reports:analytics")?;

    let use_case = GetDashboardOverviewUseCase::new(
        state.dashboard_repo(),
        state.widget_repo(),
        state.kpi_snapshot_repo(),
    );
    let response = use_case
        .execute(DashboardId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

// =============================================================================
// POST /api/v1/analytics/dashboards/{id}/widgets
// =============================================================================

pub async fn add_widget_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<AddWidgetCommand>,
) -> Result<Json<WidgetResponse>, Response> {
    require_permission(&ctx, "analytics:dashboards:write")?;

    let use_case = AddWidgetUseCase::new(state.dashboard_repo(), state.widget_repo());
    let widget = use_case
        .execute(DashboardId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(WidgetResponse::from(&widget)))
}

// =============================================================================
// DELETE /api/v1/analytics/widgets/{id}
// =============================================================================

pub async fn remove_widget_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "analytics:dashboards:write")?;

    let use_case = RemoveWidgetUseCase::new(state.widget_repo());
    use_case
        .execute(WidgetId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
