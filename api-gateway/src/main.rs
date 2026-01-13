use axum::{routing::get, Router};
use common::health::infrastructure::health_check_simple;
use std::env;

pub mod error;
mod handlers;
mod routes;
mod state;

use routes::auth_router;
use state::AppState;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Get JWT secret from environment (or use a default for development)
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "development-secret-key-change-in-production".to_string());

    // Create PostgreSQL connection pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Create application state with repositories and services
    let app_state = AppState::from_pool(pool, jwt_secret);

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check_simple))
        .nest("/api/v1/auth", auth_router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("API Gateway running on http://0.0.0.0:8000");
    axum::serve(listener, app).await.unwrap();
}
