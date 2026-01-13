use axum::{routing::get, Router};
use common::health::infrastructure::health_check_simple;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health_check_simple));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API Gateway running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
