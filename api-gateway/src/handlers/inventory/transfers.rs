// Transfer Handlers
//
// REST endpoints for inter-store stock transfers:
// - POST /api/v1/transfers - Create a transfer
// - GET /api/v1/transfers - List transfers
// - GET /api/v1/transfers/{id} - Get transfer details
// - PUT /api/v1/transfers/{id}/submit - Submit for processing
// - PUT /api/v1/transfers/{id}/ship - Ship the transfer
// - PUT /api/v1/transfers/{id}/receive - Receive the transfer
// - PUT /api/v1/transfers/{id}/cancel - Cancel the transfer

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use inventory::{
    CancelTransferUseCase, CreateTransferCommand, CreateTransferUseCase, GetTransferUseCase,
    ListResponse, ListTransfersQuery, ListTransfersUseCase, ReceiveTransferCommand,
    ReceiveTransferUseCase, ShipTransferCommand, ShipTransferUseCase, SubmitTransferUseCase,
    TransferDetailResponse, TransferResponse,
};

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;

/// Query parameters for listing transfers
#[derive(Debug, Deserialize)]
pub struct ListTransfersQueryParams {
    /// Filter by store ID
    pub store_id: Option<Uuid>,
    /// Direction: "outgoing", "incoming", or "all" (default)
    pub direction: Option<String>,
    /// Filter by status
    pub status: Option<String>,
}

impl From<ListTransfersQueryParams> for ListTransfersQuery {
    fn from(params: ListTransfersQueryParams) -> Self {
        ListTransfersQuery {
            store_id: params.store_id,
            direction: params.direction,
            status: params.status,
        }
    }
}

/// Handler for POST /api/v1/transfers
pub async fn create_transfer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<CreateTransferCommand>,
) -> Result<(StatusCode, Json<TransferDetailResponse>), Response> {
    require_permission(&ctx, "transfers:create")?;

    let use_case = CreateTransferUseCase::new(state.transfer_repo());

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for GET /api/v1/transfers
pub async fn list_transfers_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListTransfersQueryParams>,
) -> Result<Json<ListResponse<TransferResponse>>, Response> {
    require_permission(&ctx, "transfers:read")?;

    let use_case = ListTransfersUseCase::new(state.transfer_repo());

    let query: ListTransfersQuery = params.into();
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(ListResponse::new(response)))
}

/// Handler for GET /api/v1/transfers/{id}
pub async fn get_transfer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TransferDetailResponse>, Response> {
    require_permission(&ctx, "transfers:read")?;

    let use_case = GetTransferUseCase::new(state.transfer_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/transfers/{id}/submit
pub async fn submit_transfer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TransferDetailResponse>, Response> {
    require_permission(&ctx, "transfers:create")?;

    let use_case = SubmitTransferUseCase::new(state.transfer_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/transfers/{id}/ship
pub async fn ship_transfer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(mut command): Json<ShipTransferCommand>,
) -> Result<Json<TransferDetailResponse>, Response> {
    require_permission(&ctx, "transfers:ship")?;

    command.transfer_id = id;

    let use_case = ShipTransferUseCase::new(
        state.transfer_repo(),
        state.stock_repo(),
        state.movement_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/transfers/{id}/receive
pub async fn receive_transfer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(mut command): Json<ReceiveTransferCommand>,
) -> Result<Json<TransferDetailResponse>, Response> {
    require_permission(&ctx, "transfers:receive")?;

    command.transfer_id = id;

    let use_case = ReceiveTransferUseCase::new(
        state.transfer_repo(),
        state.stock_repo(),
        state.movement_repo(),
    );

    let actor_id = *ctx.user_id();
    let response = use_case
        .execute(command, actor_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/transfers/{id}/cancel
pub async fn cancel_transfer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TransferDetailResponse>, Response> {
    require_permission(&ctx, "transfers:create")?;

    let use_case = CancelTransferUseCase::new(state.transfer_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
