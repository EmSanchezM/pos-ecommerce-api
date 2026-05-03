//! LinkDepositToTransactionUseCase — pairs a `deposited` `CashDeposit` with a
//! matching `BankTransaction` so both are flagged reconciled.
//!
//! Validations:
//!   * deposit and transaction belong to the same bank account
//!   * deposit amount equals transaction amount (positive — must be a deposit)
//!   * transaction isn't already linked to a different deposit
//!   * deposit is in `deposited` state (the previous step gathered the slip)

use std::sync::Arc;

use crate::CashManagementError;
use crate::application::dtos::LinkDepositCommand;
use crate::domain::entities::CashDeposit;
use crate::domain::repositories::{BankTransactionRepository, CashDepositRepository};
use crate::domain::value_objects::{BankTransactionId, CashDepositId};

pub struct LinkDepositToTransactionUseCase {
    deposits: Arc<dyn CashDepositRepository>,
    transactions: Arc<dyn BankTransactionRepository>,
}

impl LinkDepositToTransactionUseCase {
    pub fn new(
        deposits: Arc<dyn CashDepositRepository>,
        transactions: Arc<dyn BankTransactionRepository>,
    ) -> Self {
        Self {
            deposits,
            transactions,
        }
    }

    pub async fn execute(
        &self,
        deposit_id: CashDepositId,
        cmd: LinkDepositCommand,
    ) -> Result<CashDeposit, CashManagementError> {
        let mut deposit = self
            .deposits
            .find_by_id(deposit_id)
            .await?
            .ok_or_else(|| CashManagementError::CashDepositNotFound(deposit_id.into_uuid()))?;

        let txn_id = BankTransactionId::from_uuid(cmd.bank_transaction_id);
        let mut txn =
            self.transactions.find_by_id(txn_id).await?.ok_or_else(|| {
                CashManagementError::BankTransactionNotFound(cmd.bank_transaction_id)
            })?;

        if txn.bank_account_id() != deposit.bank_account_id() {
            return Err(CashManagementError::TransactionAccountMismatch);
        }
        if txn.amount() != deposit.amount() {
            return Err(CashManagementError::TransactionAmountMismatch {
                deposit: deposit.amount(),
                transaction: txn.amount(),
            });
        }
        if self.transactions.has_linked_deposit(txn_id).await? {
            return Err(CashManagementError::TransactionAlreadyLinked);
        }

        deposit.mark_reconciled(txn_id)?;
        txn.mark_reconciled(None);
        self.transactions.update(&txn).await?;
        self.deposits.update(&deposit).await?;
        Ok(deposit)
    }
}
