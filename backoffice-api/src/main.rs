// Backoffice API binary
//
// Listens on BACKOFFICE_PORT (default 8001).
// Serves backoffice operator endpoints with Backoffice-audience JWT auth.
//
// Follows the same split-by-SRP pattern as api-gateway (commit 129726c):
// - config.rs    — reads env vars
// - database.rs  — pool initialization
// - state.rs     — DI wiring
// - router.rs    — route assembly
// - middleware/  — auth + permission guards
// - handlers/    — request handlers
// - routes/      — route sub-modules

pub mod audit;
mod config;
mod database;
pub mod error;
mod handlers;
mod integration_tests;
pub mod jobs;
pub mod middleware;
mod router;
mod routes;
mod state;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let config = config::BackofficeConfig::from_env();
    let pool = database::init_pool(&config.database).await;
    let app_state = state::BackofficeAppState::from_pool(
        pool,
        config.backoffice_secret,
        config.backoffice_issuer,
    );

    let app = router::build_router(app_state.clone());

    // P4-T08: spawn event_dispatcher with BackofficeAuditSubscriber
    jobs::spawn_event_dispatcher(
        app_state.pool().clone(),
        10, // 10-second interval; make configurable in Phase 6
        50,
    );

    let addr: SocketAddr = format!("0.0.0.0:{}", config.port)
        .parse()
        .expect("invalid bind address");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    println!("Backoffice API running on http://{addr}");

    axum::serve(listener, app).await.unwrap();
}
