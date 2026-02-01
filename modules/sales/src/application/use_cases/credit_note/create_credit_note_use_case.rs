//! Create credit note use case

use std::sync::Arc;

use crate::application::dtos::{CreateCreditNoteCommand, CreditNoteResponse};
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::{ReturnReason, ReturnType, SaleId};
use crate::SalesError;
use identity::StoreId;

/// Use case for creating a new credit note
pub struct CreateCreditNoteUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl CreateCreditNoteUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateCreditNoteCommand,
        created_by_id: identity::UserId,
    ) -> Result<CreditNoteResponse, SalesError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let original_sale_id = SaleId::from_uuid(cmd.original_sale_id);

        let return_type: ReturnType = cmd.return_type.parse()?;
        let return_reason: ReturnReason = cmd.return_reason.parse()?;

        // Generate a unique credit note number
        let credit_note_number = self
            .credit_note_repo
            .generate_credit_note_number(store_id)
            .await?;

        let mut credit_note = crate::domain::entities::CreditNote::create(
            credit_note_number,
            store_id,
            original_sale_id,
            cmd.original_invoice_number,
            return_type,
            return_reason,
            inventory::Currency::new("HNL").map_err(|_| SalesError::InvalidCurrency)?,
            created_by_id,
        );

        credit_note.set_reason_details(cmd.reason_details)?;
        credit_note.set_notes(cmd.notes)?;

        self.credit_note_repo.save(&credit_note).await?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
