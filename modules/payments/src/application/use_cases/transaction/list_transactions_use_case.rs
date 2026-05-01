//! Paginated transaction listing.

use std::str::FromStr;
use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{
    ListTransactionsQuery, TransactionListResponse, TransactionResponse,
};
use crate::domain::repositories::{TransactionFilter, TransactionRepository};
use crate::domain::value_objects::{PaymentGatewayId, TransactionStatus, TransactionType};
use identity::StoreId;
use sales::SaleId;

pub struct ListTransactionsUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl ListTransactionsUseCase {
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(
        &self,
        query: ListTransactionsQuery,
    ) -> Result<TransactionListResponse, PaymentsError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(50).clamp(1, 200);

        let filter = TransactionFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            gateway_id: query.gateway_id.map(PaymentGatewayId::from_uuid),
            sale_id: query.sale_id.map(SaleId::from_uuid),
            transaction_type: query
                .transaction_type
                .as_deref()
                .map(TransactionType::from_str)
                .transpose()?,
            status: query
                .status
                .as_deref()
                .map(TransactionStatus::from_str)
                .transpose()?,
            date_from: query.date_from,
            date_to: query.date_to,
            search: query.search,
        };

        let (rows, total) = self
            .transaction_repo
            .find_paginated(filter, page, page_size)
            .await?;

        Ok(TransactionListResponse {
            items: rows.into_iter().map(TransactionResponse::from).collect(),
            total,
            page,
            page_size,
        })
    }
}
