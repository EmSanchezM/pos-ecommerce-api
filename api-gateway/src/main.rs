use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use axum::{Router, routing::get};
use common::health::infrastructure::health_check_simple;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub mod error;
pub mod extractors;
mod handlers;
mod jobs;
pub mod middleware;
mod routes;
mod state;

use routes::{
    auth_router, cart_router, categories_router, credit_notes_router, customers_router,
    goods_receipts_router, inventory_router, pos_sales_router, products_router,
    purchase_orders_router, recipes_router, reports_router, shifts_router, store_router,
    store_terminals_router, terminals_router, transfers_router, vendors_router,
};
use state::AppState;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Get JWT secret from environment (required)
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET environment variable must be set");

    // Create PostgreSQL connection pool with configurable settings
    let max_connections = env_or::<u32>("DB_MAX_CONNECTIONS", 50);
    let min_connections = env_or::<u32>("DB_MIN_CONNECTIONS", 5);
    let acquire_timeout_secs = env_or::<u64>("DB_ACQUIRE_TIMEOUT_SECS", 5);
    let idle_timeout_secs = env_or::<u64>("DB_IDLE_TIMEOUT_SECS", 300);
    let max_lifetime_secs = env_or::<u64>("DB_MAX_LIFETIME_SECS", 1800);

    let pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(min_connections)
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(idle_timeout_secs))
        .max_lifetime(Duration::from_secs(max_lifetime_secs))
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Create application state with repositories and services
    let app_state = AppState::from_pool(pool, jwt_secret);

    // Configure CORS
    let cors_layer = build_cors_layer();

    // Build the application router
    let app = Router::new()
        .route("/health", get(health_check_simple))
        .nest("/api/v1/auth", auth_router())
        .nest("/api/v1/stores", store_router(app_state.clone()))
        .nest(
            "/api/v1/stores/{store_id}/terminals",
            store_terminals_router(app_state.clone()),
        )
        .nest("/api/v1/terminals", terminals_router(app_state.clone()))
        .nest("/api/v1/transfers", transfers_router(app_state.clone()))
        .nest("/api/v1/categories", categories_router(app_state.clone()))
        .nest("/api/v1/products", products_router(app_state.clone()))
        .nest("/api/v1/recipes", recipes_router(app_state.clone()))
        .nest("/api/v1/inventory", inventory_router(app_state.clone()))
        .nest("/api/v1/reports", reports_router(app_state.clone()))
        .nest("/api/v1/vendors", vendors_router(app_state.clone()))
        .nest(
            "/api/v1/purchase-orders",
            purchase_orders_router(app_state.clone()),
        )
        .nest(
            "/api/v1/goods-receipts",
            goods_receipts_router(app_state.clone()),
        )
        .nest("/api/v1/customers", customers_router(app_state.clone()))
        .nest("/api/v1/shifts", shifts_router(app_state.clone()))
        .nest("/api/v1/sales", pos_sales_router(app_state.clone()))
        .nest("/api/v1/carts", cart_router(app_state.clone()))
        .nest(
            "/api/v1/credit-notes",
            credit_notes_router(app_state.clone()),
        )
        .layer(cors_layer)
        .with_state(app_state.clone());

    // Start background jobs
    let reservation_expiry_interval = env_or::<u64>("RESERVATION_EXPIRY_INTERVAL_SECS", 300);
    let cart_cleanup_interval = env_or::<u64>("CART_CLEANUP_INTERVAL_SECS", 900);

    jobs::reservation_expiry::spawn(
        app_state.reservation_repo(),
        app_state.stock_repo(),
        reservation_expiry_interval,
    );
    jobs::cart_cleanup::spawn(app_state.cart_repo(), cart_cleanup_interval);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("API Gateway running on http://0.0.0.0:8000");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/// Reads an environment variable, parsing it to `T`. Returns `default` if the
/// variable is missing or cannot be parsed.
fn env_or<T: FromStr>(name: &str, default: T) -> T {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Builds a CORS layer from the `CORS_ALLOWED_ORIGINS` environment variable.
///
/// If the variable is set, it is parsed as a comma-separated list of origins.
/// If unset, all origins are allowed (development mode).
fn build_cors_layer() -> CorsLayer {
    use axum::http::{HeaderName, Method};

    let methods = AllowMethods::from(vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ]);

    let headers = AllowHeaders::from(vec![
        HeaderName::from_static("authorization"),
        HeaderName::from_static("content-type"),
        HeaderName::from_static("x-store-id"),
    ]);

    let origins = match env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins_str) if !origins_str.is_empty() => {
            let parsed: Vec<_> = origins_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            AllowOrigin::list(parsed)
        }
        _ => AllowOrigin::any(),
    };

    CorsLayer::new()
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_origin(origins)
}
