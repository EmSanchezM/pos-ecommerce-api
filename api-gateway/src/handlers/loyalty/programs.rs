//! Loyalty program endpoints.

use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use loyalty::{
    CreateLoyaltyProgramCommand, CreateLoyaltyProgramUseCase, ListLoyaltyProgramsUseCase,
    LoyaltyProgramResponse,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::{require_feature, verify_store_in_org};
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListProgramsQuery {
    pub store_id: Option<Uuid>,
}

pub async fn list_programs_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListProgramsQuery>,
) -> Result<Json<Vec<LoyaltyProgramResponse>>, Response> {
    require_permission(&ctx, "loyalty:read_program")?;
    require_feature(state.pool(), &ctx, "loyalty").await?;
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }
    let use_case = ListLoyaltyProgramsUseCase::new(state.loyalty_program_repo());
    let programs = use_case
        .execute(params.store_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        programs.iter().map(LoyaltyProgramResponse::from).collect(),
    ))
}

pub async fn create_program_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateLoyaltyProgramCommand>,
) -> Result<Json<LoyaltyProgramResponse>, Response> {
    require_permission(&ctx, "loyalty:write_program")?;
    require_feature(state.pool(), &ctx, "loyalty").await?;
    verify_store_in_org(state.pool(), &ctx, cmd.store_id).await?;
    let use_case = CreateLoyaltyProgramUseCase::new(state.loyalty_program_repo());
    let program = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(LoyaltyProgramResponse::from(&program)))
}
