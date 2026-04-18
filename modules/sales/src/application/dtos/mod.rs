//! DTOs (Data Transfer Objects) for the sales module.
//!
//! Contains commands (inputs) and responses (outputs) for all operations.

pub mod cart;
pub mod credit_note;
pub mod customer;
pub mod promotion;
pub mod sale;
pub mod shift;

pub use cart::*;
pub use credit_note::*;
pub use customer::*;
pub use promotion::commands::{
    ApplyPromotionCommand, CreatePromotionCommand, UpdatePromotionCommand,
};
pub use promotion::responses::PromotionResponse;
pub use sale::*;
pub use shift::*;
