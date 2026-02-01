//! Process payment use case

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::{ProcessPaymentCommand, SaleDetailResponse};
use crate::domain::entities::Payment;
use crate::domain::repositories::{SaleRepository, ShiftRepository};
use crate::domain::value_objects::{PaymentId, PaymentMethod, SaleId};
use crate::SalesError;

/// Use case for processing a payment
pub struct ProcessPaymentUseCase {
    sale_repo: Arc<dyn SaleRepository>,
    shift_repo: Arc<dyn ShiftRepository>,
}

impl ProcessPaymentUseCase {
    pub fn new(
        sale_repo: Arc<dyn SaleRepository>,
        shift_repo: Arc<dyn ShiftRepository>,
    ) -> Self {
        Self {
            sale_repo,
            shift_repo,
        }
    }

    pub async fn execute(&self, cmd: ProcessPaymentCommand) -> Result<SaleDetailResponse, SalesError> {
        let sale_id = SaleId::from_uuid(cmd.sale_id);
        let payment_method = PaymentMethod::from_str(&cmd.payment_method)
            .map_err(|_| SalesError::InvalidPaymentMethod)?;

        let mut sale = self
            .sale_repo
            .find_by_id_with_items(sale_id)
            .await?
            .ok_or(SalesError::SaleNotFound(cmd.sale_id))?;

        // Verify sale is in draft status
        if !sale.status().is_draft() {
            return Err(SalesError::SaleNotEditable);
        }

        // Verify the amount doesn't exceed amount due
        if cmd.amount > sale.amount_due() {
            return Err(SalesError::PaymentExceedsBalance);
        }

        // Calculate change for cash payments
        let (amount_tendered, change_given) = if payment_method == PaymentMethod::Cash {
            let tendered = cmd.amount_tendered.unwrap_or(cmd.amount);
            if tendered < cmd.amount {
                return Err(SalesError::InsufficientAmountTendered);
            }
            let change = tendered - cmd.amount;
            (Some(tendered), Some(change))
        } else {
            (None, None)
        };

        // Create the payment
        let payment = Payment::create(
            PaymentId::new(),
            sale_id,
            payment_method,
            cmd.amount,
            sale.currency(),
            cmd.reference,
            cmd.card_last_four,
            cmd.card_brand,
            amount_tendered,
            change_given,
            cmd.notes,
        );

        // Add payment to sale
        sale.add_payment(payment.clone())?;

        // Save the payment
        self.sale_repo.save_payment(&payment).await?;

        // Update sale
        self.sale_repo.update(&sale).await?;

        // Update shift sales totals if this is a POS sale
        if let Some(shift_id) = sale.shift_id() {
            if let Some(mut shift) = self.shift_repo.find_by_id(shift_id).await? {
                match payment_method {
                    PaymentMethod::Cash => shift.record_cash_sale(cmd.amount)?,
                    PaymentMethod::CreditCard | PaymentMethod::DebitCard => {
                        shift.record_card_sale(cmd.amount)?
                    }
                    _ => shift.record_other_sale(cmd.amount)?,
                }
                self.shift_repo.update(&shift).await?;
            }
        }

        Ok(SaleDetailResponse::from(sale))
    }
}
