//! Approve credit note use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CreditNoteResponse;
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::CreditNoteId;
use crate::SalesError;

/// Use case for approving a credit note
pub struct ApproveCreditNoteUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl ApproveCreditNoteUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(
        &self,
        credit_note_id: Uuid,
        approver_id: identity::UserId,
    ) -> Result<CreditNoteResponse, SalesError> {
        let cn_id = CreditNoteId::from_uuid(credit_note_id);

        let mut credit_note = self
            .credit_note_repo
            .find_by_id_with_items(cn_id)
            .await?
            .ok_or(SalesError::CreditNoteNotFound(credit_note_id))?;

        credit_note.approve(approver_id)?;

        self.credit_note_repo.update(&credit_note).await?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
