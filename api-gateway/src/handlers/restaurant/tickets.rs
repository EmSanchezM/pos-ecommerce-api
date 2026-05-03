//! KDS ticket endpoints — all the use cases that drive the kitchen workflow.

use std::str::FromStr;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use restaurant_operations::{
    CancelKdsTicketCommand, CancelKdsTicketUseCase, CreateKdsTicketCommand, CreateKdsTicketUseCase,
    GetKdsTicketUseCase, KdsDeps, KdsTicketDetailResponse, KdsTicketId, KdsTicketItemId,
    KdsTicketItemResponse, KdsTicketResponse, KdsTicketStatus, KitchenStationId,
    ListKdsTicketsFilters, ListKdsTicketsUseCase, MarkKdsTicketReadyUseCase, RestaurantTableId,
    SendKdsTicketUseCase, ServeKdsTicketUseCase, SetItemStatusCommand, SetItemStatusUseCase,
};

use crate::error::AppError;
use crate::extractors::CurrentUser;
use crate::middleware::permission::require_permission;
use crate::state::AppState;

fn deps(state: &AppState) -> KdsDeps {
    KdsDeps {
        stations: state.kitchen_station_repo(),
        tables: state.restaurant_table_repo(),
        modifiers: state.menu_modifier_repo(),
        tickets: state.kds_ticket_repo(),
        items: state.kds_ticket_item_repo(),
        broadcaster: state.kds_broadcaster(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListKdsTicketsQuery {
    pub store_id: Option<Uuid>,
    pub station_id: Option<Uuid>,
    pub table_id: Option<Uuid>,
    pub status: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

pub async fn list_tickets_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Query(params): Query<ListKdsTicketsQuery>,
) -> Result<Json<Vec<KdsTicketResponse>>, Response> {
    require_permission(&ctx, "restaurant:read_ticket")?;
    let status = match params.status.as_deref() {
        Some(s) => {
            Some(KdsTicketStatus::from_str(s).map_err(|e| AppError::from(e).into_response())?)
        }
        None => None,
    };
    let filters = ListKdsTicketsFilters {
        store_id: params.store_id,
        station_id: params.station_id.map(KitchenStationId::from_uuid),
        table_id: params.table_id.map(RestaurantTableId::from_uuid),
        status,
        from: params.from,
        to: params.to,
        limit: params.limit,
    };
    let use_case = ListKdsTicketsUseCase::new(state.kds_ticket_repo());
    let tickets = use_case
        .execute(filters)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(tickets.iter().map(KdsTicketResponse::from).collect()))
}

pub async fn create_ticket_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Json(cmd): Json<CreateKdsTicketCommand>,
) -> Result<Json<KdsTicketDetailResponse>, Response> {
    require_permission(&ctx, "restaurant:write_ticket")?;
    let use_case = CreateKdsTicketUseCase::new(deps(&state));
    let (ticket, items) = use_case
        .execute(cmd, Some(*ctx.user_id().as_uuid()))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketDetailResponse {
        ticket: KdsTicketResponse::from(&ticket),
        items: items.iter().map(KdsTicketItemResponse::from).collect(),
    }))
}

pub async fn get_ticket_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<KdsTicketDetailResponse>, Response> {
    require_permission(&ctx, "restaurant:read_ticket")?;
    let use_case = GetKdsTicketUseCase::new(state.kds_ticket_repo(), state.kds_ticket_item_repo());
    let (ticket, items) = use_case
        .execute(KdsTicketId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketDetailResponse {
        ticket: KdsTicketResponse::from(&ticket),
        items: items.iter().map(KdsTicketItemResponse::from).collect(),
    }))
}

pub async fn send_ticket_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<KdsTicketResponse>, Response> {
    require_permission(&ctx, "restaurant:transition_ticket")?;
    let use_case = SendKdsTicketUseCase::new(deps(&state));
    let ticket = use_case
        .execute(KdsTicketId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketResponse::from(&ticket)))
}

pub async fn mark_ticket_ready_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<KdsTicketResponse>, Response> {
    require_permission(&ctx, "restaurant:transition_ticket")?;
    let use_case = MarkKdsTicketReadyUseCase::new(deps(&state));
    let ticket = use_case
        .execute(KdsTicketId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketResponse::from(&ticket)))
}

pub async fn serve_ticket_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<KdsTicketResponse>, Response> {
    require_permission(&ctx, "restaurant:transition_ticket")?;
    let use_case = ServeKdsTicketUseCase::new(deps(&state));
    let ticket = use_case
        .execute(KdsTicketId::from_uuid(id))
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketResponse::from(&ticket)))
}

pub async fn cancel_ticket_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<CancelKdsTicketCommand>,
) -> Result<Json<KdsTicketResponse>, Response> {
    require_permission(&ctx, "restaurant:cancel_ticket")?;
    let use_case = CancelKdsTicketUseCase::new(deps(&state));
    let ticket = use_case
        .execute(KdsTicketId::from_uuid(id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketResponse::from(&ticket)))
}

pub async fn set_item_status_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path((_ticket_id, item_id)): Path<(Uuid, Uuid)>,
    Json(cmd): Json<SetItemStatusCommand>,
) -> Result<Json<KdsTicketItemResponse>, Response> {
    require_permission(&ctx, "restaurant:transition_ticket")?;
    let use_case = SetItemStatusUseCase::new(deps(&state));
    let item = use_case
        .execute(KdsTicketItemId::from_uuid(item_id), cmd)
        .await
        .map_err(|e| AppError::from(e).into_response())?;
    Ok(Json(KdsTicketItemResponse::from(&item)))
}
