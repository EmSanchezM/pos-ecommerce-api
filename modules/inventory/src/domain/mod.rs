//! Domain layer for the inventory module.
//!
//! This layer contains the core business logic, including:
//!
//! - **Entities**: Domain objects with identity and lifecycle (Product, InventoryStock, Recipe, etc.)
//! - **Value Objects**: Immutable objects defined by their attributes (ProductId, Sku, Currency, etc.)
//! - **Repository Traits**: Abstractions for data persistence
//!
//! The domain layer has no dependencies on external frameworks or infrastructure concerns.

pub mod entities;
pub mod repositories;
pub mod value_objects;
