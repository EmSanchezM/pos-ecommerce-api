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
    abc_classification_router, accounting_router, analytics_router, auth_router,
    bank_accounts_router, bank_reconciliations_router, bank_transactions_router,
    booking_appointments_router, booking_policies_router, booking_resources_router,
    booking_services_router, cart_router, cash_deposits_router, catalog_images_router,
    catalog_listings_router, catalog_public_router, catalog_reviews_router,
    catalog_storage_providers_router, catalog_wishlist_router, categories_router,
    credit_notes_router, customers_router, delivery_providers_router, delivery_webhooks_router,
    drivers_router, forecasts_router, goods_receipts_router, inventory_router, invoices_router,
    kds_stream_router, kds_tickets_router, loyalty_members_router, loyalty_programs_router,
    loyalty_rewards_router, loyalty_tiers_router, orders_router, payment_gateways_router,
    payouts_router, pos_sales_router, products_router, promotions_router, public_booking_router,
    public_service_orders_router, public_tenancy_router, public_tracking_router,
    purchase_orders_router, recipes_router, reorder_policies_router,
    replenishment_suggestions_router, reports_router, restaurant_modifier_groups_router,
    restaurant_product_modifiers_router, restaurant_stations_router, restaurant_tables_router,
    service_orders_assets_router, service_orders_router, shifts_router, shipments_router,
    shipping_calculate_router, shipping_methods_router, shipping_rates_router,
    shipping_zones_router, store_router, store_terminals_router, tax_rates_router,
    tenancy_organizations_router, terminals_router, transactions_router, transfers_router,
    vendors_router, webhooks_router,
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
        .nest("/api/v1/orders", orders_router(app_state.clone()))
        .nest("/api/v1/promotions", promotions_router(app_state.clone()))
        .nest("/api/v1/sales", pos_sales_router(app_state.clone()))
        .nest("/api/v1/carts", cart_router(app_state.clone()))
        .nest(
            "/api/v1/credit-notes",
            credit_notes_router(app_state.clone()),
        )
        .nest("/api/v1/invoices", invoices_router(app_state.clone()))
        .nest("/api/v1/tax-rates", tax_rates_router(app_state.clone()))
        .nest(
            "/api/v1/payment-gateways",
            payment_gateways_router(app_state.clone()),
        )
        .nest(
            "/api/v1/transactions",
            transactions_router(app_state.clone()),
        )
        .nest("/api/v1/payouts", payouts_router(app_state.clone()))
        .nest("/api/v1/webhooks", webhooks_router())
        // Shipping
        .nest(
            "/api/v1/shipping-methods",
            shipping_methods_router(app_state.clone()),
        )
        .nest(
            "/api/v1/shipping-zones",
            shipping_zones_router(app_state.clone()),
        )
        .nest(
            "/api/v1/shipping-rates",
            shipping_rates_router(app_state.clone()),
        )
        .nest(
            "/api/v1/shipping/calculate",
            shipping_calculate_router(app_state.clone()),
        )
        .nest("/api/v1/drivers", drivers_router(app_state.clone()))
        .nest(
            "/api/v1/delivery-providers",
            delivery_providers_router(app_state.clone()),
        )
        .nest("/api/v1/shipments", shipments_router(app_state.clone()))
        .nest("/api/v1/track", public_tracking_router())
        .nest("/api/v1/webhooks/delivery", delivery_webhooks_router())
        // Catalog
        .nest(
            "/api/v1/catalog/listings",
            catalog_listings_router(app_state.clone()),
        )
        .nest(
            "/api/v1/catalog/images",
            catalog_images_router(app_state.clone()),
        )
        .nest(
            "/api/v1/catalog/reviews",
            catalog_reviews_router(app_state.clone()),
        )
        .nest(
            "/api/v1/catalog/wishlist",
            catalog_wishlist_router(app_state.clone()),
        )
        .nest(
            "/api/v1/catalog/storage-providers",
            catalog_storage_providers_router(app_state.clone()),
        )
        .nest("/api/v1/catalog/public", catalog_public_router())
        // Analytics
        .nest("/api/v1/analytics", analytics_router(app_state.clone()))
        // Accounting
        .nest("/api/v1/accounting", accounting_router(app_state.clone()))
        // Demand planning
        .nest("/api/v1/forecasts", forecasts_router(app_state.clone()))
        .nest(
            "/api/v1/reorder-policies",
            reorder_policies_router(app_state.clone()),
        )
        .nest(
            "/api/v1/replenishment-suggestions",
            replenishment_suggestions_router(app_state.clone()),
        )
        .nest(
            "/api/v1/abc-classification",
            abc_classification_router(app_state.clone()),
        )
        // Cash management
        .nest(
            "/api/v1/bank-accounts",
            bank_accounts_router(app_state.clone()),
        )
        .nest(
            "/api/v1/bank-transactions",
            bank_transactions_router(app_state.clone()),
        )
        .nest(
            "/api/v1/cash-deposits",
            cash_deposits_router(app_state.clone()),
        )
        .nest(
            "/api/v1/bank-reconciliations",
            bank_reconciliations_router(app_state.clone()),
        )
        // Loyalty
        .nest(
            "/api/v1/loyalty/programs",
            loyalty_programs_router(app_state.clone()),
        )
        .nest(
            "/api/v1/loyalty/tiers",
            loyalty_tiers_router(app_state.clone()),
        )
        .nest(
            "/api/v1/loyalty/rewards",
            loyalty_rewards_router(app_state.clone()),
        )
        .nest(
            "/api/v1/loyalty/members",
            loyalty_members_router(app_state.clone()),
        )
        // Booking
        .nest(
            "/api/v1/booking/resources",
            booking_resources_router(app_state.clone()),
        )
        .nest(
            "/api/v1/booking/services",
            booking_services_router(app_state.clone()),
        )
        .nest(
            "/api/v1/booking/appointments",
            booking_appointments_router(app_state.clone()),
        )
        .nest(
            "/api/v1/booking/policies",
            booking_policies_router(app_state.clone()),
        )
        .nest("/api/v1/public/booking", public_booking_router())
        // Service orders (workshop tickets)
        .nest(
            "/api/v1/assets",
            service_orders_assets_router(app_state.clone()),
        )
        .nest(
            "/api/v1/service-orders",
            service_orders_router(app_state.clone()),
        )
        .nest(
            "/api/v1/public/service-orders",
            public_service_orders_router(),
        )
        // Restaurant operations
        .nest(
            "/api/v1/restaurant/stations",
            restaurant_stations_router(app_state.clone()),
        )
        .nest(
            "/api/v1/restaurant/tables",
            restaurant_tables_router(app_state.clone()),
        )
        .nest(
            "/api/v1/restaurant/modifier-groups",
            restaurant_modifier_groups_router(app_state.clone()),
        )
        .nest(
            "/api/v1/restaurant/products",
            restaurant_product_modifiers_router(app_state.clone()),
        )
        .nest(
            "/api/v1/restaurant/kds/tickets",
            kds_tickets_router(app_state.clone()),
        )
        .nest(
            "/api/v1/restaurant/kds/stations",
            kds_stream_router(app_state.clone()),
        )
        // Tenancy
        .nest(
            "/api/v1/organizations",
            tenancy_organizations_router(app_state.clone()),
        )
        .nest("/api/v1/public/organizations", public_tenancy_router())
        // Static file serving for the LocalServer image storage adapter.
        // The mount path matches IMAGE_STORAGE_PUBLIC_URL (default `/uploads`).
        .nest_service(
            &std::env::var("IMAGE_STORAGE_PUBLIC_URL").unwrap_or_else(|_| "/uploads".to_string()),
            tower_http::services::ServeDir::new(
                std::env::var("IMAGE_STORAGE_ROOT").unwrap_or_else(|_| "./uploads".to_string()),
            ),
        )
        .layer(cors_layer)
        .with_state(app_state.clone());

    // Start background jobs
    let reservation_expiry_interval = env_or::<u64>("RESERVATION_EXPIRY_INTERVAL_SECS", 300);
    let cart_cleanup_interval = env_or::<u64>("CART_CLEANUP_INTERVAL_SECS", 900);
    let event_dispatch_interval = env_or::<u64>("EVENT_DISPATCH_INTERVAL_SECS", 5);
    let event_dispatch_batch_size = env_or::<i64>("EVENT_DISPATCH_BATCH_SIZE", 100);
    let notification_retry_interval = env_or::<u64>("NOTIFICATION_RETRY_INTERVAL_SECS", 60);
    let notification_retry_batch_size = env_or::<i64>("NOTIFICATION_RETRY_BATCH_SIZE", 50);
    let analytics_recompute_interval = env_or::<u64>("ANALYTICS_RECOMPUTE_INTERVAL_SECS", 1800);
    let demand_planning_interval = env_or::<u64>("DEMAND_PLANNING_RECOMPUTE_INTERVAL_SECS", 86_400);

    jobs::reservation_expiry::spawn(
        app_state.reservation_repo(),
        app_state.stock_repo(),
        reservation_expiry_interval,
    );
    jobs::cart_cleanup::spawn(app_state.cart_repo(), cart_cleanup_interval);
    jobs::event_dispatcher::spawn(
        app_state.outbox_repo(),
        app_state.subscriber_registry(),
        event_dispatch_interval,
        event_dispatch_batch_size,
    );
    jobs::notification_dispatcher::spawn(
        app_state.notification_repo(),
        app_state.notification_registry(),
        notification_retry_interval,
        notification_retry_batch_size,
    );
    jobs::analytics_recompute::spawn(
        app_state.analytics_query_repo(),
        app_state.kpi_snapshot_repo(),
        analytics_recompute_interval,
    );
    jobs::demand_planning_recompute::spawn(
        app_state.sales_history_repo(),
        app_state.demand_forecast_repo(),
        app_state.reorder_policy_repo(),
        app_state.stock_snapshot_repo(),
        app_state.replenishment_suggestion_repo(),
        app_state.abc_classification_repo(),
        demand_planning_interval,
    );

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
