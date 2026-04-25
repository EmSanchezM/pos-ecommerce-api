// Fiscal routes for the API Gateway
//
// Invoices: /api/v1/invoices
// Tax Rates: /api/v1/tax-rates

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::handlers::fiscal::{
    calculate_tax_handler, create_tax_rate_handler, delete_tax_rate_handler,
    generate_invoice_handler, get_invoice_handler, get_tax_rate_handler, list_invoices_handler,
    list_tax_rates_handler, update_tax_rate_handler, void_invoice_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Creates the invoices router.
///
/// # Routes
/// - `POST /` - Generate invoice
/// - `GET /` - List invoices
/// - `GET /{id}` - Get invoice
/// - `PUT /{id}/void` - Void invoice
/// - `POST /calculate-tax` - Calculate tax
pub fn invoices_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(generate_invoice_handler).get(list_invoices_handler),
        )
        .route("/{id}", get(get_invoice_handler))
        .route("/{id}/void", put(void_invoice_handler))
        .route("/calculate-tax", post(calculate_tax_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the tax rates router.
///
/// # Routes
/// - `POST /` - Create tax rate
/// - `GET /store/{store_id}` - List tax rates for store
/// - `GET /{id}` - Get tax rate
/// - `PUT /{id}` - Update tax rate
/// - `DELETE /{id}` - Delete tax rate
pub fn tax_rates_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_tax_rate_handler))
        .route("/store/{store_id}", get(list_tax_rates_handler))
        .route(
            "/{id}",
            get(get_tax_rate_handler)
                .put(update_tax_rate_handler)
                .delete(delete_tax_rate_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
