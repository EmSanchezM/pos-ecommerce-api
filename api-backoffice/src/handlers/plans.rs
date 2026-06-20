// Subscription plan catalog admin handlers (Phase 6 — Slice A)
//
// Plans are the SaaS billing catalog (price + cadence). The platform owner
// manages them here:
//   GET  /backoffice/plans                 list (paginated) — platform:plan.read
//   GET  /backoffice/plans/{id}            get              — platform:plan.read
//   POST /backoffice/plans                 create           — platform:plan.create
//   PUT  /backoffice/plans/{id}            update           — platform:plan.update
//   POST /backoffice/plans/{id}/deactivate deactivate       — platform:plan.update
//
// Mutations require a `reason` and emit a `backoffice.audit.plan.*` event.
//
// Audit posture: the event is emitted AFTER the plan write, in its own
// transaction, and is fail-open (a failed audit is logged, the request still
// succeeds) — mirroring the api-gateway impersonated-request audit. It is NOT
// atomic with the plan write because the subscriptions plan repository exposes
// no tx-aware methods (unlike tenancy's repo, which is why the Phase 4 org
// suspend is atomic). Making plan audit atomic is a tracked follow-up.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use subscriptions::{
    CreatePlanCommand, CreatePlanUseCase, DeactivatePlanUseCase, GetPlanUseCase, ListPlansQuery,
    ListPlansUseCase, UpdatePlanCommand, UpdatePlanUseCase,
};

use crate::audit::emit_state_change_audit;
use crate::error::AppError;
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// Wraps a mutating plan command with the mandatory audit `reason`.
#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub reason: String,
    #[serde(flatten)]
    pub plan: CreatePlanCommand,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlanRequest {
    pub reason: String,
    #[serde(flatten)]
    pub plan: UpdatePlanCommand,
}

#[derive(Debug, Deserialize)]
pub struct DeactivatePlanRequest {
    pub reason: String,
}

/// GET /backoffice/plans — paginated admin listing (active + inactive).
pub async fn list_plans_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Query(query): Query<ListPlansQuery>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:plan.read")?;

    let use_case = ListPlansUseCase::new(state.subscription_plan_repo());
    let page = use_case
        .list_paginated(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(page))
}

/// GET /backoffice/plans/{id} — fetch a single plan.
pub async fn get_plan_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(plan_id): Path<Uuid>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:plan.read")?;

    let use_case = GetPlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(plan_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(plan))
}

/// POST /backoffice/plans — create a new plan in the catalog.
pub async fn create_plan_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Json(body): Json<CreatePlanRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:plan.create")?;
    crate::handlers::reason_guard(&body.reason)?;

    let use_case = CreatePlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(body.plan)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    emit_state_change_audit(
        state.pool(),
        &state.publish_event(),
        ctx.user_id,
        "plan.create",
        None,
        body.reason,
    )
    .await;

    Ok((StatusCode::CREATED, Json(plan)))
}

/// PUT /backoffice/plans/{id} — update mutable fields of a plan.
pub async fn update_plan_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(plan_id): Path<Uuid>,
    Json(body): Json<UpdatePlanRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:plan.update")?;
    crate::handlers::reason_guard(&body.reason)?;

    let use_case = UpdatePlanUseCase::new(state.subscription_plan_repo());
    let plan = use_case
        .execute(plan_id, body.plan)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    emit_state_change_audit(
        state.pool(),
        &state.publish_event(),
        ctx.user_id,
        "plan.update",
        None,
        body.reason,
    )
    .await;

    Ok((StatusCode::OK, Json(plan)))
}

/// POST /backoffice/plans/{id}/deactivate — soft-disable a plan.
pub async fn deactivate_plan_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(plan_id): Path<Uuid>,
    Json(body): Json<DeactivatePlanRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:plan.update")?;
    crate::handlers::reason_guard(&body.reason)?;

    let use_case = DeactivatePlanUseCase::new(state.subscription_plan_repo());
    use_case
        .execute(plan_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    emit_state_change_audit(
        state.pool(),
        &state.publish_event(),
        ctx.user_id,
        "plan.deactivate",
        None,
        body.reason,
    )
    .await;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "plan_id": plan_id,
            "is_active": false,
            "message": "Plan deactivated",
        })),
    ))
}
