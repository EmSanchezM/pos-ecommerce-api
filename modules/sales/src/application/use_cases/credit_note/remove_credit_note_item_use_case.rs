//! Remove credit note item use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CreditNoteResponse;
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::{CreditNoteId, CreditNoteItemId};
use crate::SalesError;

/// Use case for removing an item from a credit note
pub struct RemoveCreditNoteItemUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl RemoveCreditNoteItemUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(&self, credit_note_id: Uuid, item_id: Uuid) -> Result<CreditNoteResponse, SalesError> {
        let cn_id = CreditNoteId::from_uuid(credit_note_id);
        let item_id_vo = CreditNoteItemId::from_uuid(item_id);

        let mut credit_note = self
            .credit_note_repo
            .find_by_id_with_items(cn_id)
            .await?
            .ok_or(SalesError::CreditNoteNotFound(credit_note_id))?;

        credit_note.remove_item(item_id_vo)?;

        self.credit_note_repo.delete_item(item_id_vo).await?;
        self.credit_note_repo.update(&credit_note).await?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
