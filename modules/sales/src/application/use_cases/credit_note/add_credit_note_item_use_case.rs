//! Add credit note item use case

use std::sync::Arc;

use crate::application::dtos::{AddCreditNoteItemCommand, CreditNoteResponse};
use crate::domain::entities::CreditNoteItem;
use crate::domain::repositories::CreditNoteRepository;
use crate::domain::value_objects::{CreditNoteId, SaleItemId};
use crate::SalesError;
use inventory::{ProductId, VariantId};

/// Use case for adding an item to a credit note
pub struct AddCreditNoteItemUseCase {
    credit_note_repo: Arc<dyn CreditNoteRepository>,
}

impl AddCreditNoteItemUseCase {
    pub fn new(credit_note_repo: Arc<dyn CreditNoteRepository>) -> Self {
        Self { credit_note_repo }
    }

    pub async fn execute(&self, cmd: AddCreditNoteItemCommand) -> Result<CreditNoteResponse, SalesError> {
        let credit_note_id = CreditNoteId::from_uuid(cmd.credit_note_id);

        let mut credit_note = self
            .credit_note_repo
            .find_by_id_with_items(credit_note_id)
            .await?
            .ok_or(SalesError::CreditNoteNotFound(cmd.credit_note_id))?;

        let uom: inventory::UnitOfMeasure = cmd.unit_of_measure.parse()
            .map_err(|_| SalesError::InvalidUnitOfMeasure)?;

        let item = CreditNoteItem::create(
            credit_note_id,
            SaleItemId::from_uuid(cmd.original_sale_item_id),
            ProductId::from_uuid(cmd.product_id),
            cmd.variant_id.map(VariantId::from_uuid),
            cmd.sku,
            cmd.description,
            cmd.return_quantity,
            uom,
            cmd.unit_price,
            cmd.tax_rate,
        )?;

        self.credit_note_repo.save_item(&item).await?;
        credit_note.add_item(item)?;
        self.credit_note_repo.update(&credit_note).await?;

        Ok(CreditNoteResponse::from(credit_note))
    }
}
