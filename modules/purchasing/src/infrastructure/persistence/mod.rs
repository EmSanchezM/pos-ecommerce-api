//! PostgreSQL repository implementations for purchasing management.
//!
//! This module provides concrete implementations of the repository traits
//! using PostgreSQL as the persistence backend.

mod pg_vendor_repository;
mod pg_purchase_order_repository;
mod pg_goods_receipt_repository;

pub use pg_vendor_repository::PgVendorRepository;
pub use pg_purchase_order_repository::PgPurchaseOrderRepository;
pub use pg_goods_receipt_repository::PgGoodsReceiptRepository;
