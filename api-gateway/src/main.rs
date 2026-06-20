use std::net::SocketAddr;

mod audit;
mod bootstrap;
mod config;
mod cors;
mod database;
pub mod error;
pub mod extractors;
mod handlers;
mod jobs;
pub mod middleware;
mod router;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let (addr, app) = bootstrap::build().await;

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("API Gateway running on http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
