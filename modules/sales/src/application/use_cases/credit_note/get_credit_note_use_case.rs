//! Get credit note use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CreditNoteResponse;
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::CreditNoteId;
use crate::SalesError;

/// Use case for getting a credit note by ID
pub struct GetCreditNoteUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl GetCreditNoteUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(&self, credit_note_id: Uuid) -> Result<CreditNoteResponse, SalesError> {
        let cn_id = CreditNoteId::from_uuid(credit_note_id);

        let credit_note = self
            .credit_note_repo
            .find_by_id_with_items(cn_id)
            .await?
            .ok_or(SalesError::CreditNoteNotFound(credit_note_id))?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
