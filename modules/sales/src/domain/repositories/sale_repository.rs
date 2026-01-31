//! Sale repository trait

use async_trait::async_trait;

use crate::domain::entities::{Payment, Sale, SaleItem};
use crate::domain::value_objects::{
    CustomerId, PaymentId, SaleId, SaleItemId, SaleStatus, SaleType, ShiftId,
};
use crate::SalesError;
use pos_core::TerminalId;
use identity::StoreId;

/// Filter for querying sales
#[derive(Debug, Clone, Default)]
pub struct SaleFilter {
    pub store_id: Option<StoreId>,
    pub terminal_id: Option<TerminalId>,
    pub shift_id: Option<ShiftId>,
    pub customer_id: Option<CustomerId>,
    pub sale_type: Option<SaleType>,
    pub status: Option<SaleStatus>,
    pub search: Option<String>,
}

/// Repository trait for Sale persistence
#[async_trait]
pub trait SaleRepository: Send + Sync {
    /// Saves a new sale
    async fn save(&self, sale: &Sale) -> Result<(), SalesError>;

    /// Finds a sale by ID
    async fn find_by_id(&self, id: SaleId) -> Result<Option<Sale>, SalesError>;

    /// Finds a sale by ID with items and payments
    async fn find_by_id_with_details(&self, id: SaleId) -> Result<Option<Sale>, SalesError>;

    /// Finds a sale by sale number
    async fn find_by_sale_number(
        &self,
        store_id: StoreId,
        sale_number: &str,
    ) -> Result<Option<Sale>, SalesError>;

    /// Finds a sale by invoice number
    async fn find_by_invoice_number(
        &self,
        store_id: StoreId,
        invoice_number: &str,
    ) -> Result<Option<Sale>, SalesError>;

    /// Updates an existing sale
    async fn update(&self, sale: &Sale) -> Result<(), SalesError>;

    /// Finds sales with pagination
    async fn find_paginated(
        &self,
        filter: SaleFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Sale>, i64), SalesError>;

    /// Generates a unique sale number for a store
    async fn generate_sale_number(&self, store_id: StoreId) -> Result<String, SalesError>;

    // -------------------------------------------------------------------------
    // Sale Item operations
    // -------------------------------------------------------------------------

    /// Saves a sale item
    async fn save_item(&self, item: &SaleItem) -> Result<(), SalesError>;

    /// Updates a sale item
    async fn update_item(&self, item: &SaleItem) -> Result<(), SalesError>;

    /// Deletes a sale item
    async fn delete_item(&self, item_id: SaleItemId) -> Result<(), SalesError>;

    /// Finds items for a sale
    async fn find_items_by_sale(&self, sale_id: SaleId) -> Result<Vec<SaleItem>, SalesError>;

    /// Finds a sale item by ID
    async fn find_item_by_id(&self, item_id: SaleItemId) -> Result<Option<SaleItem>, SalesError>;

    // -------------------------------------------------------------------------
    // Payment operations
    // -------------------------------------------------------------------------

    /// Saves a payment
    async fn save_payment(&self, payment: &Payment) -> Result<(), SalesError>;

    /// Updates a payment
    async fn update_payment(&self, payment: &Payment) -> Result<(), SalesError>;

    /// Finds payments for a sale
    async fn find_payments_by_sale(&self, sale_id: SaleId) -> Result<Vec<Payment>, SalesError>;

    /// Finds a payment by ID
    async fn find_payment_by_id(&self, payment_id: PaymentId) -> Result<Option<Payment>, SalesError>;
}
