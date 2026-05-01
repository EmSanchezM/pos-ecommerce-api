// HTTP handlers for the API Gateway

pub mod accounting;
pub mod analytics;
pub mod auth_handlers;
pub mod cai_handlers;
pub mod catalog;
pub mod fiscal;
pub mod inventory;
pub mod payments;
pub mod purchasing;
pub mod sales;
pub mod shipping;
pub mod store_handlers;
pub mod terminal_handlers;

pub use auth_handlers::*;
pub use cai_handlers::*;
// fiscal handlers are used directly via crate::handlers::fiscal::{handler_name}
pub use inventory::*;
pub use purchasing::*;
pub use sales::*;
pub use store_handlers::*;
pub use terminal_handlers::*;
