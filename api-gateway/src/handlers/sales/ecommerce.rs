// E-Commerce Order Handlers
//
// REST endpoints for e-commerce order workflow transitions:
// - PUT /api/v1/orders/{id}/mark-paid - Mark order as paid
// - PUT /api/v1/orders/{id}/process - Start processing order
// - PUT /api/v1/orders/{id}/ship - Ship order
// - PUT /api/v1/orders/{id}/deliver - Mark order delivered
// - PUT /api/v1/orders/{id}/cancel - Cancel order

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use sales::{
    CancelOrderUseCase, DeliverOrderUseCase, MarkOrderPaidUseCase, ProcessOrderUseCase,
    SaleDetailResponse, ShipOrderUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

/// Handler for PUT /api/v1/orders/{id}/mark-paid
pub async fn mark_order_paid_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "orders:mark_paid")?;

    let use_case = MarkOrderPaidUseCase::new(state.sale_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/orders/{id}/process
pub async fn process_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "orders:process")?;

    let use_case = ProcessOrderUseCase::new(state.sale_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/orders/{id}/ship
pub async fn ship_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "orders:ship")?;

    let use_case = ShipOrderUseCase::new(state.sale_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/orders/{id}/deliver
pub async fn deliver_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "orders:deliver")?;

    let use_case = DeliverOrderUseCase::new(state.sale_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Handler for PUT /api/v1/orders/{id}/cancel
pub async fn cancel_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "orders:cancel")?;

    let use_case = CancelOrderUseCase::new(state.sale_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
