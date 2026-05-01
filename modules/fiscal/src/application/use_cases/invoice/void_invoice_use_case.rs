//! Void invoice use case

use std::sync::Arc;

use crate::FiscalError;
use crate::application::dtos::{InvoiceResponse, VoidInvoiceCommand};
use crate::domain::repositories::InvoiceRepository;
use crate::domain::value_objects::InvoiceId;
use identity::UserId;

/// Use case for voiding an emitted invoice
pub struct VoidInvoiceUseCase {
    invoice_repo: Arc<dyn InvoiceRepository>,
}

impl VoidInvoiceUseCase {
    pub fn new(invoice_repo: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn execute(
        &self,
        cmd: VoidInvoiceCommand,
        voided_by: UserId,
    ) -> Result<InvoiceResponse, FiscalError> {
        let invoice_id = InvoiceId::from_uuid(cmd.invoice_id);

        let mut invoice = self
            .invoice_repo
            .find_by_id(invoice_id)
            .await?
            .ok_or(FiscalError::InvoiceNotFound(cmd.invoice_id))?;

        // Void the invoice (entity validates status transition)
        invoice.void(voided_by, cmd.reason)?;

        // Update the invoice
        self.invoice_repo.update(&invoice).await?;

        Ok(InvoiceResponse::from(invoice))
    }
}
