//! Generate invoice use case - integrated with Sales module
//!
//! Supports both POS and eCommerce workflows:
//! - POS: Sale must be `Completed`. `terminal_id` is required.
//! - eCommerce: Order must be `Paid`, `Processing`, `Shipped`, or `Delivered`.
//!   `terminal_id` is optional — auto-resolved from the store's first active terminal.

use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::FiscalError;
use crate::application::dtos::{GenerateInvoiceCommand, InvoiceResponse};
use crate::domain::entities::{Invoice, InvoiceLine};
use crate::domain::repositories::{FiscalSequenceRepository, InvoiceRepository, TaxRateRepository};
use crate::domain::value_objects::{InvoiceType, TaxType};
use identity::StoreId;
use inventory::Currency;
use pos_core::{TerminalId, TerminalRepository};
use sales::{OrderStatus, SaleId, SaleRepository, SaleStatus, SaleType};

/// Use case for generating a fiscal invoice from a sale.
///
/// Reads the sale data (items, totals, payments) from the Sales module,
/// validates the sale is in a billable state, reads the CAI from the terminal,
/// generates the correlative fiscal number, and creates the invoice.
pub struct GenerateInvoiceUseCase {
    invoice_repo: Arc<dyn InvoiceRepository>,
    fiscal_seq_repo: Arc<dyn FiscalSequenceRepository>,
    tax_rate_repo: Arc<dyn TaxRateRepository>,
    sale_repo: Arc<dyn SaleRepository>,
    terminal_repo: Arc<dyn TerminalRepository>,
}

impl GenerateInvoiceUseCase {
    pub fn new(
        invoice_repo: Arc<dyn InvoiceRepository>,
        fiscal_seq_repo: Arc<dyn FiscalSequenceRepository>,
        tax_rate_repo: Arc<dyn TaxRateRepository>,
        sale_repo: Arc<dyn SaleRepository>,
        terminal_repo: Arc<dyn TerminalRepository>,
    ) -> Self {
        Self {
            invoice_repo,
            fiscal_seq_repo,
            tax_rate_repo,
            sale_repo,
            terminal_repo,
        }
    }

    pub async fn execute(
        &self,
        cmd: GenerateInvoiceCommand,
    ) -> Result<InvoiceResponse, FiscalError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let sale_id = SaleId::from_uuid(cmd.sale_id);
        let invoice_type = InvoiceType::from_str(&cmd.invoice_type)?;

        // ── 1. Validate no duplicate invoice for this sale ──────────────
        if self.invoice_repo.find_by_sale_id(sale_id).await?.is_some() {
            return Err(FiscalError::InvoiceAlreadyExistsForSale(cmd.sale_id));
        }

        // ── 2. Credit notes require an original invoice reference ───────
        if invoice_type.is_credit_note() {
            return Err(FiscalError::OriginalInvoiceRequired);
        }

        // ── 3. Read the sale with items and payments ────────────────────
        let sale = self
            .sale_repo
            .find_by_id_with_details(sale_id)
            .await
            .map_err(|_| FiscalError::SaleNotFound(cmd.sale_id))?
            .ok_or(FiscalError::SaleNotFound(cmd.sale_id))?;

        // ── 4. Validate sale is in a billable state ─────────────────────
        //
        // POS:       SaleStatus::Completed
        // eCommerce: order_status in {Paid, Processing, Shipped, Delivered}
        //            (invoice is emitted at payment time, before delivery)
        let is_billable = match sale.sale_type() {
            SaleType::Pos => sale.status() == SaleStatus::Completed,
            SaleType::Online => {
                if let Some(order_status) = sale.order_status() {
                    matches!(
                        order_status,
                        OrderStatus::Paid
                            | OrderStatus::Processing
                            | OrderStatus::Shipped
                            | OrderStatus::Delivered
                    )
                } else {
                    false
                }
            }
        };

        if !is_billable {
            return Err(FiscalError::SaleNotCompleted(cmd.sale_id));
        }

        // ── 5. Resolve terminal ─────────────────────────────────────────
        //
        // POS:       terminal_id is required (from the command)
        // eCommerce: terminal_id is optional — auto-resolve from store
        let terminal_id = if let Some(tid) = cmd.terminal_id {
            TerminalId::from_uuid(tid)
        } else if sale.sale_type() == SaleType::Online {
            // Find first active terminal with a valid CAI in this store
            self.find_fiscal_terminal(store_id).await?
        } else {
            // POS sales must provide terminal_id
            return Err(FiscalError::NoActiveCai(cmd.store_id));
        };

        // ── 6. Read the terminal to get the active CAI ──────────────────
        let terminal = self
            .terminal_repo
            .find_by_id(terminal_id)
            .await
            .map_err(|_| FiscalError::NoActiveCai(terminal_id.into_uuid()))?
            .ok_or(FiscalError::NoActiveCai(terminal_id.into_uuid()))?;

        let cai = terminal
            .current_cai()
            .ok_or(FiscalError::NoActiveCai(terminal_id.into_uuid()))?;

        if cai.is_expired() {
            return Err(FiscalError::CaiExpired(terminal_id.into_uuid()));
        }

        // ── 7. Get the fiscal sequence and generate invoice number ──────
        let fiscal_seq = self
            .fiscal_seq_repo
            .find_active(store_id, terminal_id)
            .await?
            .ok_or(FiscalError::FiscalSequenceNotFound)?;

