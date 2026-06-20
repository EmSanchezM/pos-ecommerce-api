use axum::{Router, routing::get};
use common::health::infrastructure::health_check_simple;

use crate::config::AppConfig;
use crate::handlers::internal::InternalState;
use crate::routes::internal_router;
use crate::routes::{
    abc_classification_router, accounting_router, admin_subscriptions_router, analytics_router,
    auth_router, bank_accounts_router, bank_reconciliations_router, bank_transactions_router,
    booking_appointments_router, booking_policies_router, booking_resources_router,
    booking_services_router, cart_router, cash_deposits_router, catalog_images_router,
    catalog_listings_router, catalog_public_router, catalog_reviews_router,
    catalog_storage_providers_router, catalog_wishlist_router, categories_router,
    credit_notes_router, customers_router, delivery_providers_router, delivery_webhooks_router,
    drivers_router, forecasts_router, goods_receipts_router, inventory_router, invoices_router,
    kds_stream_router, kds_tickets_router, loyalty_members_router, loyalty_programs_router,
    loyalty_rewards_router, loyalty_tiers_router, orders_router, organization_subscription_router,
    payment_gateways_router, payouts_router, pos_sales_router, products_router, promotions_router,
    public_booking_router, public_service_orders_router, public_subscription_plans_router,
    public_tenancy_router, public_tracking_router, purchase_orders_router, recipes_router,
    reorder_policies_router, replenishment_suggestions_router, reports_router,
    restaurant_modifier_groups_router, restaurant_product_modifiers_router,
    restaurant_stations_router, restaurant_tables_router, service_orders_assets_router,
    service_orders_router, shifts_router, shipments_router, shipping_calculate_router,
    shipping_methods_router, shipping_rates_router, shipping_zones_router, store_router,
    store_terminals_router, subscription_plans_router, tax_rates_router,
    tenancy_organizations_router, terminals_router, transactions_router, transfers_router,
    vendors_router, webhooks_router,
};
use crate::state::AppState;

pub fn build_router(app_state: AppState, config: &AppConfig) -> Router {
    // Internal service-to-service router — its own state + shared-secret auth,
    // deliberately outside the tenant auth middleware. Backs impersonation v2.
    let internal_state = InternalState {
        token_service: app_state.token_service(),
        user_repo: app_state.user_repo(),
        internal_secret: config.internal_service_secret.clone().into(),
    };

    Router::new()
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
        // Subscriptions (SaaS billing of the platform itself)
        .nest(
            "/api/v1/subscription-plans",
            subscription_plans_router(app_state.clone()),
        )
        .nest(
            "/api/v1/public/subscription-plans",
            public_subscription_plans_router(),
        )
        .nest(
            "/api/v1/organizations",
            organization_subscription_router(app_state.clone()),
        )
        .nest(
            "/api/v1/admin/subscriptions",
            admin_subscriptions_router(app_state.clone()),
        )
        // Static file serving for the LocalServer image storage adapter.
        // The mount path matches IMAGE_STORAGE_PUBLIC_URL (default `/uploads`).
        .nest_service(
            &config.image_storage.public_url,
            tower_http::services::ServeDir::new(&config.image_storage.root),
        )
        .with_state(app_state)
        // Internal router carries its own state, so it is nested after the
        // tenant router is fully stated (both are `Router<()>` here).
        .nest("/internal", internal_router(internal_state))
}
