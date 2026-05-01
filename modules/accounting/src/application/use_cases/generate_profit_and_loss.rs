//! GenerateProfitAndLossUseCase — assembles a P&L for a period (optionally
//! scoped to a single store) from posted journal lines.

use std::sync::Arc;

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::AccountingError;
use crate::application::dtos::{ProfitAndLossLineResponse, ProfitAndLossResponse};
use crate::domain::repositories::AccountingReportRepository;
use crate::domain::value_objects::{AccountType, AccountingPeriodId};

pub struct GenerateProfitAndLossUseCase {
    repo: Arc<dyn AccountingReportRepository>,
}

impl GenerateProfitAndLossUseCase {
    pub fn new(repo: Arc<dyn AccountingReportRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        period_id: AccountingPeriodId,
        store_id: Option<Uuid>,
    ) -> Result<ProfitAndLossResponse, AccountingError> {
        let lines = self.repo.profit_and_loss(period_id, store_id).await?;

        let mut revenue = Vec::new();
        let mut expenses = Vec::new();
        let mut total_revenue = Decimal::ZERO;
        let mut total_expense = Decimal::ZERO;

        for line in &lines {
            let dto = ProfitAndLossLineResponse::from(line);
            match line.account_type {
                AccountType::Revenue => {
                    total_revenue += line.net_amount;
                    revenue.push(dto);
                }
                AccountType::Expense => {
                    total_expense += line.net_amount;
                    expenses.push(dto);
                }
                _ => {
                    // Defensive: the report repo already filters by P&L types,
                    // but if a non-P&L row sneaks in we ignore it rather than
                    // skewing totals.
                }
            }
        }

        Ok(ProfitAndLossResponse {
            period_id: period_id.into_uuid(),
            store_id,
            revenue,
            expenses,
            total_revenue,
            total_expense,
            net_income: total_revenue - total_expense,
        })
    }
}
