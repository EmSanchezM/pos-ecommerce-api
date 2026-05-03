//! Reward catalog endpoints (per program).

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use loyalty::{
    CreateRewardCommand, CreateRewardUseCase, ListRewardsUseCase, LoyaltyProgramId, RewardResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListRewardsQuery {
    pub program_id: Uuid,
}

pub async fn list_rewards_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListRewardsQuery>,
) -> Result<Json<Vec<RewardResponse>>, Response> {
    require_permission(&ctx, "loyalty:read_reward")?;
    let use_case = ListRewardsUseCase::new(state.reward_repo());
    let rewards = use_case
        .execute(LoyaltyProgramId::from_uuid(params.program_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(rewards.iter().map(RewardResponse::from).collect()))
}

pub async fn create_reward_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateRewardCommand>,
) -> Result<Json<RewardResponse>, Response> {
    require_permission(&ctx, "loyalty:write_reward")?;
    let use_case = CreateRewardUseCase::new(state.loyalty_program_repo(), state.reward_repo());
    let reward = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(RewardResponse::from(&reward)))
}
