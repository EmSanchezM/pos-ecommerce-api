use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use restaurant_operations::{
    AddModifierUseCase, AssignProductModifierGroupsCommand, AssignProductModifierGroupsUseCase,
    CreateModifierCommand, CreateModifierGroupCommand, CreateModifierGroupUseCase,
    GetProductModifierGroupsUseCase, ListGroupsWithModifiersUseCase, MenuModifierGroupId,
    MenuModifierGroupResponse, MenuModifierId, MenuModifierResponse, UpdateModifierCommand,
    UpdateModifierGroupCommand, UpdateModifierGroupUseCase, UpdateModifierUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListModifierGroupsQuery {
    pub store_id: Uuid,
    pub include_inactive: Option<bool>,
}

pub async fn list_modifier_groups_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListModifierGroupsQuery>,
) -> Result<Json<Vec<MenuModifierGroupResponse>>, Response> {
    require_permission(&ctx, "restaurant:read_modifier")?;
    let only_active = !params.include_inactive.unwrap_or(false);
    let use_case = ListGroupsWithModifiersUseCase::new(state.menu_modifier_repo());
    let groups = use_case
        .execute(params.store_id, only_active)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        groups.iter().map(MenuModifierGroupResponse::from).collect(),
    ))
}

pub async fn create_modifier_group_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateModifierGroupCommand>,
) -> Result<Json<MenuModifierGroupResponse>, Response> {
    require_permission(&ctx, "restaurant:write_modifier")?;
    let use_case = CreateModifierGroupUseCase::new(state.menu_modifier_repo());
    let group = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(MenuModifierGroupResponse::from(&group)))
}

pub async fn update_modifier_group_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateModifierGroupCommand>,
) -> Result<Json<MenuModifierGroupResponse>, Response> {
    require_permission(&ctx, "restaurant:write_modifier")?;
    let use_case = UpdateModifierGroupUseCase::new(state.menu_modifier_repo());
    let group = use_case
        .execute(MenuModifierGroupId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(MenuModifierGroupResponse::from(&group)))
}

pub async fn add_modifier_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(group_id): Path<Uuid>,
    Json(cmd): Json<CreateModifierCommand>,
) -> Result<Json<MenuModifierResponse>, Response> {
    require_permission(&ctx, "restaurant:write_modifier")?;
    let use_case = AddModifierUseCase::new(state.menu_modifier_repo());
    let modifier = use_case
        .execute(MenuModifierGroupId::from_uuid(group_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(MenuModifierResponse::from(&modifier)))
}

pub async fn update_modifier_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_group_id, modifier_id)): Path<(Uuid, Uuid)>,
    Json(cmd): Json<UpdateModifierCommand>,
) -> Result<Json<MenuModifierResponse>, Response> {
    require_permission(&ctx, "restaurant:write_modifier")?;
    let use_case = UpdateModifierUseCase::new(state.menu_modifier_repo());
    let modifier = use_case
        .execute(MenuModifierId::from_uuid(modifier_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(MenuModifierResponse::from(&modifier)))
}

pub async fn assign_product_groups_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
    Json(cmd): Json<AssignProductModifierGroupsCommand>,
) -> Result<axum::http::StatusCode, Response> {
    require_permission(&ctx, "restaurant:write_modifier")?;
    let use_case = AssignProductModifierGroupsUseCase::new(state.menu_modifier_repo());
    use_case
        .execute(product_id, cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn get_product_groups_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(product_id): Path<Uuid>,
) -> Result<Json<Vec<MenuModifierGroupResponse>>, Response> {
    require_permission(&ctx, "restaurant:read_modifier")?;
    let use_case = GetProductModifierGroupsUseCase::new(state.menu_modifier_repo());
    let groups = use_case
        .execute(product_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        groups.iter().map(MenuModifierGroupResponse::from).collect(),
    ))
}
