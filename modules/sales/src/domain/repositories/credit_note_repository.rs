//! CreditNote repository trait

use async_trait::async_trait;

use crate::domain::entities::{CreditNote, CreditNoteItem};
use crate::domain::value_objects::{
    CreditNoteId, CreditNoteItemId, CreditNoteStatus, SaleId,
};
use crate::SalesError;
use identity::StoreId;

/// Filter for querying credit notes
#[derive(Debug, Clone, Default)]
pub struct CreditNoteFilter {
    pub store_id: Option<StoreId>,
    pub original_sale_id: Option<SaleId>,
    pub status: Option<CreditNoteStatus>,
    pub search: Option<String>,
}

/// Repository trait for CreditNote persistence
#[async_trait]
pub trait CreditNoteRepository: Send + Sync {
    /// Saves a new credit note
    async fn save(&self, credit_note: &CreditNote) -> Result<(), SalesError>;

    /// Finds a credit note by ID
    async fn find_by_id(&self, id: CreditNoteId) -> Result<Option<CreditNote>, SalesError>;

    /// Finds a credit note by ID with items
    async fn find_by_id_with_items(
        &self,
        id: CreditNoteId,
    ) -> Result<Option<CreditNote>, SalesError>;

    /// Finds a credit note by number
    async fn find_by_number(
        &self,
        store_id: StoreId,
        number: &str,
    ) -> Result<Option<CreditNote>, SalesError>;

    /// Finds credit notes for an original sale
    async fn find_by_sale(
        &self,
        sale_id: SaleId,
    ) -> Result<Vec<CreditNote>, SalesError>;

    /// Updates an existing credit note
    async fn update(&self, credit_note: &CreditNote) -> Result<(), SalesError>;

    /// Finds credit notes with pagination
    async fn find_paginated(
        &self,
        filter: CreditNoteFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CreditNote>, i64), SalesError>;

    /// Generates a unique credit note number for a store
    async fn generate_credit_note_number(
        &self,
        store_id: StoreId,
    ) -> Result<String, SalesError>;

    // -------------------------------------------------------------------------
    // Credit Note Item operations
    // -------------------------------------------------------------------------

    /// Saves a credit note item
    async fn save_item(&self, item: &CreditNoteItem) -> Result<(), SalesError>;

    /// Updates a credit note item
    async fn update_item(&self, item: &CreditNoteItem) -> Result<(), SalesError>;

    /// Deletes a credit note item
    async fn delete_item(&self, item_id: CreditNoteItemId) -> Result<(), SalesError>;

    /// Finds items for a credit note
    async fn find_items_by_credit_note(
        &self,
        credit_note_id: CreditNoteId,
    ) -> Result<Vec<CreditNoteItem>, SalesError>;

    /// Finds a credit note item by ID
    async fn find_item_by_id(
        &self,
        item_id: CreditNoteItemId,
    ) -> Result<Option<CreditNoteItem>, SalesError>;
}
