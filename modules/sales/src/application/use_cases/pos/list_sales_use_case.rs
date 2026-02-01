//! List sales use case

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::{ListSalesQuery, SaleListResponse, SaleResponse};
use crate::domain::repositories::{SaleFilter, SaleRepository};
use crate::domain::value_objects::{CustomerId, SaleStatus, SaleType, ShiftId};
use crate::SalesError;
use identity::StoreId;
use pos_core::TerminalId;

/// Use case for listing sales with filters and pagination
pub struct ListSalesUseCase {
    sale_repo: Arc<dyn SaleRepository>,
}

impl ListSalesUseCase {
    pub fn new(sale_repo: Arc<dyn SaleRepository>) -> Self {
        Self { sale_repo }
    }

    pub async fn execute(&self, query: ListSalesQuery) -> Result<SaleListResponse, SalesError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100).max(1);

        let filter = SaleFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            terminal_id: query.terminal_id.map(TerminalId::from_uuid),
            customer_id: query.customer_id.map(CustomerId::from_uuid),
            shift_id: query.shift_id.map(ShiftId::from_uuid),
            sale_type: query
                .sale_type
                .as_ref()
                .and_then(|s| SaleType::from_str(s).ok()),
            status: query
                .status
                .as_ref()
                .and_then(|s| SaleStatus::from_str(s).ok()),
            search: query.search,
        };

        let (sales, total) = self.sale_repo.find_paginated(filter, page, page_size).await?;

        let total_pages = (total as f64 / page_size as f64).ceil() as i64;

        Ok(SaleListResponse {
            data: sales.iter().map(SaleResponse::from).collect(),
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
