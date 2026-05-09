//! `/api/v1/organizations/{org_id}/subscription/*` handlers — and admin
//! cross-org listing.
//!
//! Every endpoint here calls `require_org_match` so an org_admin from org A
//! can never reach the subscription of org B. `super_admin` bypasses, which
//! is what the admin listing relies on.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use subscriptions::{
    CancelSubscriptionCommand, CancelSubscriptionUseCase, ChangePlanCommand, ChangePlanUseCase,
    GetSubscriptionUseCase, ListBillingCyclesQuery, ListBillingCyclesUseCase,
    PaginatedBillingCycles, ResumeSubscriptionCommand, ResumeSubscriptionUseCase,
    SubscribeOrganizationCommand, SubscribeOrganizationUseCase, SubscriptionResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::require_org_match;
use crate::middleware::permission::{require_permission, require_super_admin};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CycleListQuery {
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_page_size")]
    page_size: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct SubscribeBody {
    pub plan_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct CancelBody {
    /// `true` → cancel immediately. Reserved for super_admin; non-admins
    /// always get the period-end semantics regardless of the flag.
    #[serde(default)]
    pub immediately: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChangePlanBody {
    pub new_plan_id: Uuid,
}

fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    50
}

pub async fn subscribe_organization_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(body): Json<SubscribeBody>,
) -> Result<Json<SubscriptionResponse>, Response> {
    require_permission(&ctx, "subscriptions:write_subscription")?;
    require_org_match(&ctx, org_id)?;
    let use_case = SubscribeOrganizationUseCase::new(
        state.subscription_plan_repo(),
        state.subscription_repo(),
        state.billing_cycle_repo(),
    );
    let sub = use_case
        .execute(SubscribeOrganizationCommand {
            organization_id: org_id,
            plan_id: body.plan_id,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(sub))
}

pub async fn get_subscription_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
) -> Result<Json<SubscriptionResponse>, Response> {
    require_permission(&ctx, "subscriptions:read_subscription")?;
    require_org_match(&ctx, org_id)?;
    let use_case = GetSubscriptionUseCase::new(state.subscription_repo());
    let sub = use_case
        .execute(org_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(sub))
}

pub async fn list_billing_cycles_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Query(q): Query<CycleListQuery>,
) -> Result<Json<PaginatedBillingCycles>, Response> {
    require_permission(&ctx, "subscriptions:read_subscription")?;
    require_org_match(&ctx, org_id)?;
    let use_case =
        ListBillingCyclesUseCase::new(state.subscription_repo(), state.billing_cycle_repo());
    let cycles = use_case
        .execute(ListBillingCyclesQuery {
            organization_id: org_id,
            page: q.page,
            page_size: q.page_size,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(cycles))
}

pub async fn cancel_subscription_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(body): Json<CancelBody>,
) -> Result<Json<SubscriptionResponse>, Response> {
    require_permission(&ctx, "subscriptions:cancel_subscription")?;
    require_org_match(&ctx, org_id)?;
    // `immediately=true` is a super_admin-only escalation — org_admins always
    // get the safer cancel-at-period-end semantics regardless of body input
    // so they can reverse the decision via `resume` while the period is alive.
    let immediately = body.immediately && require_super_admin(&ctx).is_ok();
    let use_case = CancelSubscriptionUseCase::new(state.subscription_repo());
    let sub = use_case
        .execute(CancelSubscriptionCommand {
            organization_id: org_id,
            immediately,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(sub))
}

pub async fn resume_subscription_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
) -> Result<Json<SubscriptionResponse>, Response> {
    require_permission(&ctx, "subscriptions:cancel_subscription")?;
    require_org_match(&ctx, org_id)?;
    let use_case = ResumeSubscriptionUseCase::new(state.subscription_repo());
    let sub = use_case
        .execute(ResumeSubscriptionCommand {
            organization_id: org_id,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(sub))
}

pub async fn change_plan_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(org_id): Path<Uuid>,
    Json(body): Json<ChangePlanBody>,
) -> Result<Json<SubscriptionResponse>, Response> {
    require_permission(&ctx, "subscriptions:write_subscription")?;
    require_org_match(&ctx, org_id)?;
    let use_case =
        ChangePlanUseCase::new(state.subscription_plan_repo(), state.subscription_repo());
    let sub = use_case
        .execute(ChangePlanCommand {
            organization_id: org_id,
            new_plan_id: body.new_plan_id,
        })
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(sub))
}

/// Admin-only cross-org listing. v1.0 keeps this a placeholder returning
/// 501 because the underlying repo doesn't expose a `list_all` query yet —
/// the data is still reachable per-org via the standard endpoints.
pub async fn list_subscriptions_admin_handler(
    State(_state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
) -> Result<Response, Response> {
    require_super_admin(&ctx)?;
    Ok((
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error_code": "NOT_IMPLEMENTED",
            "message": "Admin-wide subscription listing lands in v1.1; query per-organization for now."
        })),
    )
        .into_response())
}
