//! Accounting period endpoints.

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use accounting::{
    AccountingPeriodId, AccountingPeriodResponse, ClosePeriodUseCase, ListPeriodsUseCase,
    OpenPeriodCommand, OpenPeriodUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn list_periods_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> Result<Json<Vec<AccountingPeriodResponse>>, Response> {
    require_permission(&ctx, "accounting:read")?;

    let use_case = ListPeriodsUseCase::new(state.accounting_period_repo());
    let periods = use_case
        .execute()
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(
        periods.iter().map(AccountingPeriodResponse::from).collect(),
    ))
}

pub async fn open_period_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<OpenPeriodCommand>,
) -> Result<Json<AccountingPeriodResponse>, Response> {
    require_permission(&ctx, "accounting:write")?;

    let use_case = OpenPeriodUseCase::new(state.accounting_period_repo());
    let period = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(AccountingPeriodResponse::from(&period)))
}

pub async fn close_period_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountingPeriodResponse>, Response> {
    require_permission(&ctx, "accounting:write")?;

    let use_case = ClosePeriodUseCase::new(state.accounting_period_repo());
    let period = use_case
        .execute(AccountingPeriodId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(AccountingPeriodResponse::from(&period)))
}
