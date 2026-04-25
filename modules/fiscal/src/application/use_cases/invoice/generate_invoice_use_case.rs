//! Generate invoice use case

use std::str::FromStr;
use std::sync::Arc;

use rust_decimal::Decimal;

use crate::FiscalError;
use crate::application::dtos::{GenerateInvoiceCommand, InvoiceResponse};
use crate::domain::entities::Invoice;
use crate::domain::repositories::{FiscalSequenceRepository, InvoiceRepository, TaxRateRepository};
use crate::domain::value_objects::InvoiceType;
use identity::StoreId;
use inventory::Currency;
use pos_core::TerminalId;
use sales::SaleId;

/// Use case for generating a fiscal invoice from a completed sale
pub struct GenerateInvoiceUseCase {
    invoice_repo: Arc<dyn InvoiceRepository>,
    fiscal_seq_repo: Arc<dyn FiscalSequenceRepository>,
    tax_rate_repo: Arc<dyn TaxRateRepository>,
}

impl GenerateInvoiceUseCase {
    pub fn new(
        invoice_repo: Arc<dyn InvoiceRepository>,
        fiscal_seq_repo: Arc<dyn FiscalSequenceRepository>,
        tax_rate_repo: Arc<dyn TaxRateRepository>,
    ) -> Self {
        Self {
            invoice_repo,
            fiscal_seq_repo,
            tax_rate_repo,
        }
    }

    pub async fn execute(
        &self,
        cmd: GenerateInvoiceCommand,
    ) -> Result<InvoiceResponse, FiscalError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let terminal_id = TerminalId::from_uuid(cmd.terminal_id);
        let sale_id = SaleId::from_uuid(cmd.sale_id);
        let invoice_type = InvoiceType::from_str(&cmd.invoice_type)?;

        // Check if invoice already exists for this sale
        if self.invoice_repo.find_by_sale_id(sale_id).await?.is_some() {
            return Err(FiscalError::InvoiceAlreadyExistsForSale(cmd.sale_id));
        }

        // Credit notes require an original invoice reference
        if invoice_type.is_credit_note() {
            return Err(FiscalError::OriginalInvoiceRequired);
        }

        // Get the active fiscal sequence for this store/terminal
        let fiscal_seq = self
            .fiscal_seq_repo
            .find_active(store_id, terminal_id)
            .await?
            .ok_or(FiscalError::FiscalSequenceNotFound)?;

        // Get next invoice number from the sequence
        let invoice_number = self
            .fiscal_seq_repo
            .increment_and_get(fiscal_seq.id())
            .await?;

        // Get the default tax rates for the store (for future tax computation)
        let _tax_rates = self.tax_rate_repo.find_by_store(store_id).await?;

        // Create the invoice entity
        // For now, amounts are zero - they will be populated from sale data
        // when the full integration is implemented
        let invoice = Invoice::create(
            invoice_number,
            store_id,
            terminal_id,
            sale_id,
            fiscal_seq.cai_range_id(),
            invoice_type,
            None, // customer_id
            cmd.customer_name,
            cmd.customer_rtn,
            cmd.customer_address,
            Currency::default(),
            Decimal::ZERO,      // subtotal
            Decimal::ZERO,      // exempt_amount
            Decimal::ZERO,      // taxable_amount_15
            Decimal::ZERO,      // taxable_amount_18
            Decimal::ZERO,      // tax_15
            Decimal::ZERO,      // tax_18
            Decimal::ZERO,      // total_tax
            Decimal::ZERO,      // discount_amount
            Decimal::ZERO,      // total
            String::new(),      // amount_in_words
            String::new(),      // payment_method
            String::new(),      // cai_number (to be filled from CAI range)
            chrono::Utc::now(), // cai_expiry_date (to be filled from CAI range)
            String::new(),      // range_start
            String::new(),      // range_end
            None,               // original_invoice_id
            Vec::new(),         // items
        );

        // Save the invoice
        self.invoice_repo.save(&invoice).await?;

        Ok(InvoiceResponse::from(invoice))
    }
}
