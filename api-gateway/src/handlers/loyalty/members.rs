//! Member endpoints — enroll, list/get, ledger, earn/adjust/redeem.

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use loyalty::{
    AdjustPointsCommand, AdjustPointsUseCase, EarnPointsCommand, EarnPointsUseCase,
    EnrollMemberCommand, EnrollMemberUseCase, GetLoyaltyMemberUseCase, GetMemberLedgerUseCase,
    ListLoyaltyMembersUseCase, LoyaltyMemberId, LoyaltyMemberResponse, LoyaltyProgramId,
    PointsLedgerEntryResponse, RedeemRewardCommand, RedeemRewardUseCase, RewardRedemptionResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListMembersQuery {
    pub program_id: Uuid,
}

pub async fn list_members_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListMembersQuery>,
) -> Result<Json<Vec<LoyaltyMemberResponse>>, Response> {
    require_permission(&ctx, "loyalty:read_member")?;
    let use_case = ListLoyaltyMembersUseCase::new(state.loyalty_member_repo());
    let members = use_case
        .execute(LoyaltyProgramId::from_uuid(params.program_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        members.iter().map(LoyaltyMemberResponse::from).collect(),
    ))
}

pub async fn get_member_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<LoyaltyMemberResponse>, Response> {
    require_permission(&ctx, "loyalty:read_member")?;
    let use_case = GetLoyaltyMemberUseCase::new(state.loyalty_member_repo());
    let member = use_case
        .execute(LoyaltyMemberId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(LoyaltyMemberResponse::from(&member)))
}

pub async fn enroll_member_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<EnrollMemberCommand>,
) -> Result<Json<LoyaltyMemberResponse>, Response> {
    require_permission(&ctx, "loyalty:enroll_member")?;
    let use_case = EnrollMemberUseCase::new(
        state.loyalty_program_repo(),
        state.loyalty_member_repo(),
        state.pool().clone(),
    );
    let member = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(LoyaltyMemberResponse::from(&member)))
}

#[derive(Debug, Deserialize)]
pub struct LedgerQuery {
    pub limit: Option<i64>,
}

pub async fn get_member_ledger_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Query(params): Query<LedgerQuery>,
) -> Result<Json<Vec<PointsLedgerEntryResponse>>, Response> {
    require_permission(&ctx, "loyalty:read_member")?;
    let use_case = GetMemberLedgerUseCase::new(state.points_ledger_repo());
    let entries = use_case
        .execute(LoyaltyMemberId::from_uuid(id), params.limit.unwrap_or(50))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        entries
            .iter()
            .map(PointsLedgerEntryResponse::from)
            .collect(),
    ))
}

pub async fn earn_points_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<EarnPointsCommand>,
) -> Result<Json<LoyaltyMemberResponse>, Response> {
    // Earn is admin-driven in v1; subscriber takes over once publishers ship.
    require_permission(&ctx, "loyalty:adjust_points")?;
    let use_case = EarnPointsUseCase::new(
        state.loyalty_program_repo(),
        state.loyalty_member_repo(),
        state.member_tier_repo(),
        state.points_ledger_repo(),
    );
    use_case
        .execute(
            LoyaltyMemberId::from_uuid(id),
            Some(*ctx.user_id().as_uuid()),
            cmd,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    // Re-read so the response includes the new tier (the use case re-computes
    // tier and persists; the post result only carries balance/lifetime).
    let member = GetLoyaltyMemberUseCase::new(state.loyalty_member_repo())
        .execute(LoyaltyMemberId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(LoyaltyMemberResponse::from(&member)))
}

pub async fn adjust_points_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<AdjustPointsCommand>,
) -> Result<Json<LoyaltyMemberResponse>, Response> {
    require_permission(&ctx, "loyalty:adjust_points")?;
    let use_case = AdjustPointsUseCase::new(
        state.loyalty_member_repo(),
        state.member_tier_repo(),
        state.points_ledger_repo(),
    );
    use_case
        .execute(
            LoyaltyMemberId::from_uuid(id),
            *ctx.user_id().as_uuid(),
            cmd,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    let member = GetLoyaltyMemberUseCase::new(state.loyalty_member_repo())
        .execute(LoyaltyMemberId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(LoyaltyMemberResponse::from(&member)))
}

pub async fn redeem_reward_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<RedeemRewardCommand>,
) -> Result<Json<RewardRedemptionResponse>, Response> {
    require_permission(&ctx, "loyalty:redeem_reward")?;
    let use_case = RedeemRewardUseCase::new(
        state.loyalty_member_repo(),
        state.reward_repo(),
        state.points_ledger_repo(),
        state.reward_redemption_repo(),
    );
    let redemption = use_case
        .execute(
            LoyaltyMemberId::from_uuid(id),
            Some(*ctx.user_id().as_uuid()),
            cmd,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(RewardRedemptionResponse::from(&redemption)))
}
