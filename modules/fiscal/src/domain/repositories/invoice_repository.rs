//! Invoice repository trait

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::FiscalError;
use crate::domain::entities::{Invoice, InvoiceLine};
use crate::domain::value_objects::{InvoiceId, InvoiceStatus, InvoiceType};
use identity::StoreId;
use pos_core::TerminalId;
use sales::SaleId;

/// Filter for querying invoices
#[derive(Debug, Clone, Default)]
pub struct InvoiceFilter {
    pub store_id: Option<StoreId>,
    pub terminal_id: Option<TerminalId>,
    pub invoice_type: Option<InvoiceType>,
    pub status: Option<InvoiceStatus>,
    pub customer_rtn: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

/// Repository trait for Invoice persistence
#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    /// Saves a new invoice
    async fn save(&self, invoice: &Invoice) -> Result<(), FiscalError>;

    /// Finds an invoice by ID
    async fn find_by_id(&self, id: InvoiceId) -> Result<Option<Invoice>, FiscalError>;

    /// Finds an invoice by ID with its line items
    async fn find_by_id_with_lines(&self, id: InvoiceId) -> Result<Option<Invoice>, FiscalError>;

    /// Finds an invoice by number within a store
    async fn find_by_number(
        &self,
        store_id: StoreId,
        number: &str,
    ) -> Result<Option<Invoice>, FiscalError>;

    /// Finds an invoice by sale ID
    async fn find_by_sale_id(&self, sale_id: SaleId) -> Result<Option<Invoice>, FiscalError>;

    /// Updates an existing invoice
    async fn update(&self, invoice: &Invoice) -> Result<(), FiscalError>;

    /// Finds invoices with pagination
    async fn find_paginated(
        &self,
        filter: InvoiceFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Invoice>, i64), FiscalError>;

    /// Saves an invoice line
    async fn save_line(&self, line: &InvoiceLine) -> Result<(), FiscalError>;

    /// Finds all lines for an invoice
    async fn find_lines_by_invoice(
        &self,
        invoice_id: InvoiceId,
    ) -> Result<Vec<InvoiceLine>, FiscalError>;
}
