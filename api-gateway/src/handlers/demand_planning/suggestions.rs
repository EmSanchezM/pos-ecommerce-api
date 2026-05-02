//! Replenishment suggestion endpoints (list + approve + dismiss).

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use demand_planning::{
    ApproveSuggestionCommand, ApproveSuggestionUseCase, DismissSuggestionCommand,
    DismissSuggestionUseCase, ListReplenishmentSuggestionsUseCase, ReplenishmentSuggestionResponse,
    SuggestionId, SuggestionStatus,
};
use purchasing::CreatePurchaseOrderUseCase;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListSuggestionsQuery {
    pub store_id: Option<Uuid>,
    pub status: Option<SuggestionStatus>,
}

pub async fn list_replenishment_suggestions_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListSuggestionsQuery>,
) -> Result<Json<Vec<ReplenishmentSuggestionResponse>>, Response> {
    require_permission(&ctx, "demand_planning:read_suggestion")?;
    let use_case = ListReplenishmentSuggestionsUseCase::new(state.replenishment_suggestion_repo());
    let suggestions = use_case
        .execute(params.store_id, params.status)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        suggestions
            .iter()
            .map(ReplenishmentSuggestionResponse::from)
            .collect(),
    ))
}

pub async fn approve_suggestion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<ApproveSuggestionCommand>,
) -> Result<Json<ReplenishmentSuggestionResponse>, Response> {
    require_permission(&ctx, "demand_planning:approve_suggestion")?;

    let create_po = Arc::new(CreatePurchaseOrderUseCase::new(
        state.purchase_order_repo(),
        state.vendor_repo(),
    ));
    let use_case = ApproveSuggestionUseCase::new(state.replenishment_suggestion_repo(), create_po);
    let suggestion = use_case
        .execute(SuggestionId::from_uuid(id), *ctx.user_id().as_uuid(), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ReplenishmentSuggestionResponse::from(&suggestion)))
}

pub async fn dismiss_suggestion_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<DismissSuggestionCommand>,
) -> Result<Json<ReplenishmentSuggestionResponse>, Response> {
    require_permission(&ctx, "demand_planning:dismiss_suggestion")?;
    let use_case = DismissSuggestionUseCase::new(state.replenishment_suggestion_repo());
    let suggestion = use_case
        .execute(SuggestionId::from_uuid(id), *ctx.user_id().as_uuid(), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ReplenishmentSuggestionResponse::from(&suggestion)))
}
