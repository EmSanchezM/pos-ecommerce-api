//! List invoices use case

use std::str::FromStr;
use std::sync::Arc;

use chrono::NaiveDate;

use crate::FiscalError;
use crate::application::dtos::{InvoiceListResponse, InvoiceSummaryResponse, ListInvoicesQuery};
use crate::domain::repositories::{InvoiceFilter, InvoiceRepository};
use crate::domain::value_objects::{InvoiceStatus, InvoiceType};
use identity::StoreId;
use pos_core::TerminalId;

/// Use case for listing invoices with filters and pagination
pub struct ListInvoicesUseCase {
    invoice_repo: Arc<dyn InvoiceRepository>,
}

impl ListInvoicesUseCase {
    pub fn new(invoice_repo: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn execute(
        &self,
        query: ListInvoicesQuery,
    ) -> Result<InvoiceListResponse, FiscalError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

        let date_from = query.date_from.as_ref().and_then(|s| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
        });

        let date_to = query.date_to.as_ref().and_then(|s| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .map(|d| d.and_hms_opt(23, 59, 59).unwrap().and_utc())
        });

        let filter = InvoiceFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            terminal_id: query.terminal_id.map(TerminalId::from_uuid),
            invoice_type: query
                .invoice_type
                .as_ref()
                .and_then(|s| InvoiceType::from_str(s).ok()),
            status: query
                .status
                .as_ref()
                .and_then(|s| InvoiceStatus::from_str(s).ok()),
            customer_rtn: query.customer_rtn,
            date_from,
            date_to,
            search: query.search,
        };

        let (invoices, total) = self
            .invoice_repo
            .find_paginated(filter, page, page_size)
            .await?;

        let total_pages = (total as f64 / page_size as f64).ceil() as i64;

        Ok(InvoiceListResponse {
            items: invoices.iter().map(InvoiceSummaryResponse::from).collect(),
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
