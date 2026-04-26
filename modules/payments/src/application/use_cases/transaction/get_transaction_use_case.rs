//! Read a single transaction.

use std::sync::Arc;

use uuid::Uuid;

use crate::PaymentsError;
use crate::application::dtos::TransactionResponse;
use crate::domain::repositories::TransactionRepository;
use crate::domain::value_objects::TransactionId;

pub struct GetTransactionUseCase {
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl GetTransactionUseCase {
    pub fn new(transaction_repo: Arc<dyn TransactionRepository>) -> Self {
        Self { transaction_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<TransactionResponse, PaymentsError> {
        let tx = self
            .transaction_repo
            .find_by_id(TransactionId::from_uuid(id))
            .await?
            .ok_or(PaymentsError::TransactionNotFound(id))?;
        Ok(TransactionResponse::from(tx))
    }
}
