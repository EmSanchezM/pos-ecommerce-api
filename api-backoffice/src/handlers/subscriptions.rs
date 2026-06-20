// Subscription admin handlers (Phase 6 — Slice B)
//
// Platform-owner control over an organization's subscription:
//   GET  /backoffice/subscriptions/{org_id}              read   — platform:org.list
//   POST /backoffice/subscriptions/{org_id}/force-cancel        — platform:subscription.force_cancel
//   POST /backoffice/subscriptions/{org_id}/change-plan         — platform:subscription.override_billing
//   POST /backoffice/subscriptions/{org_id}/resume              — platform:subscription.override_billing
//
// The read endpoint reuses `platform:org.list`: an operator who can see the org
// directory can inspect an org's subscription (no dedicated subscription-read
// permission is seeded). Per OQ resolution, `override_billing` covers BOTH
// change-plan and resume.
//
// All mutations require a `reason` and emit a `backoffice.audit.subscription.*`
// event via crate::audit::emit_state_change_audit (fail-open, not atomic with
// the write — see that helper's docs).

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use subscriptions::{
    CancelSubscriptionCommand, CancelSubscriptionUseCase, ChangePlanCommand, ChangePlanUseCase,
    GetSubscriptionUseCase, ResumeSubscriptionCommand, ResumeSubscriptionUseCase,
};

use crate::audit::{begin_tx, commit_with_audit};
use crate::error::AppError;
use crate::handlers::reason_guard;
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// Body for mutations that only need an audit `reason`.
#[derive(Debug, Deserialize)]
pub struct ReasonRequest {
    pub reason: String,
}

/// Body for change-plan: audit `reason` + the target plan.
#[derive(Debug, Deserialize)]
pub struct ChangePlanRequest {
    pub reason: String,
    pub new_plan_id: Uuid,
}

/// GET /backoffice/subscriptions/{org_id} — the org's active subscription.
pub async fn get_subscription_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:org.list")?;

    let use_case = GetSubscriptionUseCase::new(state.subscription_repo());
    let subscription = use_case
        .execute(org_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(subscription))
}

/// POST /backoffice/subscriptions/{org_id}/force-cancel — terminate immediately.
pub async fn force_cancel_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<ReasonRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:subscription.force_cancel")?;
    reason_guard(&body.reason)?;

    let mut tx = begin_tx(state.pool())
        .await
        .map_err(IntoResponse::into_response)?;
    let use_case = CancelSubscriptionUseCase::new(state.subscription_repo());
    let subscription = use_case
        .execute_in_tx(
            &mut tx,
            CancelSubscriptionCommand {
                organization_id: org_id,
                immediately: true,
            },
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    commit_with_audit(
        tx,
        &state.publish_event(),
        ctx.user_id,
        "subscription.force_cancel",
        Some(org_id),
        body.reason,
    )
    .await
    .map_err(IntoResponse::into_response)?;

    Ok((StatusCode::OK, Json(subscription)))
}

/// POST /backoffice/subscriptions/{org_id}/change-plan — switch billed plan.
pub async fn change_plan_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<ChangePlanRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:subscription.override_billing")?;
    reason_guard(&body.reason)?;

    let mut tx = begin_tx(state.pool())
        .await
        .map_err(IntoResponse::into_response)?;
    let use_case =
        ChangePlanUseCase::new(state.subscription_plan_repo(), state.subscription_repo());
    let subscription = use_case
        .execute_in_tx(
            &mut tx,
            ChangePlanCommand {
                organization_id: org_id,
                new_plan_id: body.new_plan_id,
            },
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    commit_with_audit(
        tx,
        &state.publish_event(),
        ctx.user_id,
        "subscription.change_plan",
        Some(org_id),
        body.reason,
    )
    .await
    .map_err(IntoResponse::into_response)?;

    Ok((StatusCode::OK, Json(subscription)))
}

/// POST /backoffice/subscriptions/{org_id}/resume — reactivate a past-due sub.
pub async fn resume_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<ReasonRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:subscription.override_billing")?;
    reason_guard(&body.reason)?;

    let mut tx = begin_tx(state.pool())
        .await
        .map_err(IntoResponse::into_response)?;
    let use_case = ResumeSubscriptionUseCase::new(state.subscription_repo());
    let subscription = use_case
        .execute_in_tx(
            &mut tx,
            ResumeSubscriptionCommand {
                organization_id: org_id,
            },
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    commit_with_audit(
        tx,
        &state.publish_event(),
        ctx.user_id,
        "subscription.resume",
        Some(org_id),
        body.reason,
    )
    .await
    .map_err(IntoResponse::into_response)?;

    Ok((StatusCode::OK, Json(subscription)))
}
