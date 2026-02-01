//! Cancel credit note use case

use std::sync::Arc;

use crate::application::dtos::{CancelCreditNoteCommand, CreditNoteResponse};
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::CreditNoteId;
use crate::SalesError;

/// Use case for cancelling a credit note
pub struct CancelCreditNoteUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl CancelCreditNoteUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(
        &self,
        cmd: CancelCreditNoteCommand,
        cancelled_by_id: identity::UserId,
    ) -> Result<CreditNoteResponse, SalesError> {
        let cn_id = CreditNoteId::from_uuid(cmd.credit_note_id);

        let mut credit_note = self
            .credit_note_repo
            .find_by_id_with_items(cn_id)
            .await?
            .ok_or(SalesError::CreditNoteNotFound(cmd.credit_note_id))?;

        credit_note.cancel(cancelled_by_id, cmd.reason)?;

        self.credit_note_repo.update(&credit_note).await?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
