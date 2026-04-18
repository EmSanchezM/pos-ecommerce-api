// POS (Point of Sale) handlers for the Sales module

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::extractors::{CurrentUser, JsonBody};
use crate::middleware::permission::require_permission;
use crate::state::AppState;
use inventory::{
    Currency, InventoryMovement, InventoryMovementRepository, InventoryStockRepository,
    MovementType,
};
use sales::{
    AddSaleItemCommand, ApplyDiscountCommand, CreatePosSaleCommand, ListSalesQuery, Payment,
    PaymentMethod, PgSaleRepository, PgShiftRepository, ProcessPaymentCommand, SaleDetailResponse,
    SaleId, SaleListResponse, SaleRepository, ShiftRepository, VoidSaleCommand,
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
    JsonBody(command): JsonBody<CreatePosSaleCommand>,
) -> Result<(StatusCode, Json<SaleDetailResponse>), Response> {
    require_permission(&ctx, "sales:create")?;

    let use_case = sales::CreatePosSaleUseCase::new(state.sale_repo(), state.shift_repo());

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
    JsonBody(req): JsonBody<AddSaleItemRequest>,
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

    let uom: inventory::UnitOfMeasure = req
        .unit_of_measure
        .parse()
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
    JsonBody(req): JsonBody<UpdateSaleItemRequest>,
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

    // Idempotency check: if key provided, look for existing payment
    if let Some(ref key) = command.idempotency_key
        && let Some(existing_payment) = state
            .sale_repo()
            .find_payment_by_idempotency_key(key)
            .await
            .map_err(|e| AppError::from(e).into_response())?
    {
        let sale = state
            .sale_repo()
            .find_by_id_with_details(existing_payment.sale_id())
            .await
            .map_err(|e| AppError::from(e).into_response())?
            .ok_or_else(|| {
                AppError::from(sales::SalesError::SaleNotFound(
                    existing_payment.sale_id().into_uuid(),
                ))
                .into_response()
            })?;
        return Ok((StatusCode::CREATED, Json(SaleDetailResponse::from(sale))));
    }

    let sale_id_vo = SaleId::from_uuid(command.sale_id);
    let payment_method: PaymentMethod = command
        .payment_method
        .parse()
        .map_err(|_| AppError::from(sales::SalesError::InvalidPaymentMethod).into_response())?;

    let mut sale = state
        .sale_repo()
        .find_by_id_with_details(sale_id_vo)
        .await
        .map_err(|e| AppError::from(e).into_response())?
        .ok_or_else(|| {
            AppError::from(sales::SalesError::SaleNotFound(command.sale_id)).into_response()
        })?;

    if !sale.is_editable() {
        return Err(AppError::from(sales::SalesError::SaleNotEditable).into_response());
    }

    // Create the payment
    let mut payment = if payment_method == PaymentMethod::Cash {
        let tendered = command.amount_tendered.unwrap_or(command.amount);
        Payment::create_cash(
            sale_id_vo,
            command.amount,
            sale.currency().clone(),
            tendered,
        )
        .map_err(|e| AppError::from(e).into_response())?
    } else {
        Payment::create(
            sale_id_vo,
            payment_method,
            command.amount,
            sale.currency().clone(),
        )
        .map_err(|e| AppError::from(e).into_response())?
    };

    payment.set_reference_number(command.reference.clone());
    payment.set_notes(command.notes.clone());
    payment.set_idempotency_key(command.idempotency_key.clone());

    sale.add_payment(payment.clone())
        .map_err(|e| AppError::from(e).into_response())?;

    // Find shift before transaction to minimize TX duration
    let shift_update = if let Some(shift_id) = sale.shift_id() {
        state
            .shift_repo()
            .find_by_id(shift_id)
            .await
            .map_err(|e| AppError::from(e).into_response())?
            .map(|mut shift| {
                let result = match payment_method {
                    PaymentMethod::Cash => shift.record_cash_sale(command.amount),
                    PaymentMethod::CreditCard | PaymentMethod::DebitCard => {
                        shift.record_card_sale(command.amount)
                    }
                    _ => shift.record_other_sale(command.amount),
                };
                result.map(|()| shift)
            })
            .transpose()
            .map_err(|e| AppError::from(e).into_response())?
    } else {
        None
    };

    // All writes in a single transaction
    let mut tx = state
        .pool()
        .begin()
        .await
        .map_err(|e| AppError::from(sales::SalesError::from(e)).into_response())?;

    PgSaleRepository::save_payment_in_tx(&mut tx, &payment)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    PgSaleRepository::update_in_tx(&mut tx, &sale)
        .await
        .map_err(|e| AppError::from(e).into_response())?;

    if let Some(shift) = &shift_update {
        PgShiftRepository::update_in_tx(&mut tx, shift)
            .await
            .map_err(|e| AppError::from(e).into_response())?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::from(sales::SalesError::from(e)).into_response())?;

    Ok((StatusCode::CREATED, Json(SaleDetailResponse::from(sale))))
}

pub async fn complete_sale_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    JsonBody(body): JsonBody<serde_json::Value>,
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

    // Deduct inventory stock for each sale item
    let stock_repo = state.stock_repo();
    let movement_repo = state.movement_repo();
    let actor_id = *ctx.user_id();
    let store_id = identity::StoreId::from_uuid(response.store_id);

    for item in &response.items {
        let product_id = inventory::ProductId::from_uuid(item.product_id);
        let variant_id = item.variant_id.map(inventory::VariantId::from_uuid);

        // Find stock record
        let existing = if let Some(vid) = variant_id {
            stock_repo
                .find_by_store_and_variant(store_id, vid)
                .await
                .map_err(|e| AppError::from(e).into_response())?
        } else {
            stock_repo
                .find_by_store_and_product(store_id, product_id)
                .await
                .map_err(|e| AppError::from(e).into_response())?
        };

        if let Some(mut stock) = existing {
            let expected_version = stock.version();
            stock
                .adjust_quantity(-item.quantity)
                .map_err(|e| AppError::from(e).into_response())?;
            stock.increment_version();

            stock_repo
                .update_with_version(&stock, expected_version)
                .await
                .map_err(|e| AppError::from(e).into_response())?;

            // Create sale movement
            let movement = InventoryMovement::create(
                stock.id(),
                MovementType::Out,
                Some("Sale completed".to_string()),
                -item.quantity,
                Some(item.unit_price),
                Currency::hnl(),
                stock.quantity(),
                Some("sale".to_string()),
                Some(sale_id),
                actor_id,
                None,
            );
            movement_repo
                .save(&movement)
                .await
                .map_err(|e| AppError::from(e).into_response())?;
        }
        // If no stock record exists, skip (product may not be trackable)
    }

    Ok(Json(response))
}

pub async fn void_sale_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(sale_id): Path<Uuid>,
    JsonBody(command): JsonBody<VoidSaleCommand>,
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
