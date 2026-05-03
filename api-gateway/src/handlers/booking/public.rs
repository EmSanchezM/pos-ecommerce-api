//! Public, unauthenticated booking endpoints. Mounted under
//! `/api/v1/public/booking` with **no** auth middleware (same pattern as
//! `public_tracking_handler` in `handlers/shipping/public.rs`).

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use booking::{
    AppointmentResponse, BookAppointmentPublicUseCase, CheckAvailabilityQuery,
    CheckAvailabilityUseCase, GetPublicAppointmentUseCase, ListPublicServicesUseCase,
    PublicBookCommand, PublicBookingResponse, ResourceAvailabilityResponse, ServiceResponse,
};

use crate::error::AppError;
use crate::state::AppState;

pub async fn list_public_services_handler(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
) -> Result<Json<Vec<ServiceResponse>>, Response> {
    let use_case = ListPublicServicesUseCase::new(state.booking_service_repo());
    let services = use_case
        .execute(store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(services))
}

#[derive(Debug, Deserialize)]
pub struct PublicAvailabilityQuery {
    pub service_id: Uuid,
    pub date: NaiveDate,
    pub resource_id: Option<Uuid>,
}

pub async fn public_availability_handler(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
    Query(params): Query<PublicAvailabilityQuery>,
) -> Result<Json<Vec<ResourceAvailabilityResponse>>, Response> {
    let use_case = CheckAvailabilityUseCase::new(
        state.booking_service_repo(),
        state.resource_repo(),
        state.resource_calendar_repo(),
        state.appointment_repo(),
    );
    let availability = use_case
        .execute(
            store_id,
            CheckAvailabilityQuery {
                service_id: params.service_id,
                date: params.date,
                resource_id: params.resource_id,
            },
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(availability))
}

pub async fn public_book_handler(
    State(state): State<AppState>,
    Path(store_id): Path<Uuid>,
    Json(cmd): Json<PublicBookCommand>,
) -> Result<Json<PublicBookingResponse>, Response> {
    let use_case = BookAppointmentPublicUseCase::new(
        state.booking_service_repo(),
        state.resource_repo(),
        state.appointment_repo(),
    );
    let response = use_case
        .execute(store_id, cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct PublicAppointmentQuery {
    pub token: String,
}

pub async fn get_public_appointment_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<PublicAppointmentQuery>,
) -> Result<Json<AppointmentResponse>, Response> {
    let use_case = GetPublicAppointmentUseCase::new(state.appointment_repo());
    let appt = use_case
        .execute(id, &params.token)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(appt))
}
