//! ConfirmTransaction - mark a pending manual transaction as succeeded.
//!
//! Used by a manager (`transactions:confirm`) after they verify the deposit
//! appeared in the bank statement (BAC, Ficohsa, Atlántida, …) or after the
//! delivery person reports the cash on delivery was collected.

use std::sync::Arc;

use crate::PaymentsError;
use crate::application::dtos::{ConfirmTransactionCommand, TransactionResponse};
use crate::domain::repositories::TransactionRepository;
use crate::domain::value_objects::TransactionId;
use identity::UserId;

pub struct ConfirmTransactionUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl ConfirmTransactionUseCase {
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(
        &self,
        cmd: ConfirmTransactionCommand,
    ) -> Result<TransactionResponse, PaymentsError> {
        let id = TransactionId::from_uuid(cmd.transaction_id);
        let mut tx = self
            .transaction_repo
            .find_by_id(id)
            .await?
            .ok_or(PaymentsError::TransactionNotFound(cmd.transaction_id))?;

        tx.confirm(UserId::from_uuid(cmd.confirmed_by_id), cmd.reference_number)?;
        self.transaction_repo.update(&tx).await?;

        Ok(TransactionResponse::from(tx))
    }
}
