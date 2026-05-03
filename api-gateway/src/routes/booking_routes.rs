// Booking routes: resources, services, appointments, policies, plus the
// PUBLIC booking router (no auth, mirrors `public_tracking_router` in
// `shipping_routes.rs:124`).
//
// /api/v1/booking/resources                     - resource CRUD + calendar
// /api/v1/booking/services                      - bookable service CRUD + M2M
// /api/v1/booking/appointments                  - admin/staff lifecycle
// /api/v1/booking/policies                      - per-store policy
// /api/v1/public/booking/{store_id}/services    - PUBLIC list services
// /api/v1/public/booking/{store_id}/availability- PUBLIC slot lookup
// /api/v1/public/booking/{store_id}/book        - PUBLIC create appointment
// /api/v1/public/booking/appointments/{id}      - PUBLIC view (token-guarded)

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::handlers::booking::{
    assign_service_resources_handler, cancel_appointment_handler, complete_appointment_handler,
    confirm_appointment_handler, create_appointment_handler, create_resource_handler,
    create_service_handler, deactivate_resource_handler, deactivate_service_handler,
    get_appointment_handler, get_booking_policy_handler, get_public_appointment_handler,
    get_resource_calendar_handler, list_appointments_handler, list_public_services_handler,
    list_resources_handler, list_services_handler, no_show_appointment_handler,
    public_availability_handler, public_book_handler, set_resource_calendar_handler,
    start_appointment_handler, update_resource_handler, update_service_handler,
    upsert_booking_policy_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn booking_resources_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_resources_handler).post(create_resource_handler),
        )
        .route(
            "/{id}",
            put(update_resource_handler).delete(deactivate_resource_handler),
        )
        .route(
            "/{id}/calendar",
            get(get_resource_calendar_handler).put(set_resource_calendar_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn booking_services_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_services_handler).post(create_service_handler))
        .route(
            "/{id}",
            put(update_service_handler).delete(deactivate_service_handler),
        )
        .route("/{id}/resources", put(assign_service_resources_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn booking_appointments_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_appointments_handler).post(create_appointment_handler),
        )
        .route("/{id}", get(get_appointment_handler))
        .route("/{id}/confirm", post(confirm_appointment_handler))
        .route("/{id}/start", post(start_appointment_handler))
        .route("/{id}/complete", post(complete_appointment_handler))
        .route("/{id}/cancel", post(cancel_appointment_handler))
        .route("/{id}/no-show", post(no_show_appointment_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn booking_policies_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(get_booking_policy_handler).put(upsert_booking_policy_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// PUBLIC: no auth. Mounted under `/api/v1/public/booking`. Mirrors
/// `public_tracking_router` in `shipping_routes.rs`.
pub fn public_booking_router() -> Router<AppState> {
    Router::new()
        .route("/{store_id}/services", get(list_public_services_handler))
        .route("/{store_id}/availability", get(public_availability_handler))
        .route("/{store_id}/book", post(public_book_handler))
        .route("/appointments/{id}", get(get_public_appointment_handler))
}
