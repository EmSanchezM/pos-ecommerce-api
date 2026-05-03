// HTTP routes for the API Gateway

pub mod accounting_routes;
pub mod analytics_routes;
pub mod auth_routes;
pub mod booking_routes;
pub mod cash_management_routes;
pub mod catalog_routes;
pub mod demand_planning_routes;
pub mod fiscal_routes;
pub mod inventory_routes;
pub mod loyalty_routes;
pub mod payments_routes;
pub mod purchasing_routes;
pub mod sales_routes;
pub mod service_orders_routes;
pub mod shipping_routes;
pub mod store_routes;
pub mod terminal_routes;

pub use accounting_routes::accounting_router;
pub use analytics_routes::analytics_router;
pub use auth_routes::auth_router;
pub use booking_routes::{
    booking_appointments_router, booking_policies_router, booking_resources_router,
    booking_services_router, public_booking_router,
};
pub use cash_management_routes::{
    bank_accounts_router, bank_reconciliations_router, bank_transactions_router,
    cash_deposits_router,
};
pub use catalog_routes::{
    catalog_images_router, catalog_listings_router, catalog_public_router, catalog_reviews_router,
    catalog_storage_providers_router, catalog_wishlist_router,
};
pub use demand_planning_routes::{
    abc_classification_router, forecasts_router, reorder_policies_router,
    replenishment_suggestions_router,
};
pub use fiscal_routes::{invoices_router, tax_rates_router};
pub use inventory_routes::{
    categories_router, inventory_router, products_router, recipes_router, reports_router,
    transfers_router,
};
pub use loyalty_routes::{
    loyalty_members_router, loyalty_programs_router, loyalty_rewards_router, loyalty_tiers_router,
};
pub use payments_routes::{
    payment_gateways_router, payouts_router, transactions_router, webhooks_router,
};
pub use purchasing_routes::{goods_receipts_router, purchase_orders_router, vendors_router};
pub use sales_routes::{
    cart_router, credit_notes_router, customers_router, orders_router, pos_sales_router,
    promotions_router, shifts_router,
};
pub use service_orders_routes::{
    public_service_orders_router, service_orders_assets_router, service_orders_router,
};
pub use shipping_routes::{
    delivery_providers_router, delivery_webhooks_router, drivers_router, public_tracking_router,
    shipments_router, shipping_calculate_router, shipping_methods_router, shipping_rates_router,
    shipping_zones_router,
};
pub use store_routes::store_router;
pub use terminal_routes::{store_terminals_router, terminals_router};
