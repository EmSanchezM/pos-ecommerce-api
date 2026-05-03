use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use service_orders::{
    ApproveQuoteUseCase, CreateQuoteCommand, CreateQuoteUseCase, DecideQuoteCommand, QuoteId,
    QuoteResponse, RejectQuoteUseCase, SendQuoteUseCase, ServiceOrderId,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

pub async fn create_quote_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(order_id): Path<Uuid>,
    Json(cmd): Json<CreateQuoteCommand>,
) -> Result<Json<QuoteResponse>, Response> {
    require_permission(&ctx, "service_orders:write_quote")?;
    let use_case = CreateQuoteUseCase::new(
        state.service_order_repo(),
        state.service_order_item_repo(),
        state.service_quote_repo(),
    );
    let quote = use_case
        .execute(ServiceOrderId::from_uuid(order_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(QuoteResponse::from(&quote)))
}

pub async fn send_quote_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_order_id, quote_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<QuoteResponse>, Response> {
    require_permission(&ctx, "service_orders:transition_quote")?;
    let use_case = SendQuoteUseCase::new(state.service_order_repo(), state.service_quote_repo());
    let quote = use_case
        .execute(QuoteId::from_uuid(quote_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(QuoteResponse::from(&quote)))
}

pub async fn approve_quote_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_order_id, quote_id)): Path<(Uuid, Uuid)>,
    cmd: Option<Json<DecideQuoteCommand>>,
) -> Result<Json<QuoteResponse>, Response> {
    require_permission(&ctx, "service_orders:transition_quote")?;
    let cmd = cmd.map(|j| j.0).unwrap_or_default();
    let use_case = ApproveQuoteUseCase::new(state.service_order_repo(), state.service_quote_repo());
    let quote = use_case
        .execute(QuoteId::from_uuid(quote_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(QuoteResponse::from(&quote)))
}

pub async fn reject_quote_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_order_id, quote_id)): Path<(Uuid, Uuid)>,
    cmd: Option<Json<DecideQuoteCommand>>,
) -> Result<Json<QuoteResponse>, Response> {
    require_permission(&ctx, "service_orders:transition_quote")?;
    let cmd = cmd.map(|j| j.0).unwrap_or_default();
    let use_case = RejectQuoteUseCase::new(state.service_order_repo(), state.service_quote_repo());
    let quote = use_case
        .execute(QuoteId::from_uuid(quote_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(QuoteResponse::from(&quote)))
}
