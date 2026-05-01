//! ReconcileManualPayments - bulk-confirm pending transactions against a
//! bank statement.
//!
//! For each `BankStatementEntry` we look up a pending transaction in the
//! same store with a matching `reference_number` and amount. Matches are
//! auto-confirmed; mismatches are returned so the operator can investigate.

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{
    BankStatementEntry, ReconcilePaymentsCommand, ReconciliationResponse, TransactionResponse,
};
use crate::domain::repositories::TransactionRepository;
use identity::{StoreId, UserId};

pub struct ReconcileManualPaymentsUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl ReconcileManualPaymentsUseCase {
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(
        &self,
        cmd: ReconcilePaymentsCommand,
    ) -> Result<ReconciliationResponse, PaymentsError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let confirmed_by = UserId::from_uuid(cmd.confirmed_by_id);

        let pending = self
            .transaction_repo
            .find_pending_for_reconciliation(store_id)
            .await?;

        let mut auto_confirmed: Vec<TransactionResponse> = Vec::new();
        let mut unmatched_references: Vec<String> = Vec::new();

        for entry in cmd.entries {
            match find_match(&pending, &entry) {
                Some(idx) => {
                    let mut tx = pending[idx].clone();
                    // `confirm` is idempotent against the manual-confirm
                    // checks; if it errors we surface the entry as unmatched
                    // rather than aborting the whole batch.
                    if tx
                        .confirm(confirmed_by, Some(entry.reference_number.clone()))
                        .is_err()
                    {
                        unmatched_references.push(entry.reference_number.clone());
                        continue;
                    }
                    self.transaction_repo.update(&tx).await?;
                    auto_confirmed.push(TransactionResponse::from(tx));
                }
                None => unmatched_references.push(entry.reference_number.clone()),
            }
        }

        Ok(ReconciliationResponse {
            matched_count: auto_confirmed.len() as i64,
            unmatched_count: unmatched_references.len() as i64,
            auto_confirmed,
            unmatched_references,
        })
    }
}

/// Naïve matching: same reference_number (case-insensitive) and same amount.
/// Anything more sophisticated (date windows, partial matches) belongs in a
/// future iteration.
fn find_match(
    pending: &[crate::domain::entities::Transaction],
    entry: &BankStatementEntry,
) -> Option<usize> {
    let target_ref = entry.reference_number.to_lowercase();
    pending.iter().position(|tx| {
        tx.amount() == entry.amount
            && tx
                .reference_number()
                .map(|r| r.to_lowercase() == target_ref)
                .unwrap_or(false)
    })
}
