// HTTP handlers for the API Gateway

pub mod accounting;
pub mod analytics;
pub mod auth_handlers;
pub mod booking;
pub mod cai_handlers;
pub mod cash_management;
pub mod catalog;
pub mod demand_planning;
pub mod fiscal;
pub mod inventory;
pub mod loyalty;
pub mod payments;
pub mod purchasing;
pub mod restaurant;
pub mod sales;
pub mod service_orders;
pub mod shipping;
pub mod store_handlers;
pub mod tenancy;
pub mod terminal_handlers;

pub use auth_handlers::*;
pub use cai_handlers::*;
// fiscal handlers are used directly via crate::handlers::fiscal::{handler_name}
pub use inventory::*;
pub use purchasing::*;
pub use sales::*;
pub use store_handlers::*;
pub use terminal_handlers::*;
