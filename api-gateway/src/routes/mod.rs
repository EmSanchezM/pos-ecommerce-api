// HTTP routes for the API Gateway

pub mod auth_routes;
pub mod inventory_routes;
pub mod store_routes;
pub mod terminal_routes;

pub use auth_routes::auth_router;
pub use inventory_routes::{inventory_router, products_router, recipes_router};
pub use store_routes::store_router;
pub use terminal_routes::{store_terminals_router, terminals_router};
