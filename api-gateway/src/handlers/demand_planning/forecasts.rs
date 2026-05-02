//! Forecast read endpoints.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use demand_planning::{DemandForecastResponse, GetForecastUseCase};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct GetForecastQuery {
    pub store_id: Uuid,
}

pub async fn get_forecast_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(variant_id): Path<Uuid>,
    Query(params): Query<GetForecastQuery>,
) -> Result<Json<Vec<DemandForecastResponse>>, Response> {
    require_permission(&ctx, "demand_planning:read_forecast")?;
    let use_case = GetForecastUseCase::new(state.demand_forecast_repo());
    let forecasts = use_case
        .execute(variant_id, params.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        forecasts.iter().map(DemandForecastResponse::from).collect(),
    ))
}
