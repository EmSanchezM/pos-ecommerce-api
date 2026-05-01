//! Get invoice use case

use std::sync::Arc;
use uuid::Uuid;

use crate::FiscalError;
use crate::application::dtos::InvoiceResponse;
use crate::domain::repositories::InvoiceRepository;
use crate::domain::value_objects::InvoiceId;

/// Use case for retrieving an invoice by ID
pub struct GetInvoiceUseCase {
    invoice_repo: Arc<dyn InvoiceRepository>,
}

impl GetInvoiceUseCase {
    pub fn new(invoice_repo: Arc<dyn InvoiceRepository>) -> Self {
        Self { invoice_repo }
    }

    pub async fn execute(&self, invoice_id: Uuid) -> Result<InvoiceResponse, FiscalError> {
        let id = InvoiceId::from_uuid(invoice_id);

        let invoice = self
            .invoice_repo
            .find_by_id(id)
            .await?
            .ok_or(FiscalError::InvoiceNotFound(invoice_id))?;

        Ok(InvoiceResponse::from(invoice))
    }
}
