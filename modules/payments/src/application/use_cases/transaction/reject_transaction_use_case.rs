//! RejectTransaction - mark a pending manual transaction as failed.
//!
//! Used when the expected deposit never appears in the bank statement, or
//! when the delivery is refused (cash on delivery returned).

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{RejectTransactionCommand, TransactionResponse};
use crate::domain::repositories::TransactionRepository;
use crate::domain::value_objects::TransactionId;
use identity::UserId;

pub struct RejectTransactionUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl RejectTransactionUseCase {
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(
        &self,
        cmd: RejectTransactionCommand,
    ) -> Result<TransactionResponse, PaymentsError> {
        let id = TransactionId::from_uuid(cmd.transaction_id);
        let mut tx = self
            .transaction_repo
            .find_by_id(id)
            .await?
            .ok_or(PaymentsError::TransactionNotFound(cmd.transaction_id))?;

        tx.reject(UserId::from_uuid(cmd.rejected_by_id), cmd.reason)?;
        self.transaction_repo.update(&tx).await?;

        Ok(TransactionResponse::from(tx))
    }
}
