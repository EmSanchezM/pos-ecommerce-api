// Accounting routes: chart of accounts, periods, journal entries, reports.

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::accounting::{
    close_period_handler, create_account_handler, get_journal_entry_handler, list_accounts_handler,
    list_journal_entries_handler, list_periods_handler, open_period_handler,
    post_journal_entry_handler, profit_and_loss_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn accounting_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/accounts",
            get(list_accounts_handler).post(create_account_handler),
        )
        .route(
            "/periods",
            get(list_periods_handler).post(open_period_handler),
        )
        .route("/periods/{id}/close", post(close_period_handler))
        .route(
            "/journal-entries",
            get(list_journal_entries_handler).post(post_journal_entry_handler),
        )
        .route("/journal-entries/{id}", get(get_journal_entry_handler))
        .route("/reports/profit-and-loss", get(profit_and_loss_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
