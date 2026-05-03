// Restaurant operations routes: stations, tables, modifiers, KDS tickets,
// plus the SSE stream for kitchen displays.
//
// /api/v1/restaurant/stations                   - station CRUD
// /api/v1/restaurant/tables                     - table CRUD + status
// /api/v1/restaurant/modifier-groups            - modifier groups + modifiers
// /api/v1/restaurant/products/{id}/...          - product M2M for modifiers
// /api/v1/restaurant/kds/tickets                - KDS ticket lifecycle
// /api/v1/restaurant/kds/stations/{id}/stream   - SSE stream of events

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::handlers::restaurant::{
    add_modifier_handler, assign_product_groups_handler, cancel_ticket_handler,
    create_modifier_group_handler, create_station_handler, create_table_handler,
    create_ticket_handler, deactivate_station_handler, deactivate_table_handler,
    get_product_groups_handler, get_ticket_handler, kds_stream_handler,
    list_modifier_groups_handler, list_stations_handler, list_tables_handler, list_tickets_handler,
    mark_ticket_ready_handler, send_ticket_handler, serve_ticket_handler, set_item_status_handler,
    set_table_status_handler, update_modifier_group_handler, update_modifier_handler,
    update_station_handler, update_table_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn restaurant_stations_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_stations_handler).post(create_station_handler))
        .route(
            "/{id}",
            put(update_station_handler).delete(deactivate_station_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn restaurant_tables_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_tables_handler).post(create_table_handler))
        .route(
            "/{id}",
            put(update_table_handler).delete(deactivate_table_handler),
        )
        .route("/{id}/status", post(set_table_status_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn restaurant_modifier_groups_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_modifier_groups_handler).post(create_modifier_group_handler),
        )
        .route("/{id}", put(update_modifier_group_handler))
        .route("/{id}/modifiers", post(add_modifier_handler))
        .route("/{id}/modifiers/{mid}", put(update_modifier_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn restaurant_product_modifiers_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/{product_id}/modifier-groups",
            get(get_product_groups_handler).put(assign_product_groups_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn kds_tickets_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_tickets_handler).post(create_ticket_handler))
        .route("/{id}", get(get_ticket_handler))
        .route("/{id}/send", post(send_ticket_handler))
        .route("/{id}/ready", post(mark_ticket_ready_handler))
        .route("/{id}/serve", post(serve_ticket_handler))
        .route("/{id}/cancel", post(cancel_ticket_handler))
        .route(
            "/{id}/items/{item_id}/status",
            post(set_item_status_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// SSE stream for kitchen displays. Auth is required on the initial
/// connection (the JWT goes in the `Authorization` header just like every
/// other authenticated endpoint).
pub fn kds_stream_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{station_id}/stream", get(kds_stream_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
