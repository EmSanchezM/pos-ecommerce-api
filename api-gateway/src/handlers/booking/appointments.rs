//! Appointment endpoints — staff/admin side. Public booking lives in
//! `public.rs`.

use std::str::FromStr;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use booking::{
    AppointmentId, AppointmentResponse, AppointmentStatus, CancelAppointmentCommand,
    CancelAppointmentUseCase, CompleteAppointmentCommand, CompleteAppointmentUseCase,
    ConfirmAppointmentUseCase, CreateAppointmentAdminCommand, CreateAppointmentAdminUseCase,
    GetAppointmentUseCase, ListAppointmentsFilters, ListAppointmentsUseCase,
    NoShowAppointmentUseCase, ResourceId, StartAppointmentUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListAppointmentsQuery {
    pub store_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

pub async fn list_appointments_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListAppointmentsQuery>,
) -> Result<Json<Vec<AppointmentResponse>>, Response> {
    require_permission(&ctx, "booking:read_appointment")?;

    let status = match params.status.as_deref() {
        Some(s) => {
            Some(AppointmentStatus::from_str(s).map_err(|e| AppError::from(e).into_response())?)
        }
        None => None,
    };

    let filters = ListAppointmentsFilters {
        store_id: params.store_id,
        resource_id: params.resource_id.map(ResourceId::from_uuid),
        customer_id: params.customer_id,
        status,
        from: params.from,
        to: params.to,
        limit: params.limit,
    };

    let use_case = ListAppointmentsUseCase::new(state.appointment_repo());
    let appointments = use_case
        .execute(filters)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        appointments.iter().map(AppointmentResponse::from).collect(),
    ))
}

pub async fn create_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateAppointmentAdminCommand>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:write_appointment")?;
    let use_case = CreateAppointmentAdminUseCase::new(
        state.booking_service_repo(),
        state.resource_repo(),
        state.appointment_repo(),
    );
    let appt = use_case
        .execute(cmd, *ctx.user_id().as_uuid())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}

pub async fn get_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:read_appointment")?;
    let use_case = GetAppointmentUseCase::new(state.appointment_repo());
    let appt = use_case
        .execute(AppointmentId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}

pub async fn confirm_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:transition_appointment")?;
    let use_case = ConfirmAppointmentUseCase::new(state.appointment_repo());
    let appt = use_case
        .execute(AppointmentId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}

pub async fn start_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:transition_appointment")?;
    let use_case = StartAppointmentUseCase::new(state.appointment_repo());
    let appt = use_case
        .execute(AppointmentId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}

pub async fn complete_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<CompleteAppointmentCommand>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:transition_appointment")?;
    let use_case = CompleteAppointmentUseCase::new(state.appointment_repo());
    let appt = use_case
        .execute(AppointmentId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}

pub async fn cancel_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<CancelAppointmentCommand>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:cancel_appointment")?;
    let use_case =
        CancelAppointmentUseCase::new(state.appointment_repo(), state.booking_policy_repo());
    let appt = use_case
        .execute(AppointmentId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}

pub async fn no_show_appointment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AppointmentResponse>, Response> {
    require_permission(&ctx, "booking:transition_appointment")?;
    let use_case = NoShowAppointmentUseCase::new(state.appointment_repo());
    let appt = use_case
        .execute(AppointmentId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(AppointmentResponse::from(&appt)))
}
