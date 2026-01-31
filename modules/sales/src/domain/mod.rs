//! Domain layer for the sales module.
//!
//! This module contains the core business logic including:
//! - Entities (Customer, Sale, Payment, Cart, CashierShift, CreditNote)
//! - Value objects (IDs, status enums)
//! - Repository traits

pub mod entities;
pub mod repositories;
pub mod value_objects;
