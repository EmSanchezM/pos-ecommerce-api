// Credit note handlers for the Sales module (Returns)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use sales::{
    AddCreditNoteItemCommand, ApplyCreditNoteCommand, CancelCreditNoteCommand,
    CreateCreditNoteCommand, CreditNoteListResponse, CreditNoteResponse, ListCreditNotesQuery,
};

pub async fn create_credit_note_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateCreditNoteCommand>,
) -> Result<(StatusCode, Json<CreditNoteResponse>), Response> {
    require_permission(&ctx, "sales:manage_credit_note")?;

    let use_case = sales::CreateCreditNoteUseCase::new(state.credit_note_repo());

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn add_credit_note_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(credit_note_id): Path<Uuid>,
    Json(command): Json<AddCreditNoteItemCommand>,
) -> Result<(StatusCode, Json<CreditNoteResponse>), Response> {
    require_permission(&ctx, "sales:manage_credit_note")?;

    let use_case = sales::AddCreditNoteItemUseCase::new(state.credit_note_repo());

    let mut cmd = command;
    cmd.credit_note_id = credit_note_id;

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn remove_credit_note_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((credit_note_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CreditNoteResponse>, Response> {
    require_permission(&ctx, "sales:manage_credit_note")?;

    let use_case = sales::RemoveCreditNoteItemUseCase::new(state.credit_note_repo());

    let response = use_case
        .execute(credit_note_id, item_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn submit_credit_note_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(credit_note_id): Path<Uuid>,
) -> Result<Json<CreditNoteResponse>, Response> {
    require_permission(&ctx, "sales:manage_credit_note")?;

    let use_case = sales::SubmitCreditNoteUseCase::new(state.credit_note_repo());

    let response = use_case
        .execute(credit_note_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn approve_credit_note_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(credit_note_id): Path<Uuid>,
) -> Result<Json<CreditNoteResponse>, Response> {
    require_permission(&ctx, "sales:approve_credit_note")?;

    let use_case = sales::ApproveCreditNoteUseCase::new(state.credit_note_repo());

    let response = use_case
        .execute(credit_note_id, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn apply_credit_note_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(credit_note_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<CreditNoteResponse>, Response> {
    require_permission(&ctx, "sales:manage_credit_note")?;

    let use_case = sales::ApplyCreditNoteUseCase::new(state.credit_note_repo());

    let refund_method = body
        .get("refund_method")
        .and_then(|v| v.as_str())
        .unwrap_or("cash")
        .to_string();

    let cmd = ApplyCreditNoteCommand {
        credit_note_id,
        refund_method,
    };

    let response = use_case
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn cancel_credit_note_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(credit_note_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<CreditNoteResponse>, Response> {
    require_permission(&ctx, "sales:manage_credit_note")?;

    let use_case = sales::CancelCreditNoteUseCase::new(state.credit_note_repo());

    let reason = body
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("No reason provided")
        .to_string();

    let cmd = CancelCreditNoteCommand {
        credit_note_id,
        reason,
    };

    let response = use_case
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn get_credit_note_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(credit_note_id): Path<Uuid>,
) -> Result<Json<CreditNoteResponse>, Response> {
    require_permission(&ctx, "sales:read_credit_note")?;

    let use_case = sales::GetCreditNoteUseCase::new(state.credit_note_repo());

    let response = use_case
        .execute(credit_note_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn list_credit_notes_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListCreditNotesQuery>,
) -> Result<Json<CreditNoteListResponse>, Response> {
    require_permission(&ctx, "sales:read_credit_note")?;

    let use_case = sales::ListCreditNotesUseCase::new(state.credit_note_repo());

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
