// Payment transaction handlers

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use payments::{
    ConfirmTransactionCommand, ConfirmTransactionUseCase, GetTransactionUseCase,
    ListTransactionsQuery, ListTransactionsUseCase, ProcessOnlinePaymentCommand,
    ProcessOnlinePaymentUseCase, ProcessRefundCommand, ProcessRefundUseCase,
    ReconcileManualPaymentsUseCase, ReconcilePaymentsCommand, ReconciliationResponse,
    RejectTransactionCommand, RejectTransactionUseCase, TransactionListResponse,
    TransactionResponse,
};

pub async fn process_payment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<ProcessOnlinePaymentCommand>,
) -> Result<(StatusCode, Json<TransactionResponse>), Response> {
    require_permission(&ctx, "transactions:create")?;
    verify_store_in_org(state.pool(), &ctx, command.store_id).await?;

    let use_case = ProcessOnlinePaymentUseCase::new(
        state.payment_gateway_repo(),
        state.transaction_repo(),
        state.gateway_registry(),
    );

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn process_refund_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(transaction_id): Path<Uuid>,
    JsonBody(mut command): JsonBody<ProcessRefundCommand>,
) -> Result<(StatusCode, Json<TransactionResponse>), Response> {
    require_permission(&ctx, "transactions:refund")?;
    command.transaction_id = transaction_id;

    let use_case = ProcessRefundUseCase::new(
        state.payment_gateway_repo(),
        state.transaction_repo(),
        state.gateway_registry(),
    );

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_transactions_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<TransactionListResponse>, Response> {
    require_permission(&ctx, "transactions:read")?;
    if let Some(sid) = query.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }

    let use_case = ListTransactionsUseCase::new(state.transaction_repo());
    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn get_transaction_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, Response> {
    require_permission(&ctx, "transactions:read")?;

    let use_case = GetTransactionUseCase::new(state.transaction_repo());
    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Manually confirm a pending transaction. Used after a manager verifies
/// the deposit appeared in the bank statement (BAC, Ficohsa, …) or after
/// the delivery person reports the cash on delivery was collected.
pub async fn confirm_transaction_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut command): JsonBody<ConfirmTransactionCommand>,
) -> Result<Json<TransactionResponse>, Response> {
    require_permission(&ctx, "transactions:confirm")?;
    command.transaction_id = id;
    command.confirmed_by_id = (*ctx.user_id()).into_uuid();

    let use_case = ConfirmTransactionUseCase::new(state.transaction_repo());
    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn reject_transaction_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut command): JsonBody<RejectTransactionCommand>,
) -> Result<Json<TransactionResponse>, Response> {
    require_permission(&ctx, "transactions:confirm")?;
    command.transaction_id = id;
    command.rejected_by_id = (*ctx.user_id()).into_uuid();

    let use_case = RejectTransactionUseCase::new(state.transaction_repo());
    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

/// Bulk-confirm pending transactions against a bank statement upload.
pub async fn reconcile_transactions_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(mut command): JsonBody<ReconcilePaymentsCommand>,
) -> Result<Json<ReconciliationResponse>, Response> {
    require_permission(&ctx, "transactions:reconcile")?;
    verify_store_in_org(state.pool(), &ctx, command.store_id).await?;
    command.confirmed_by_id = (*ctx.user_id()).into_uuid();

    let use_case = ReconcileManualPaymentsUseCase::new(state.transaction_repo());
    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
