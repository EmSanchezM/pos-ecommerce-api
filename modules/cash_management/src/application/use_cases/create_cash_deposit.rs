//! CreateCashDepositUseCase — opens a `pending` deposit linked to a closed
//! `cashier_shift`. The shift must already be in `closed` status (otherwise
//! cash counts aren't final yet). One deposit per shift; we reject duplicates.

use std::sync::Arc;

use sqlx::PgPool;

use crate::CashManagementError;
use crate::application::dtos::CreateCashDepositCommand;
use crate::domain::entities::CashDeposit;
use crate::domain::repositories::{BankAccountRepository, CashDepositRepository};

pub struct CreateCashDepositUseCase {
    accounts: Arc<dyn BankAccountRepository>,
    deposits: Arc<dyn CashDepositRepository>,
    pool: PgPool,
}

impl CreateCashDepositUseCase {
    pub fn new(
        accounts: Arc<dyn BankAccountRepository>,
        deposits: Arc<dyn CashDepositRepository>,
        pool: PgPool,
    ) -> Self {
        Self {
            accounts,
            deposits,
            pool,
        }
    }

    pub async fn execute(
        &self,
        cmd: CreateCashDepositCommand,
    ) -> Result<CashDeposit, CashManagementError> {
        // Bank account must exist (and ideally be active — but we allow
        // creating a deposit against a deactivated account so historical
        // shifts can still be settled).
        let _account = self
            .accounts
            .find_by_id(cmd.bank_account_id)
            .await?
            .ok_or_else(|| {
                CashManagementError::BankAccountNotFound(cmd.bank_account_id.into_uuid())
            })?;

        // Validate shift exists and is closed by checking the row directly —
        // crossing the modules::sales boundary via SQL is the lightest path
        // since cash_management doesn't otherwise depend on sales.
        let row: Option<(String,)> =
            sqlx::query_as("SELECT status FROM cashier_shifts WHERE id = $1")
                .bind(cmd.cashier_shift_id)
                .fetch_optional(&self.pool)
                .await?;
        let status = row
            .map(|(s,)| s)
            .ok_or(CashManagementError::ShiftNotFound(cmd.cashier_shift_id))?;
        if status != "closed" {
            return Err(CashManagementError::ShiftNotClosed(cmd.cashier_shift_id));
        }

        // One deposit per shift.
        if self
            .deposits
            .find_by_shift(cmd.cashier_shift_id)
            .await?
            .is_some()
        {
            return Err(CashManagementError::InvalidDepositTransition {
                from: "existing".into(),
                to: "pending".into(),
            });
        }

        let deposit = CashDeposit::create(
            cmd.cashier_shift_id,
            cmd.bank_account_id,
            cmd.amount,
            cmd.deposit_date,
            cmd.notes,
        )?;
        self.deposits.save(&deposit).await?;
        Ok(deposit)
    }
}
