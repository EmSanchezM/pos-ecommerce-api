// Shipment handlers — the heart of the module.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use shipping::{
    AssignDriverCommand, AssignDriverUseCase, CancelShipmentCommand, CancelShipmentUseCase,
    ConfirmPickupCommand, ConfirmPickupUseCase, CreateShipmentCommand, CreateShipmentUseCase,
    DispatchProviderCommand, DispatchProviderUseCase, GetShipmentUseCase, ListShipmentsQuery,
    ListShipmentsUseCase, MarkDeliveredCommand, MarkDeliveredUseCase, MarkFailedCommand,
    MarkFailedUseCase, MarkReadyForPickupUseCase, RescheduleShipmentCommand,
    RescheduleShipmentUseCase, ShipmentListResponse, ShipmentResponse, UpdateShipmentStatusCommand,
    UpdateShipmentStatusUseCase, UpdateTrackingCommand, UpdateTrackingUseCase,
};

pub async fn create_shipment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(cmd): JsonBody<CreateShipmentCommand>,
) -> Result<(StatusCode, Json<ShipmentResponse>), Response> {
    require_permission(&ctx, "shipments:create")?;
    let uc = CreateShipmentUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_shipments_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<ListShipmentsQuery>,
) -> Result<Json<ShipmentListResponse>, Response> {
    require_permission(&ctx, "shipments:read")?;
    let uc = ListShipmentsUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(q)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn get_shipment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:read")?;
    let uc = GetShipmentUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn mark_ready_for_pickup_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:update")?;
    let uc = MarkReadyForPickupUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn confirm_pickup_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<ConfirmPickupCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:update")?;
    cmd.shipment_id = id;
    let uc = ConfirmPickupUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn assign_driver_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<AssignDriverCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:assign")?;
    cmd.shipment_id = id;
    let uc = AssignDriverUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn reschedule_shipment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<RescheduleShipmentCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:assign")?;
    cmd.shipment_id = id;
    let uc = RescheduleShipmentUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn dispatch_provider_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<DispatchProviderCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:assign")?;
    cmd.shipment_id = id;
    let uc = DispatchProviderUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_shipment_status_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateShipmentStatusCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:update")?;
    cmd.shipment_id = id;
    let uc = UpdateShipmentStatusUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn mark_delivered_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<MarkDeliveredCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:update")?;
    cmd.shipment_id = id;
    let uc = MarkDeliveredUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn mark_failed_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<MarkFailedCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:update")?;
    cmd.shipment_id = id;
    let uc = MarkFailedUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn cancel_shipment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<CancelShipmentCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:cancel")?;
    cmd.shipment_id = id;
    let uc = CancelShipmentUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}

pub async fn update_tracking_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut cmd): JsonBody<UpdateTrackingCommand>,
) -> Result<Json<ShipmentResponse>, Response> {
    require_permission(&ctx, "shipments:update")?;
    cmd.shipment_id = id;
    let uc = UpdateTrackingUseCase::new(state.shipment_deps());
    let resp = uc
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(resp))
}
