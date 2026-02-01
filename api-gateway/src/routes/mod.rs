// HTTP routes for the API Gateway

pub mod auth_routes;
pub mod inventory_routes;
pub mod purchasing_routes;
pub mod sales_routes;
pub mod store_routes;
pub mod terminal_routes;

pub use auth_routes::auth_router;
pub use inventory_routes::{inventory_router, products_router, recipes_router, reports_router};
pub use purchasing_routes::{goods_receipts_router, purchase_orders_router, vendors_router};
pub use sales_routes::{cart_router, credit_notes_router, customers_router, pos_sales_router, shifts_router};
pub use store_routes::store_router;
pub use terminal_routes::{store_terminals_router, terminals_router};
