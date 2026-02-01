//! List customers use case

use std::sync::Arc;

use crate::application::dtos::{CustomerListResponse, CustomerResponse, ListCustomersQuery};
use crate::domain::repositories::{CustomerFilter, CustomerRepository};
use crate::SalesError;
use identity::StoreId;

/// Use case for listing customers with filters and pagination
pub struct ListCustomersUseCase {
    customer_repo: Arc<dyn CustomerRepository>,
}

impl ListCustomersUseCase {
    pub fn new(customer_repo: Arc<dyn CustomerRepository>) -> Self {
        Self { customer_repo }
    }

    pub async fn execute(&self, query: ListCustomersQuery) -> Result<CustomerListResponse, SalesError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100).max(1);

        let filter = CustomerFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            search: query.search,
            is_active: query.is_active,
        };

        let (customers, total) = self
            .customer_repo
            .find_paginated(filter, page, page_size)
            .await?;

        let total_pages = (total as f64 / page_size as f64).ceil() as i64;

        Ok(CustomerListResponse {
            data: customers.into_iter().map(CustomerResponse::from).collect(),
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
