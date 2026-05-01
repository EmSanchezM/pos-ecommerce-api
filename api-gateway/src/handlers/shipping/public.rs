// Public tracking — NO auth, looked up by tracking_number.

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::error::AppError;
use crate::state::AppState;
use shipping::{PublicTrackingResponse, PublicTrackingUseCase};

pub async fn public_tracking_handler(
    State(state): State<AppState>,
    Path(tracking_number): Path<String>,
) -> Result<Json<PublicTrackingResponse>, Response> {
    let uc = PublicTrackingUseCase::new(state.shipment_repo(), state.shipment_event_repo());
    let resp = uc
        .execute(&tracking_number)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
