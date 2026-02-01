//! List credit notes use case

use std::sync::Arc;

use crate::application::dtos::{CreditNoteListResponse, CreditNoteResponse, ListCreditNotesQuery};
use crate::domain::repositories::{CreditNoteFilter, CreditNoteRepository};
use crate::domain::value_objects::SaleId;
use crate::SalesError;
use identity::StoreId;

/// Use case for listing credit notes with pagination
pub struct ListCreditNotesUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl ListCreditNotesUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(&self, query: ListCreditNotesQuery) -> Result<CreditNoteListResponse, SalesError> {
        let status = match &query.status {
            Some(s) => Some(s.parse()?),
            None => None,
        };

        let filter = CreditNoteFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            original_sale_id: query.original_sale_id.map(SaleId::from_uuid),
            status,
            search: query.search,
        };

        let (credit_notes, total) = self
            .credit_note_repo
            .find_paginated(filter, query.page, query.page_size)
            .await?;

        let data: Vec<CreditNoteResponse> = credit_notes.into_iter().map(CreditNoteResponse::from).collect();

        Ok(CreditNoteListResponse {
            data,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }
}
