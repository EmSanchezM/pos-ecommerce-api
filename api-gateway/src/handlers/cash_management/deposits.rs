//! Cash deposit endpoints (create + list + mark-sent + link-to-transaction).

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use cash_management::{
    CashDepositId, CashDepositResponse, CashDepositStatus, CreateCashDepositCommand,
    CreateCashDepositUseCase, LinkDepositCommand, LinkDepositToTransactionUseCase,
    ListCashDepositsUseCase, MarkDepositSentCommand, MarkDepositSentUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListDepositsQuery {
    pub store_id: Option<Uuid>,
    pub status: Option<CashDepositStatus>,
}

pub async fn list_cash_deposits_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListDepositsQuery>,
) -> Result<Json<Vec<CashDepositResponse>>, Response> {
    require_permission(&ctx, "cash_management:read_deposit")?;
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }
    let use_case = ListCashDepositsUseCase::new(state.cash_deposit_repo());
    let deposits = use_case
        .execute(params.store_id, params.status)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        deposits.iter().map(CashDepositResponse::from).collect(),
    ))
}

pub async fn create_cash_deposit_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateCashDepositCommand>,
) -> Result<Json<CashDepositResponse>, Response> {
    require_permission(&ctx, "cash_management:write_deposit")?;
    let use_case = CreateCashDepositUseCase::new(
        state.bank_account_repo(),
        state.cash_deposit_repo(),
        state.pool().clone(),
    );
    let deposit = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(CashDepositResponse::from(&deposit)))
}

pub async fn mark_deposit_sent_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<MarkDepositSentCommand>,
) -> Result<Json<CashDepositResponse>, Response> {
    require_permission(&ctx, "cash_management:write_deposit")?;
    let use_case = MarkDepositSentUseCase::new(state.cash_deposit_repo());
    let deposit = use_case
        .execute(CashDepositId::from_uuid(id), *ctx.user_id().as_uuid(), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(CashDepositResponse::from(&deposit)))
}

pub async fn link_deposit_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<LinkDepositCommand>,
) -> Result<Json<CashDepositResponse>, Response> {
    require_permission(&ctx, "cash_management:link_deposit")?;
    let use_case = LinkDepositToTransactionUseCase::new(
        state.cash_deposit_repo(),
        state.bank_transaction_repo(),
    );
    let deposit = use_case
        .execute(CashDepositId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(CashDepositResponse::from(&deposit)))
}
