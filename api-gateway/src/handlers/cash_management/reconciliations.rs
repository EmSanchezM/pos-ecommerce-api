//! Bank reconciliation endpoints (start + list + close).

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use cash_management::{
    BankAccountId, BankReconciliationId, BankReconciliationResponse, CloseReconciliationCommand,
    CloseReconciliationUseCase, ListReconciliationsUseCase, StartReconciliationCommand,
    StartReconciliationUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListReconciliationsQuery {
    pub bank_account_id: Uuid,
}

pub async fn list_reconciliations_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListReconciliationsQuery>,
) -> Result<Json<Vec<BankReconciliationResponse>>, Response> {
    require_permission(&ctx, "cash_management:read_reconciliation")?;
    let use_case = ListReconciliationsUseCase::new(state.bank_reconciliation_repo());
    let recons = use_case
        .execute(BankAccountId::from_uuid(params.bank_account_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        recons
            .iter()
            .map(BankReconciliationResponse::from)
            .collect(),
    ))
}

pub async fn start_reconciliation_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<StartReconciliationCommand>,
) -> Result<Json<BankReconciliationResponse>, Response> {
    require_permission(&ctx, "cash_management:write_reconciliation")?;
    let use_case = StartReconciliationUseCase::new(
        state.bank_account_repo(),
        state.bank_reconciliation_repo(),
    );
    let recon = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankReconciliationResponse::from(&recon)))
}

pub async fn close_reconciliation_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<CloseReconciliationCommand>,
) -> Result<Json<BankReconciliationResponse>, Response> {
    require_permission(&ctx, "cash_management:close_reconciliation")?;
    let use_case = CloseReconciliationUseCase::new(
        state.bank_reconciliation_repo(),
        state.bank_transaction_repo(),
    );
    let recon = use_case
        .execute(
            BankReconciliationId::from_uuid(id),
            *ctx.user_id().as_uuid(),
            cmd,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankReconciliationResponse::from(&recon)))
}
