//! Reorder policy CRUD endpoints.

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use demand_planning::{
    ListReorderPoliciesUseCase, ReorderPolicyResponse, UpsertReorderPolicyCommand,
    UpsertReorderPolicyUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::verify_store_in_org;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub store_id: Option<Uuid>,
}

pub async fn list_reorder_policies_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListPoliciesQuery>,
) -> Result<Json<Vec<ReorderPolicyResponse>>, Response> {
    require_permission(&ctx, "demand_planning:read_policy")?;
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }
    let use_case = ListReorderPoliciesUseCase::new(state.reorder_policy_repo());
    let policies = use_case
        .execute(params.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        policies.iter().map(ReorderPolicyResponse::from).collect(),
    ))
}

pub async fn upsert_reorder_policy_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<UpsertReorderPolicyCommand>,
) -> Result<Json<ReorderPolicyResponse>, Response> {
    require_permission(&ctx, "demand_planning:write_policy")?;
    verify_store_in_org(state.pool(), &ctx, cmd.store_id).await?;
    let use_case = UpsertReorderPolicyUseCase::new(state.reorder_policy_repo());
    let policy = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ReorderPolicyResponse::from(&policy)))
}
