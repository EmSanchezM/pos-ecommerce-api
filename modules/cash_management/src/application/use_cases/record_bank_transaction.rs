//! RecordBankTransactionUseCase — manual entry of a single bank-statement line.
//! Updates the account's book balance in the same logical operation.
//!
//! Concurrency: we read the account, apply the delta in memory, persist the
//! transaction and the updated account. If two requests arrive concurrently
//! the optimistic lock on `bank_accounts.version` causes one to fail with
//! `AccountVersionConflict` — the caller retries.

use std::sync::Arc;

use uuid::Uuid;

use crate::CashManagementError;
use crate::application::dtos::RecordBankTransactionCommand;
use crate::domain::entities::BankTransaction;
use crate::domain::repositories::{BankAccountRepository, BankTransactionRepository};

pub struct RecordBankTransactionUseCase {
    accounts: Arc<dyn BankAccountRepository>,
    transactions: Arc<dyn BankTransactionRepository>,
}

impl RecordBankTransactionUseCase {
    pub fn new(
        accounts: Arc<dyn BankAccountRepository>,
        transactions: Arc<dyn BankTransactionRepository>,
    ) -> Self {
        Self {
            accounts,
            transactions,
        }
    }

    pub async fn execute(
        &self,
        cmd: RecordBankTransactionCommand,
        actor_id: Option<Uuid>,
    ) -> Result<BankTransaction, CashManagementError> {
        let mut account = self
            .accounts
            .find_by_id(cmd.bank_account_id)
            .await?
            .ok_or_else(|| {
                CashManagementError::BankAccountNotFound(cmd.bank_account_id.into_uuid())
            })?;

        let txn = BankTransaction::record(
            cmd.bank_account_id,
            cmd.txn_type,
            cmd.amount,
            cmd.reference,
            cmd.description,
            cmd.occurred_at,
            actor_id,
        )?;

        // Apply delta to the running book balance.
        account.apply_delta(cmd.amount);

        self.transactions.save(&txn).await?;
        self.accounts.update(&account).await?;
        Ok(txn)
    }
}
