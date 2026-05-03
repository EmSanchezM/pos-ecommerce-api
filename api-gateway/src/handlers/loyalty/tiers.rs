//! Member tier endpoints (per program).

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use loyalty::{
    CreateMemberTierCommand, CreateMemberTierUseCase, ListMemberTiersUseCase, LoyaltyProgramId,
    MemberTierResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTiersQuery {
    pub program_id: Uuid,
}

pub async fn list_tiers_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListTiersQuery>,
) -> Result<Json<Vec<MemberTierResponse>>, Response> {
    require_permission(&ctx, "loyalty:read_tier")?;
    let use_case = ListMemberTiersUseCase::new(state.member_tier_repo());
    let tiers = use_case
        .execute(LoyaltyProgramId::from_uuid(params.program_id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(tiers.iter().map(MemberTierResponse::from).collect()))
}

pub async fn create_tier_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateMemberTierCommand>,
) -> Result<Json<MemberTierResponse>, Response> {
    require_permission(&ctx, "loyalty:write_tier")?;
    let use_case =
        CreateMemberTierUseCase::new(state.loyalty_program_repo(), state.member_tier_repo());
    let tier = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(MemberTierResponse::from(&tier)))
}
