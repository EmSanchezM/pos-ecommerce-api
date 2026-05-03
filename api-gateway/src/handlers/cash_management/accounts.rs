//! Bank account CRUD endpoints.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use cash_management::{
    BankAccountId, BankAccountResponse, CreateBankAccountCommand, CreateBankAccountUseCase,
    DeactivateBankAccountUseCase, GetBankAccountUseCase, ListBankAccountsUseCase,
    UpdateBankAccountCommand, UpdateBankAccountUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListBankAccountsQuery {
    pub store_id: Option<Uuid>,
}

pub async fn list_bank_accounts_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListBankAccountsQuery>,
) -> Result<Json<Vec<BankAccountResponse>>, Response> {
    require_permission(&ctx, "cash_management:read_account")?;
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }
    let use_case = ListBankAccountsUseCase::new(state.bank_account_repo());
    let accounts = use_case
        .execute(params.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        accounts.iter().map(BankAccountResponse::from).collect(),
    ))
}

pub async fn get_bank_account_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BankAccountResponse>, Response> {
    require_permission(&ctx, "cash_management:read_account")?;
    let use_case = GetBankAccountUseCase::new(state.bank_account_repo());
    let account = use_case
        .execute(BankAccountId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankAccountResponse::from(&account)))
}

pub async fn create_bank_account_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateBankAccountCommand>,
) -> Result<Json<BankAccountResponse>, Response> {
    require_permission(&ctx, "cash_management:write_account")?;
    verify_store_in_org(state.pool(), &ctx, cmd.store_id).await?;
    let use_case = CreateBankAccountUseCase::new(state.bank_account_repo());
    let account = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankAccountResponse::from(&account)))
}

pub async fn update_bank_account_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateBankAccountCommand>,
) -> Result<Json<BankAccountResponse>, Response> {
    require_permission(&ctx, "cash_management:write_account")?;
    let use_case = UpdateBankAccountUseCase::new(state.bank_account_repo());
    let account = use_case
        .execute(BankAccountId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankAccountResponse::from(&account)))
}

pub async fn deactivate_bank_account_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BankAccountResponse>, Response> {
    require_permission(&ctx, "cash_management:write_account")?;
    let use_case = DeactivateBankAccountUseCase::new(state.bank_account_repo());
    let account = use_case
        .execute(BankAccountId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(BankAccountResponse::from(&account)))
}
