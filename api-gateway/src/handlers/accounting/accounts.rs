//! Chart-of-accounts CRUD endpoints.

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use accounting::{
    AccountResponse, AccountType, CreateAccountCommand, CreateAccountUseCase, ListAccountsUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    pub account_type: Option<AccountType>,
}

pub async fn list_accounts_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListAccountsQuery>,
) -> Result<Json<Vec<AccountResponse>>, Response> {
    require_permission(&ctx, "accounting:read")?;

    let use_case = ListAccountsUseCase::new(state.account_repo());
    let accounts = use_case
        .execute(params.account_type)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(accounts.iter().map(AccountResponse::from).collect()))
}

pub async fn create_account_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateAccountCommand>,
) -> Result<Json<AccountResponse>, Response> {
    require_permission(&ctx, "accounting:write")?;

    let use_case = CreateAccountUseCase::new(state.account_repo());
    let account = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(AccountResponse::from(&account)))
}
