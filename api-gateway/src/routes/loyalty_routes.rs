// Loyalty routes: programs, tiers, members (+ledger/earn/adjust/redeem),
// rewards.

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::loyalty::{
    adjust_points_handler, create_program_handler, create_reward_handler, create_tier_handler,
    earn_points_handler, enroll_member_handler, get_member_handler, get_member_ledger_handler,
    list_members_handler, list_programs_handler, list_rewards_handler, list_tiers_handler,
    redeem_reward_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn loyalty_programs_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_programs_handler).post(create_program_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn loyalty_tiers_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_tiers_handler).post(create_tier_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn loyalty_rewards_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_rewards_handler).post(create_reward_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn loyalty_members_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_members_handler).post(enroll_member_handler))
        .route("/{id}", get(get_member_handler))
        .route("/{id}/ledger", get(get_member_ledger_handler))
        .route("/{id}/earn", post(earn_points_handler))
        .route("/{id}/adjust", post(adjust_points_handler))
        .route("/{id}/redeem", post(redeem_reward_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
