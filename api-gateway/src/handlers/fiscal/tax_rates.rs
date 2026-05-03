// Tax rate handlers for the Fiscal module

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use fiscal::{CreateTaxRateCommand, TaxRateResponse, UpdateTaxRateCommand};

pub async fn create_tax_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<CreateTaxRateCommand>,
) -> Result<(StatusCode, Json<TaxRateResponse>), Response> {
    require_permission(&ctx, "tax_rates:create")?;
    verify_store_in_org(state.pool(), &ctx, command.store_id).await?;

    let use_case = fiscal::CreateTaxRateUseCase::new(state.tax_rate_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_tax_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TaxRateResponse>, Response> {
    require_permission(&ctx, "tax_rates:read")?;

    let use_case = fiscal::GetTaxRateUseCase::new(state.tax_rate_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn list_tax_rates_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(store_id): Path<Uuid>,
) -> Result<Json<Vec<TaxRateResponse>>, Response> {
    require_permission(&ctx, "tax_rates:read")?;
    verify_store_in_org(state.pool(), &ctx, store_id).await?;

    let use_case = fiscal::ListTaxRatesUseCase::new(state.tax_rate_repo());

    let response = use_case
        .execute(store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn update_tax_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut command): JsonBody<UpdateTaxRateCommand>,
) -> Result<Json<TaxRateResponse>, Response> {
    require_permission(&ctx, "tax_rates:update")?;

    command.tax_rate_id = id;

    let use_case = fiscal::UpdateTaxRateUseCase::new(state.tax_rate_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn delete_tax_rate_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Response> {
    require_permission(&ctx, "tax_rates:delete")?;

    let use_case = fiscal::DeleteTaxRateUseCase::new(state.tax_rate_repo());

    use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(StatusCode::NO_CONTENT)
}
