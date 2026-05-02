//! Bank transaction endpoints (manual entry + list with filters).

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use cash_management::{
    BankAccountId, BankTransactionFilter, BankTransactionResponse, ListBankTransactionsUseCase,
    RecordBankTransactionCommand, RecordBankTransactionUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub bank_account_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub reconciled: Option<bool>,
}

pub async fn list_bank_transactions_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListTransactionsQuery>,
) -> Result<Json<Vec<BankTransactionResponse>>, Response> {
    require_permission(&ctx, "cash_management:read_transaction")?;
    let filter = BankTransactionFilter {
        bank_account_id: params.bank_account_id.map(BankAccountId::from_uuid),
        from: params.from,
        to: params.to,
        reconciled: params.reconciled,
    };
    let use_case = ListBankTransactionsUseCase::new(state.bank_transaction_repo());
    let txns = use_case
        .execute(filter)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        txns.iter().map(BankTransactionResponse::from).collect(),
    ))
}

pub async fn record_bank_transaction_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<RecordBankTransactionCommand>,
) -> Result<Json<BankTransactionResponse>, Response> {
    require_permission(&ctx, "cash_management:write_transaction")?;
    let use_case =
        RecordBankTransactionUseCase::new(state.bank_account_repo(), state.bank_transaction_repo());
    let txn = use_case
        .execute(cmd, Some(*ctx.user_id().as_uuid()))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankTransactionResponse::from(&txn)))
}
