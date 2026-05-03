// Payout handlers

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use payments::{ListPayoutsUseCase, PayoutListResponse};

#[derive(Debug, Deserialize)]
pub struct ListPayoutsQuery {
    pub store_id: Uuid,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn list_payouts_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListPayoutsQuery>,
) -> Result<Json<PayoutListResponse>, Response> {
    require_permission(&ctx, "payouts:read")?;
    verify_store_in_org(state.pool(), &ctx, query.store_id).await?;

    let use_case = ListPayoutsUseCase::new(state.payout_repo());
    let response = use_case
        .execute(query.store_id, query.page, query.page_size)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
