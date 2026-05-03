//! Service order intake / list / get-detail endpoints.

use std::str::FromStr;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use service_orders::{
    AssetId, DiagnosticResponse, GetServiceOrderUseCase, IntakeServiceOrderCommand,
    IntakeServiceOrderUseCase, ListDiagnosticsUseCase, ListItemsUseCase, ListQuotesUseCase,
    ListServiceOrdersFilters, ListServiceOrdersUseCase, QuoteResponse, ServiceOrderDetailResponse,
    ServiceOrderId, ServiceOrderItemResponse, ServiceOrderResponse, ServiceOrderStatus,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::org_scope::{require_feature, verify_store_in_org};
use crate::middleware::permission::require_permission;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListServiceOrdersQuery {
    pub store_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

pub async fn list_service_orders_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListServiceOrdersQuery>,
) -> Result<Json<Vec<ServiceOrderResponse>>, Response> {
    require_permission(&ctx, "service_orders:read_order")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    if let Some(sid) = params.store_id {
        verify_store_in_org(state.pool(), &ctx, sid).await?;
    }

    let status = match params.status.as_deref() {
        Some(s) => {
            Some(ServiceOrderStatus::from_str(s).map_err(|e| AppError::from(e).into_response())?)
        }
        None => None,
    };

    let filters = ListServiceOrdersFilters {
        store_id: params.store_id,
        customer_id: params.customer_id,
        asset_id: params.asset_id.map(AssetId::from_uuid),
        status,
        from: params.from,
        to: params.to,
        limit: params.limit,
    };

    let use_case = ListServiceOrdersUseCase::new(state.service_order_repo());
    let orders = use_case
        .execute(filters)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(
        orders.iter().map(ServiceOrderResponse::from).collect(),
    ))
}

pub async fn intake_service_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<IntakeServiceOrderCommand>,
) -> Result<Json<ServiceOrderResponse>, Response> {
    require_permission(&ctx, "service_orders:write_order")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    verify_store_in_org(state.pool(), &ctx, cmd.store_id).await?;
    let use_case =
        IntakeServiceOrderUseCase::new(state.service_asset_repo(), state.service_order_repo());
    let order = use_case
        .execute(cmd, Some(*ctx.user_id().as_uuid()))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceOrderResponse::from(&order)))
}

pub async fn get_service_order_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ServiceOrderDetailResponse>, Response> {
    require_permission(&ctx, "service_orders:read_order")?;
    require_feature(state.pool(), &ctx, "service_orders").await?;
    let order_id = ServiceOrderId::from_uuid(id);
    let order = GetServiceOrderUseCase::new(state.service_order_repo())
        .execute(order_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    let items = ListItemsUseCase::new(state.service_order_item_repo())
        .execute(order_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    let diagnostics = ListDiagnosticsUseCase::new(state.service_diagnostic_repo())
        .execute(order_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    let quotes = ListQuotesUseCase::new(state.service_quote_repo())
        .execute(order_id)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(ServiceOrderDetailResponse {
        order: ServiceOrderResponse::from(&order),
        items: items.iter().map(ServiceOrderItemResponse::from).collect(),
        diagnostics: diagnostics.iter().map(DiagnosticResponse::from).collect(),
        quotes: quotes.iter().map(QuoteResponse::from).collect(),
    }))
}
