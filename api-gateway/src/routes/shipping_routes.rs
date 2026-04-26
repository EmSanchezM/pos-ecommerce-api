// Shipping routes.
//
// /api/v1/shipping-methods       - method catalog (CRUD)
// /api/v1/shipping-zones         - geographic zones (CRUD)
// /api/v1/shipping-rates         - rate matrix (CRUD)
// /api/v1/shipping/calculate     - quote shipping for a destination + order
// /api/v1/drivers                - driver pool (CRUD)
// /api/v1/delivery-providers     - super-admin gateway-style CUD
// /api/v1/shipments              - the actual fulfillment records
// /api/v1/track/{tracking}       - PUBLIC tracking (no auth)
// /api/v1/webhooks/delivery/{p}  - PUBLIC webhook (signature validated)

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::handlers::shipping::{
    assign_driver_handler, calculate_shipping_handler, cancel_shipment_handler,
    configure_delivery_provider_handler, confirm_pickup_handler, create_driver_handler,
    create_shipment_handler, create_shipping_method_handler, create_shipping_rate_handler,
    create_shipping_zone_handler, delete_delivery_provider_handler, delete_driver_handler,
    delete_shipping_method_handler, delete_shipping_rate_handler, delete_shipping_zone_handler,
    delivery_webhook_handler, dispatch_provider_handler, get_shipment_handler,
    list_delivery_providers_handler, list_drivers_handler, list_shipments_handler,
    list_shipping_methods_handler, list_shipping_zones_handler, mark_delivered_handler,
    mark_failed_handler, mark_ready_for_pickup_handler, public_tracking_handler,
    reschedule_shipment_handler, update_delivery_provider_handler, update_driver_handler,
    update_shipment_status_handler, update_shipping_method_handler, update_shipping_rate_handler,
    update_shipping_zone_handler, update_tracking_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn shipping_methods_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_shipping_method_handler).get(list_shipping_methods_handler),
        )
        .route(
            "/{id}",
            put(update_shipping_method_handler).delete(delete_shipping_method_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn shipping_zones_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_shipping_zone_handler).get(list_shipping_zones_handler),
        )
        .route(
            "/{id}",
            put(update_shipping_zone_handler).delete(delete_shipping_zone_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn shipping_rates_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_shipping_rate_handler))
        .route(
            "/{id}",
            put(update_shipping_rate_handler).delete(delete_shipping_rate_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn shipping_calculate_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(calculate_shipping_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn drivers_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_driver_handler).get(list_drivers_handler))
        .route(
            "/{id}",
            put(update_driver_handler).delete(delete_driver_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn delivery_providers_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(configure_delivery_provider_handler).get(list_delivery_providers_handler),
        )
        .route(
            "/{id}",
            put(update_delivery_provider_handler).delete(delete_delivery_provider_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn shipments_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_shipment_handler).get(list_shipments_handler),
        )
        .route("/{id}", get(get_shipment_handler))
        .route(
            "/{id}/ready-for-pickup",
            post(mark_ready_for_pickup_handler),
        )
        .route("/{id}/confirm-pickup", post(confirm_pickup_handler))
        .route("/{id}/assign-driver", post(assign_driver_handler))
        .route("/{id}/dispatch-provider", post(dispatch_provider_handler))
        .route("/{id}/status", put(update_shipment_status_handler))
        .route("/{id}/delivered", post(mark_delivered_handler))
        .route("/{id}/failed", post(mark_failed_handler))
        .route("/{id}/reschedule", post(reschedule_shipment_handler))
        .route("/{id}/cancel", post(cancel_shipment_handler))
        .route("/{id}/tracking", put(update_tracking_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// PUBLIC: no auth, just a tracking number.
pub fn public_tracking_router() -> Router<AppState> {
    Router::new().route("/{tracking_number}", get(public_tracking_handler))
}

/// PUBLIC: no auth, signature validated by adapter.
pub fn delivery_webhooks_router() -> Router<AppState> {
    Router::new().route("/{provider_type}", post(delivery_webhook_handler))
}
