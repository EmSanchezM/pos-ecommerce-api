// Sales routes for the API Gateway
//
// This module defines the sales routers that group all sales-related endpoints
// with authentication middleware.
//
// Customers: /api/v1/customers
// Shifts: /api/v1/shifts
// POS Sales: /api/v1/sales

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::{
    activate_customer_handler, add_cart_item_handler, add_credit_note_item_handler,
    add_sale_item_handler, apply_credit_note_handler, apply_discount_handler,
    approve_credit_note_handler, cancel_credit_note_handler, cash_in_handler,
    cash_out_handler, clear_cart_handler, close_shift_handler, complete_sale_handler,
    create_cart_handler, create_credit_note_handler, create_customer_handler,
    create_pos_sale_handler, deactivate_customer_handler, get_cart_handler,
    get_credit_note_handler, get_current_shift_handler, get_customer_handler,
    get_sale_handler, get_shift_report_handler, list_credit_notes_handler,
    list_customers_handler, list_sales_handler, list_shifts_handler, open_shift_handler,
    process_payment_handler, remove_cart_item_handler, remove_credit_note_item_handler,
    remove_sale_item_handler, submit_credit_note_handler, update_cart_item_handler,
    update_customer_handler, update_sale_item_handler, void_sale_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Creates the customers router.
///
/// # Routes
/// - `POST /` - Create customer
/// - `GET /` - List customers
/// - `GET /{id}` - Get customer
/// - `PUT /{id}` - Update customer
/// - `PUT /{id}/activate` - Activate customer
/// - `PUT /{id}/deactivate` - Deactivate customer
pub fn customers_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_customer_handler).get(list_customers_handler))
        .route("/{id}", get(get_customer_handler).put(update_customer_handler))
        .route("/{id}/activate", put(activate_customer_handler))
        .route("/{id}/deactivate", put(deactivate_customer_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the shifts router.
///
/// # Routes
/// - `POST /` - Open shift
/// - `GET /` - List shifts
/// - `GET /current/{terminal_id}` - Get current open shift for terminal
/// - `GET /{id}/report` - Get shift report
/// - `PUT /{id}/close` - Close shift
/// - `POST /{id}/cash-in` - Record cash in
/// - `POST /{id}/cash-out` - Record cash out
pub fn shifts_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(open_shift_handler).get(list_shifts_handler))
        .route("/current/{terminal_id}", get(get_current_shift_handler))
        .route("/{id}/report", get(get_shift_report_handler))
        .route("/{id}/close", put(close_shift_handler))
        .route("/{id}/cash-in", post(cash_in_handler))
        .route("/{id}/cash-out", post(cash_out_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the POS sales router.
///
/// # Routes
/// - `POST /` - Create POS sale
/// - `GET /` - List sales
/// - `GET /{id}` - Get sale details
/// - `POST /{id}/items` - Add item to sale
/// - `PUT /{id}/items/{item_id}` - Update sale item
/// - `DELETE /{id}/items/{item_id}` - Remove sale item
/// - `POST /{id}/discount` - Apply discount
/// - `POST /{id}/payment` - Process payment
/// - `PUT /{id}/complete` - Complete sale
/// - `PUT /{id}/void` - Void sale
pub fn pos_sales_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_pos_sale_handler).get(list_sales_handler))
        .route("/{id}", get(get_sale_handler))
        .route("/{id}/items", post(add_sale_item_handler))
        .route("/{id}/items/{item_id}", put(update_sale_item_handler).delete(remove_sale_item_handler))
        .route("/{id}/discount", post(apply_discount_handler))
        .route("/{id}/payment", post(process_payment_handler))
        .route("/{id}/complete", put(complete_sale_handler))
        .route("/{id}/void", put(void_sale_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the e-commerce carts router.
///
/// # Routes
/// - `POST /` - Create cart
/// - `GET /{id}` - Get cart
/// - `POST /{id}/items` - Add item to cart
/// - `PUT /{id}/items/{item_id}` - Update cart item
/// - `DELETE /{id}/items/{item_id}` - Remove cart item
/// - `DELETE /{id}/items` - Clear cart
pub fn cart_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_cart_handler))
        .route("/{id}", get(get_cart_handler))
        .route("/{id}/items", post(add_cart_item_handler).delete(clear_cart_handler))
        .route("/{id}/items/{item_id}", put(update_cart_item_handler).delete(remove_cart_item_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// Creates the credit notes router.
///
/// # Routes
/// - `POST /` - Create credit note
/// - `GET /` - List credit notes
/// - `GET /{id}` - Get credit note
/// - `POST /{id}/items` - Add item to credit note
/// - `DELETE /{id}/items/{item_id}` - Remove item from credit note
/// - `PUT /{id}/submit` - Submit credit note for approval
/// - `PUT /{id}/approve` - Approve credit note
/// - `PUT /{id}/apply` - Apply credit note (process refund)
/// - `PUT /{id}/cancel` - Cancel credit note
pub fn credit_notes_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_credit_note_handler).get(list_credit_notes_handler))
        .route("/{id}", get(get_credit_note_handler))
        .route("/{id}/items", post(add_credit_note_item_handler))
        .route("/{id}/items/{item_id}", delete(remove_credit_note_item_handler))
        .route("/{id}/submit", put(submit_credit_note_handler))
        .route("/{id}/approve", put(approve_credit_note_handler))
        .route("/{id}/apply", put(apply_credit_note_handler))
        .route("/{id}/cancel", put(cancel_credit_note_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
