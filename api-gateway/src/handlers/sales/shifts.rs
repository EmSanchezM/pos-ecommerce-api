// Cashier shift handlers for the Sales module

use axum::{extract::{Path, Query, State}, http::StatusCode, Json, response::{IntoResponse, Response}};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use sales::{
    CashMovementCommand, CloseShiftCommand, ListShiftsQuery, OpenShiftCommand,
    ShiftListResponse, ShiftReportResponse, ShiftResponse,
};

pub async fn open_shift_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<OpenShiftCommand>,
) -> Result<(StatusCode, Json<ShiftResponse>), Response> {
    require_permission(&ctx, "sales:manage_shift")?;

    let use_case =
        sales::OpenShiftUseCase::new(state.shift_repo());

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn close_shift_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<CloseShiftCommand>,
) -> Result<Json<ShiftResponse>, Response> {
    require_permission(&ctx, "sales:manage_shift")?;

    let use_case =
        sales::CloseShiftUseCase::new(state.shift_repo());

    let mut cmd = command;
    cmd.shift_id = id;

    let response = use_case
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn get_current_shift_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(terminal_id): Path<Uuid>,
) -> Result<Json<ShiftResponse>, Response> {
    require_permission(&ctx, "sales:read_shift")?;

    let use_case =
        sales::GetCurrentShiftUseCase::new(state.shift_repo());

    let response = use_case
        .execute(terminal_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn get_shift_report_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ShiftReportResponse>, Response> {
    require_permission(&ctx, "sales:read_shift")?;

    let use_case =
        sales::GetShiftReportUseCase::new(state.shift_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn list_shifts_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListShiftsQuery>,
) -> Result<Json<ShiftListResponse>, Response> {
    require_permission(&ctx, "sales:read_shift")?;

    let use_case =
        sales::ListShiftsUseCase::new(state.shift_repo());

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn cash_in_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<CashMovementCommand>,
) -> Result<Json<ShiftResponse>, Response> {
    require_permission(&ctx, "sales:manage_shift")?;

    let use_case =
        sales::RecordCashMovementUseCase::new(state.shift_repo());

    let mut cmd = command;
    cmd.shift_id = id;

    let response = use_case
        .cash_in(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn cash_out_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<CashMovementCommand>,
) -> Result<Json<ShiftResponse>, Response> {
    require_permission(&ctx, "sales:manage_shift")?;

    let use_case =
        sales::RecordCashMovementUseCase::new(state.shift_repo());

    let mut cmd = command;
    cmd.shift_id = id;

    let response = use_case
        .cash_out(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
