//! Public service order status — `/api/v1/public/service-orders/{id}?token=`.
//! Same shape as `handlers/booking/public.rs::get_public_appointment_handler`.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use service_orders::{GetPublicServiceOrderUseCase, PublicServiceOrderResponse};

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PublicServiceOrderQuery {
    pub token: String,
}

pub async fn get_public_service_order_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<PublicServiceOrderQuery>,
) -> Result<Json<PublicServiceOrderResponse>, Response> {
    let use_case = GetPublicServiceOrderUseCase::new(
        state.service_order_repo(),
        state.service_order_item_repo(),
        state.service_diagnostic_repo(),
        state.service_quote_repo(),
    );
    let response = use_case
        .execute(id, &params.token)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(response))
}
