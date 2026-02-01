// POS (Point of Sale) handlers for the Sales module

use axum::{extract::{Path, Query, State}, http::StatusCode, Json, response::{IntoResponse, Response}};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use sales::{
    AddSaleItemCommand, ApplyDiscountCommand, CreatePosSaleCommand, ListSalesQuery,
    ProcessPaymentCommand, SaleDetailResponse, SaleListResponse, VoidSaleCommand,
};

/// Extended request for adding a sale item.
/// Includes product details that the POS terminal provides after product lookup.
#[derive(Debug, Deserialize)]
pub struct AddSaleItemRequest {
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub quantity: Decimal,
    pub unit_price: Option<Decimal>,
    pub sku: String,
    pub description: String,
    pub unit_cost: Decimal,
    pub tax_rate: Decimal,
    pub unit_of_measure: String,
    pub notes: Option<String>,
}

/// Request for updating a sale item (path provides sale_id)
#[derive(Debug, Deserialize)]
pub struct UpdateSaleItemRequest {
    pub quantity: Option<Decimal>,
    pub unit_price: Option<Decimal>,
    pub notes: Option<String>,
}

pub async fn create_pos_sale_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(command): Json<CreatePosSaleCommand>,
) -> Result<(StatusCode, Json<SaleDetailResponse>), Response> {
    require_permission(&ctx, "sales:create")?;

    let use_case = sales::CreatePosSaleUseCase::new(
        state.sale_repo(),
        state.shift_repo(),
    );

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn add_sale_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    Json(req): Json<AddSaleItemRequest>,
) -> Result<(StatusCode, Json<SaleDetailResponse>), Response> {
    require_permission(&ctx, "sales:create")?;

    let use_case = sales::AddSaleItemUseCase::new(state.sale_repo());

    let command = AddSaleItemCommand {
        sale_id,
        product_id: req.product_id,
        variant_id: req.variant_id,
        quantity: req.quantity,
        unit_price: req.unit_price,
        notes: req.notes,
    };

    let uom: inventory::UnitOfMeasure = req.unit_of_measure.parse()
        .map_err(|_| AppError::from(sales::SalesError::InvalidUnitOfMeasure).into_response())?;

    let response = use_case
        .execute(
            command,
            req.sku,
            req.description,
            req.unit_price.unwrap_or(req.unit_cost),
            req.unit_cost,
            req.tax_rate,
            uom,
        )
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_sale_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_sale_id, item_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateSaleItemRequest>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "sales:update")?;

    let use_case = sales::UpdateSaleItemUseCase::new(state.sale_repo());

    let command = sales::UpdateSaleItemCommand {
        item_id,
        quantity: req.quantity,
        unit_price: req.unit_price,
        notes: req.notes,
    };

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn remove_sale_item_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_sale_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "sales:update")?;

    let use_case = sales::RemoveSaleItemUseCase::new(state.sale_repo());

    let response = use_case
        .execute(item_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn apply_discount_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    Json(mut command): Json<ApplyDiscountCommand>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "sales:apply_discount")?;

    command.sale_id = sale_id;

    let use_case = sales::ApplyDiscountUseCase::new(state.sale_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn process_payment_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    Json(mut command): Json<ProcessPaymentCommand>,
) -> Result<(StatusCode, Json<SaleDetailResponse>), Response> {
    require_permission(&ctx, "sales:process_payment")?;

    command.sale_id = sale_id;

    let use_case = sales::ProcessPaymentUseCase::new(
        state.sale_repo(),
        state.shift_repo(),
    );

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn complete_sale_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "sales:complete")?;

    let invoice_number = body
        .get("invoice_number")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let use_case = sales::CompleteSaleUseCase::new(state.sale_repo());

    let response = use_case
        .execute(sale_id, invoice_number)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn void_sale_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    Json(command): Json<VoidSaleCommand>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "sales:void")?;

    let mut cmd = command;
    cmd.sale_id = sale_id;

    let use_case = sales::VoidSaleUseCase::new(state.sale_repo());

    let response = use_case
        .execute(cmd, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn get_sale_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleDetailResponse>, Response> {
    require_permission(&ctx, "sales:read")?;

    let use_case = sales::GetSaleUseCase::new(state.sale_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn list_sales_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListSalesQuery>,
) -> Result<Json<SaleListResponse>, Response> {
    require_permission(&ctx, "sales:read")?;

    let use_case = sales::ListSalesUseCase::new(state.sale_repo());

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
