// Customer handlers for the Sales module

use axum::{extract::{Path, Query, State}, http::StatusCode, Json, response::{IntoResponse, Response}};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use sales::{
    CreateCustomerCommand, CustomerListResponse, CustomerResponse, ListCustomersQuery,
    UpdateCustomerCommand,
};

pub async fn create_customer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreateCustomerCommand>,
) -> Result<(StatusCode, Json<CustomerResponse>), Response> {
    require_permission(&ctx, "sales:create_customer")?;

    let use_case = sales::CreateCustomerUseCase::new(state.customer_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_customer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, Response> {
    require_permission(&ctx, "sales:read_customer")?;

    let use_case = sales::GetCustomerUseCase::new(state.customer_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn list_customers_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListCustomersQuery>,
) -> Result<Json<CustomerListResponse>, Response> {
    require_permission(&ctx, "sales:read_customer")?;

    let use_case = sales::ListCustomersUseCase::new(state.customer_repo());

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn update_customer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(command): Json<UpdateCustomerCommand>,
) -> Result<Json<CustomerResponse>, Response> {
    require_permission(&ctx, "sales:update_customer")?;

    let use_case = sales::UpdateCustomerUseCase::new(state.customer_repo());

    let mut cmd = command;
    cmd.customer_id = id;

    let response = use_case
        .execute(cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn activate_customer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, Response> {
    require_permission(&ctx, "sales:update_customer")?;

    let use_case = sales::ToggleCustomerStatusUseCase::new(state.customer_repo());

    let response = use_case
        .activate(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn deactivate_customer_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, Response> {
    require_permission(&ctx, "sales:update_customer")?;

    let use_case = sales::ToggleCustomerStatusUseCase::new(state.customer_repo());

    let response = use_case
        .deactivate(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
