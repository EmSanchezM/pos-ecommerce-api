// Cash management routes: bank accounts, bank transactions, cash deposits,
// reconciliations.

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::cash_management::{
    close_reconciliation_handler, create_bank_account_handler, create_cash_deposit_handler,
    deactivate_bank_account_handler, get_bank_account_handler, link_deposit_handler,
    list_bank_accounts_handler, list_bank_transactions_handler, list_cash_deposits_handler,
    list_reconciliations_handler, mark_deposit_sent_handler, record_bank_transaction_handler,
    start_reconciliation_handler, update_bank_account_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn bank_accounts_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_bank_accounts_handler).post(create_bank_account_handler),
        )
        .route(
            "/{id}",
            get(get_bank_account_handler)
                .put(update_bank_account_handler)
                .delete(deactivate_bank_account_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn bank_transactions_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_bank_transactions_handler).post(record_bank_transaction_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn cash_deposits_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_cash_deposits_handler).post(create_cash_deposit_handler),
        )
        .route("/{id}/send", post(mark_deposit_sent_handler))
        .route("/{id}/link", post(link_deposit_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn bank_reconciliations_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_reconciliations_handler).post(start_reconciliation_handler),
        )
        .route("/{id}/close", post(close_reconciliation_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
