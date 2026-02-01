//! Apply credit note use case

use std::sync::Arc;

use crate::application::dtos::{ApplyCreditNoteCommand, CreditNoteResponse};
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::CreditNoteId;
use crate::SalesError;

/// Use case for applying (processing the refund of) a credit note
pub struct ApplyCreditNoteUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl ApplyCreditNoteUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(
        &self,
        cmd: ApplyCreditNoteCommand,
        applied_by_id: identity::UserId,
    ) -> Result<CreditNoteResponse, SalesError> {
        let cn_id = CreditNoteId::from_uuid(cmd.credit_note_id);

        let mut credit_note = self
            .credit_note_repo
            .find_by_id_with_items(cn_id)
            .await?
            .ok_or(SalesError::CreditNoteNotFound(cmd.credit_note_id))?;

        credit_note.apply(applied_by_id, cmd.refund_method)?;

        self.credit_note_repo.update(&credit_note).await?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
