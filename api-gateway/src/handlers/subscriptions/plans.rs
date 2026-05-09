//! `/api/v1/subscription-plans` handlers — public listing/lookup +
//! authenticated CRUD restricted by `subscriptions:write_plan`.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use subscriptions::{
    CreatePlanCommand, CreatePlanUseCase, DeactivatePlanUseCase, GetPlanUseCase, ListPlansQuery,
    ListPlansUseCase, PaginatedPlans, PlanResponse, UpdatePlanCommand, UpdatePlanUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

/// PUBLIC: every active plan, ordered by `sort_order, created_at`.
pub async fn public_list_plans_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlanResponse>>, Response> {
    let use_case = ListPlansUseCase::new(state.subscription_plan_repo());
    let plans = use_case
        .list_active()
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plans))
}

/// PUBLIC: lookup a single plan by id. Returns `is_active=false` rows too —
/// the caller may already have a subscription on a now-retired plan.
pub async fn public_get_plan_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PlanResponse>, Response> {
    let use_case = GetPlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plan))
}

#[derive(Debug, Deserialize)]
pub struct ListPlansAdminQuery {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_page_size")]
    page_size: i64,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    50
}

/// Same as `public_list_plans_handler` but reachable while authed; useful for
/// the admin SPA which embeds active-only listings without exposing /public.
pub async fn list_plans_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> Result<Json<Vec<PlanResponse>>, Response> {
    require_permission(&ctx, "subscriptions:read_plan")?;
    let use_case = ListPlansUseCase::new(state.subscription_plan_repo());
    let plans = use_case
        .list_active()
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plans))
}

/// Admin paginated listing — every plan, active or not.
pub async fn list_plans_paginated_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(q): Query<ListPlansAdminQuery>,
) -> Result<Json<PaginatedPlans>, Response> {
    require_permission(&ctx, "subscriptions:read_plan")?;
    let use_case = ListPlansUseCase::new(state.subscription_plan_repo());
    let plans = use_case
        .list_paginated(ListPlansQuery {
            page: q.page,
            page_size: q.page_size,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plans))
}

pub async fn get_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PlanResponse>, Response> {
    require_permission(&ctx, "subscriptions:read_plan")?;
    let use_case = GetPlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plan))
}

pub async fn create_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreatePlanCommand>,
) -> Result<Json<PlanResponse>, Response> {
    require_permission(&ctx, "subscriptions:write_plan")?;
    let use_case = CreatePlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plan))
}

pub async fn update_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdatePlanCommand>,
) -> Result<Json<PlanResponse>, Response> {
    require_permission(&ctx, "subscriptions:write_plan")?;
    let use_case = UpdatePlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(id, cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(plan))
}

pub async fn deactivate_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    require_permission(&ctx, "subscriptions:write_plan")?;
    let use_case = DeactivatePlanUseCase::new(state.subscription_plan_repo());
    use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT.into_response())
}
