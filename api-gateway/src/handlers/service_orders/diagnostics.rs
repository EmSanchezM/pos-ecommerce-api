use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use service_orders::{
    AddDiagnosticCommand, AddDiagnosticUseCase, DiagnosticResponse, ServiceOrderId,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn add_diagnostic_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(order_id): Path<Uuid>,
    Json(cmd): Json<AddDiagnosticCommand>,
) -> Result<Json<DiagnosticResponse>, Response> {
    require_permission(&ctx, "service_orders:write_diagnostic")?;
    let use_case =
        AddDiagnosticUseCase::new(state.service_order_repo(), state.service_diagnostic_repo());
    let diagnostic = use_case
        .execute(
            ServiceOrderId::from_uuid(order_id),
            Some(*ctx.user_id().as_uuid()),
            cmd,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(DiagnosticResponse::from(&diagnostic)))
}
