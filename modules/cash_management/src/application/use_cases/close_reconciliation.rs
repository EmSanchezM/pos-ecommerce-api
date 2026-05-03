//! CloseReconciliationUseCase — computes the book balance from recorded
//! transactions, compares it with the bank statement balance the caller
//! supplies, and closes the reconciliation. On success, every transaction
//! within the period is flagged reconciled against this reconciliation id.

use std::sync::Arc;
use uuid::Uuid;

use crate::CashManagementError;
use crate::application::dtos::CloseReconciliationCommand;
use crate::domain::entities::BankReconciliation;
use crate::domain::repositories::{BankReconciliationRepository, BankTransactionRepository};
use crate::domain::value_objects::BankReconciliationId;

pub struct CloseReconciliationUseCase {
    repo: Arc<dyn BankReconciliationRepository>,
    transactions: Arc<dyn BankTransactionRepository>,
}

impl CloseReconciliationUseCase {
    pub fn new(
        repo: Arc<dyn BankReconciliationRepository>,
        transactions: Arc<dyn BankTransactionRepository>,
    ) -> Self {
        Self { repo, transactions }
    }

    pub async fn execute(
        &self,
        id: BankReconciliationId,
        actor_id: Uuid,
        cmd: CloseReconciliationCommand,
    ) -> Result<BankReconciliation, CashManagementError> {
        let mut recon = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| CashManagementError::ReconciliationNotFound(id.into_uuid()))?;

        // Compute book balance: opening + sum(amounts) within the recon period.
        let book_balance = self
            .transactions
            .book_balance(
                recon.bank_account_id(),
                recon.opening_balance(),
                recon.period_start(),
                recon.period_end(),
            )
            .await?;

        recon.close(book_balance, cmd.statement_balance, actor_id, cmd.notes)?;
        self.repo.update(&recon).await?;

        // Flag transactions in the period as reconciled. Idempotent: rerunning
        // close is blocked by the status check, so this only runs once.
        self.transactions
            .mark_range_reconciled(
                recon.bank_account_id(),
                id.into_uuid(),
                recon.period_start(),
                recon.period_end(),
            )
            .await?;

        Ok(recon)
    }
}
