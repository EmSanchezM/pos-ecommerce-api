// Invoice handlers for the Fiscal module

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use fiscal::{
    CalculateTaxCommand, GenerateInvoiceCommand, InvoiceResponse, ListInvoicesQuery,
    TaxCalculationResponse, VoidInvoiceCommand,
};

pub async fn generate_invoice_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<GenerateInvoiceCommand>,
) -> Result<(StatusCode, Json<InvoiceResponse>), Response> {
    require_permission(&ctx, "invoices:create")?;

    let use_case = fiscal::GenerateInvoiceUseCase::new(
        state.invoice_repo(),
        state.fiscal_sequence_repo(),
        state.tax_rate_repo(),
        state.sale_repo(),
        state.terminal_repo(),
    );

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_invoice_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceResponse>, Response> {
    require_permission(&ctx, "invoices:read")?;

    let use_case = fiscal::GetInvoiceUseCase::new(state.invoice_repo());

    let response = use_case
        .execute(id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn list_invoices_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(query): Query<ListInvoicesQuery>,
) -> Result<Json<fiscal::InvoiceListResponse>, Response> {
    require_permission(&ctx, "invoices:read")?;

    let use_case = fiscal::ListInvoicesUseCase::new(state.invoice_repo());

    let response = use_case
        .execute(query)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn void_invoice_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    JsonBody(mut command): JsonBody<VoidInvoiceCommand>,
) -> Result<Json<InvoiceResponse>, Response> {
    require_permission(&ctx, "invoices:void")?;

    command.invoice_id = id;

    let use_case = fiscal::VoidInvoiceUseCase::new(state.invoice_repo());

    let response = use_case
        .execute(command, *ctx.user_id())
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}

pub async fn calculate_tax_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    JsonBody(command): JsonBody<CalculateTaxCommand>,
) -> Result<Json<TaxCalculationResponse>, Response> {
    require_permission(&ctx, "invoices:read")?;

    let use_case = fiscal::CalculateTaxUseCase::new(state.tax_rate_repo());

    let response = use_case
        .execute(command)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    Ok(Json(response))
}