        let invoice_number = self
            .fiscal_seq_repo
            .increment_and_get(fiscal_seq.id())
            .await?;

        // ── 8. Load tax rates for reference ─────────────────────────────
        let _tax_rates = self.tax_rate_repo.find_active_by_store(store_id).await?;

        // ── 9. Build invoice lines from sale items ──────────────────────
        let invoice_id = crate::domain::value_objects::InvoiceId::new();
        let hundred = dec!(100);
        let mut lines = Vec::new();
        let mut subtotal = Decimal::ZERO;
        let mut exempt_amount = Decimal::ZERO;
        let mut taxable_amount_15 = Decimal::ZERO;
        let mut taxable_amount_18 = Decimal::ZERO;
        let mut tax_15 = Decimal::ZERO;
        let mut tax_18 = Decimal::ZERO;
        let mut total_discount = Decimal::ZERO;

        for (idx, item) in sale.items().iter().enumerate() {
            let line_subtotal = item.quantity() * item.unit_price();
            let line_discount = item.discount_amount();
            let taxable_base = line_subtotal - line_discount;

            subtotal += line_subtotal;
            total_discount += line_discount;

            // Determine tax type based on the item's tax_rate
            let item_tax_rate = item.tax_rate();
            let (tax_type, effective_rate, is_exempt) = if item_tax_rate == Decimal::ZERO {
                (TaxType::Exempt, Decimal::ZERO, true)
            } else if item_tax_rate > dec!(15) {
                (TaxType::Isv18, item_tax_rate, false)
            } else {
                (TaxType::Isv15, item_tax_rate, false)
            };

            let line_tax = if is_exempt {
                exempt_amount += taxable_base;
                Decimal::ZERO
            } else {
                let tax = taxable_base * effective_rate / hundred;
                match tax_type {
                    TaxType::Isv15 => {
                        taxable_amount_15 += taxable_base;
                        tax_15 += tax;
                    }
                    TaxType::Isv18 => {
                        taxable_amount_18 += taxable_base;
                        tax_18 += tax;
                    }
                    TaxType::Exempt => {}
                }
                tax
            };

            let line_total = taxable_base + line_tax;

            lines.push(InvoiceLine::create(
                invoice_id,
                (idx + 1) as i32,
                item.product_id().into_uuid(),
                item.variant_id().map(|v| v.into_uuid()),
                item.sku().to_string(),
                item.description().to_string(),
                item.quantity(),
                item.unit_of_measure().to_string(),
                item.unit_price(),
                line_discount,
                tax_type,
                effective_rate,
                line_tax,
                line_subtotal,
                line_total,
                is_exempt,
            ));
        }

        let total_tax = tax_15 + tax_18;
        let total = subtotal - total_discount + total_tax;

        // ── 10. Determine payment method from sale payments ─────────────
        let payment_method = sale
            .primary_payment_method()
            .map(|pm| pm.to_string())
            .unwrap_or_else(|| "cash".to_string());

        // ── 11. Build CAI metadata ──────────────────────────────────────
        let cai_number = cai.cai_number().as_str().to_string();
        let cai_expiry: DateTime<Utc> =
            Utc.from_utc_datetime(&cai.expiration_date().and_hms_opt(23, 59, 59).unwrap());
        let range_start = format!("{:08}", cai.range_start());
        let range_end = format!("{:08}", cai.range_end());

        // ── 12. Amount in words (simplified) ────────────────────────────
        let amount_in_words = format!("L. {:.2}", total);

        // ── 13. Create the invoice entity ───────────────────────────────
        let mut invoice = Invoice::create(
            invoice_number,
            store_id,
            terminal_id,
            sale_id,
            cai.id(),
            invoice_type,
            sale.customer_id(),
            cmd.customer_name,
            cmd.customer_rtn,
            cmd.customer_address,
            Currency::from_string(sale.currency().to_string()),
            subtotal,
            exempt_amount,
            taxable_amount_15,
            taxable_amount_18,
            tax_15,
            tax_18,
            total_tax,
            total_discount,
            total,
            amount_in_words,
            payment_method,
            cai_number,
            cai_expiry,
            range_start,
            range_end,
            None, // original_invoice_id
            lines,
        );

        // Override the auto-generated ID with our pre-computed one for line consistency
        invoice.set_id(invoice_id);

        // ── 14. Save the invoice with its lines ─────────────────────────
        self.invoice_repo.save(&invoice).await?;

        Ok(InvoiceResponse::from(invoice))
    }

    /// Finds the first active terminal with a valid CAI in the given store.
    /// Used for eCommerce orders that don't have a specific terminal assigned.
    async fn find_fiscal_terminal(&self, store_id: StoreId) -> Result<TerminalId, FiscalError> {
        let terminals = self
            .terminal_repo
            .find_by_store(store_id)
            .await
            .map_err(|_| FiscalError::NoActiveCai(store_id.into_uuid()))?;

        for terminal in &terminals {
            if terminal.is_active()
                && terminal
                    .current_cai()
                    .is_some_and(|cai| !cai.is_expired() && cai.remaining() > 0)
            {
                return Ok(terminal.id());
            }
        }

        Err(FiscalError::NoActiveCai(store_id.into_uuid()))
    }
}
