//! Get shift report use case

use std::sync::Arc;
use uuid::Uuid;

use rust_decimal::Decimal;

use crate::application::dtos::{SalesBreakdown, ShiftReportResponse, ShiftResponse};
use crate::domain::repositories::ShiftRepository;
use crate::domain::value_objects::ShiftId;
use crate::SalesError;

/// Use case for getting a detailed shift report
pub struct GetShiftReportUseCase {
    shift_repo: Arc<dyn ShiftRepository>,
}

impl GetShiftReportUseCase {
    pub fn new(shift_repo: Arc<dyn ShiftRepository>) -> Self {
        Self { shift_repo }
    }

    pub async fn execute(&self, shift_id: Uuid) -> Result<ShiftReportResponse, SalesError> {
        let id = ShiftId::from_uuid(shift_id);

        let shift = self
            .shift_repo
            .find_by_id(id)
            .await?
            .ok_or(SalesError::ShiftNotFound(shift_id))?;

        let net_sales = shift.total_sales() - shift.refunds();
        let average_transaction = if shift.transaction_count() > 0 {
            net_sales / Decimal::from(shift.transaction_count())
        } else {
            Decimal::ZERO
        };

        let sales_breakdown = SalesBreakdown {
            total_sales: shift.total_sales(),
            total_refunds: shift.refunds(),
            net_sales,
            transaction_count: shift.transaction_count(),
            average_transaction,
        };

        // Payment breakdown would normally come from analyzing sales data
        // For now, we return the aggregate data from the shift
        let payment_breakdown = vec![
            crate::application::dtos::PaymentBreakdownItem {
                payment_method: "cash".to_string(),
                amount: shift.cash_sales(),
                count: 0, // Would need to aggregate from sales
            },
            crate::application::dtos::PaymentBreakdownItem {
                payment_method: "card".to_string(),
                amount: shift.card_sales(),
                count: 0,
            },
            crate::application::dtos::PaymentBreakdownItem {
                payment_method: "other".to_string(),
                amount: shift.other_sales(),
                count: 0,
            },
        ];

        Ok(ShiftReportResponse {
            shift: ShiftResponse::from(shift),
            sales_breakdown,
            payment_breakdown,
        })
    }
}
