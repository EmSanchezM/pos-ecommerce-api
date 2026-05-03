//! Per-store booking policy (one row per store).

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use booking::{
    BookingPolicyResponse, GetBookingPolicyUseCase, UpsertBookingPolicyCommand,
    UpsertBookingPolicyUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PolicyQuery {
    pub store_id: Uuid,
}

pub async fn get_booking_policy_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<PolicyQuery>,
) -> Result<Json<BookingPolicyResponse>, Response> {
    require_permission(&ctx, "booking:read_policy")?;
    let use_case = GetBookingPolicyUseCase::new(state.booking_policy_repo());
    let policy = use_case
        .execute(params.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BookingPolicyResponse::from(&policy)))
}

pub async fn upsert_booking_policy_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<UpsertBookingPolicyCommand>,
) -> Result<Json<BookingPolicyResponse>, Response> {
    require_permission(&ctx, "booking:write_policy")?;
    let use_case = UpsertBookingPolicyUseCase::new(state.booking_policy_repo());
    let policy = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BookingPolicyResponse::from(&policy)))
}
