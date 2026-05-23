use std::net::SocketAddr;

use axum::Router;

use crate::config::AppConfig;
use crate::cors::build_cors_layer;
use crate::database::init_pool;
use crate::jobs;
use crate::router::build_router;
use crate::state::AppState;

pub async fn build() -> (SocketAddr, Router) {
    let config = AppConfig::from_env();
    let pool = init_pool(&config.database).await;
    let app_state = AppState::from_pool(pool, config.jwt_secret.clone());

    let app = build_router(app_state.clone(), &config).layer(build_cors_layer(&config));

    jobs::spawn_all(&app_state, &config.jobs);

    let addr: SocketAddr = "0.0.0.0:8000".parse().expect("invalid bind address");
    (addr, app)
}
