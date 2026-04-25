//! Domain layer for the fiscal module.
//!
//! This module contains the core business logic including:
//! - Entities: Invoice, InvoiceLine, TaxRate, FiscalSequence
//! - Value objects: IDs, status enums, tax types
//! - Repository traits

pub mod entities;
pub mod repositories;
pub mod value_objects;
