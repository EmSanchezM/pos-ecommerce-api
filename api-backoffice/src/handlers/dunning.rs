// Manual dunning trigger handler (Phase 6 — Slice C)
//
//   POST /backoffice/dunning/{attempt_id}/trigger — platform:dunning.trigger
//
// Manually fires a scheduled dunning attempt instead of waiting for the billing
// job: it re-charges via the (v1.0 stub) payment gateway, stamps a
// transaction_id on the attempt, and leaves the outcome Pending until the
// payment webhook resolves it. Already-fired attempts are a no-op (200).
//
// Requires a `reason` and emits a `backoffice.audit.dunning.trigger` event. The
// affected org is resolved (attempt -> cycle -> subscription) for the audit
// trail; an unknown attempt 404s before any charge. Audit is fail-open and not
// atomic — see crate::audit::emit_state_change_audit.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use subscriptions::{DunningAttemptId, ProcessDunningAttemptUseCase, SubscriptionError};

use crate::audit::emit_state_change_audit;
use crate::error::AppError;
use crate::handlers::reason_guard;
use crate::handlers::subscriptions::ReasonRequest;
use crate::middleware::auth::BackofficeUserContext;
use crate::middleware::permission::require_backoffice_permission;
use crate::state::BackofficeAppState;

/// POST /backoffice/dunning/{attempt_id}/trigger — manually fire a dunning attempt.
pub async fn trigger_dunning_handler(
    State(state): State<BackofficeAppState>,
    Extension(ctx): Extension<BackofficeUserContext>,
    Path(attempt_id): Path<Uuid>,
    Json(body): Json<ReasonRequest>,
) -> Result<impl IntoResponse, Response> {
    require_backoffice_permission(&ctx, "platform:dunning.trigger")?;
    reason_guard(&body.reason)?;

    let aid = DunningAttemptId::from_uuid(attempt_id);

    // Resolve the affected org for the audit trail. Also 404s early when the
    // attempt is unknown, matching the use case's own behaviour.
    let org_id = resolve_org_for_attempt(&state, aid)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    let use_case = ProcessDunningAttemptUseCase::new(
        state.subscription_repo(),
        state.billing_cycle_repo(),
        state.dunning_repo(),
        state.dunning_payment_gateway(),
    );
    use_case
        .execute(aid)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    emit_state_change_audit(
        state.pool(),
        &state.publish_event(),
        ctx.user_id,
        "dunning.trigger",
        org_id,
        body.reason,
    )
    .await;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "attempt_id": attempt_id,
            "message": "Dunning attempt triggered; charge pending webhook resolution",
        })),
    ))
}

/// Resolves attempt → cycle → subscription → org for the audit `target_org_id`.
/// Returns `Err(DunningAttemptNotFound)` if the attempt does not exist so the
/// handler can 404 before charging.
async fn resolve_org_for_attempt(
    state: &BackofficeAppState,
    attempt_id: DunningAttemptId,
) -> Result<Option<Uuid>, SubscriptionError> {
    let Some(attempt) = state.dunning_repo().find_by_id(attempt_id).await? else {
        return Err(SubscriptionError::DunningAttemptNotFound(
            attempt_id.into_uuid(),
        ));
    };
    let Some(cycle) = state
        .billing_cycle_repo()
        .find_by_id(attempt.billing_cycle_id())
        .await?
    else {
        return Ok(None);
    };
    let Some(sub) = state
        .subscription_repo()
        .find_by_id(cycle.subscription_id())
        .await?
    else {
        return Ok(None);
    };
    Ok(Some(sub.organization_id().into_uuid()))
}
