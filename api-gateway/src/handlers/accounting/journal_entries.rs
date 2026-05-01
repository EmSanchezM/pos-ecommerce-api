//! Journal entry endpoints. Posting validates balance + period open + accounts
//! exist; the use case allocates the next entry_number for the period.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use accounting::{
    AccountingPeriodId, GetJournalEntryUseCase, JournalEntryId, JournalEntryResponse,
    ListJournalEntriesUseCase, PostJournalEntryCommand, PostJournalEntryUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListJournalEntriesQuery {
    pub period_id: Uuid,
}

pub async fn list_journal_entries_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListJournalEntriesQuery>,
) -> Result<Json<Vec<JournalEntryResponse>>, Response> {
    require_permission(&ctx, "accounting:read")?;

    let use_case = ListJournalEntriesUseCase::new(state.journal_entry_repo());
    let entries = use_case
        .execute(AccountingPeriodId::from_uuid(params.period_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(
        entries.iter().map(JournalEntryResponse::from).collect(),
    ))
}

pub async fn get_journal_entry_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<JournalEntryResponse>, Response> {
    require_permission(&ctx, "accounting:read")?;

    let use_case = GetJournalEntryUseCase::new(state.journal_entry_repo());
    let entry = use_case
        .execute(JournalEntryId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(JournalEntryResponse::from(&entry)))
}

pub async fn post_journal_entry_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(mut cmd): Json<PostJournalEntryCommand>,
) -> Result<Json<JournalEntryResponse>, Response> {
    require_permission(&ctx, "accounting:write")?;

    // Stamp the authenticated user as the creator if the client didn't supply one.
    if cmd.created_by.is_none() {
        cmd.created_by = Some(*ctx.user_id().as_uuid());
    }

    let use_case = PostJournalEntryUseCase::new(
        state.account_repo(),
        state.accounting_period_repo(),
        state.journal_entry_repo(),
    );
    let entry = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(JournalEntryResponse::from(&entry)))
}
