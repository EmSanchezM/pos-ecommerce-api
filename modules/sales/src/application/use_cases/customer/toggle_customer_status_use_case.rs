//! Toggle customer status use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CustomerResponse;
use crate::domain::repositories::CustomerRepository;
use crate::domain::value_objects::CustomerId;
use crate::SalesError;

/// Use case for activating or deactivating a customer
pub struct ToggleCustomerStatusUseCase {
    customer_repo: Arc<dyn CustomerRepository>,
}

impl ToggleCustomerStatusUseCase {
    pub fn new(customer_repo: Arc<dyn CustomerRepository>) -> Self {
        Self { customer_repo }
    }

    /// Activate a customer
    pub async fn activate(&self, customer_id: Uuid) -> Result<CustomerResponse, SalesError> {
        self.toggle(customer_id, true).await
    }

    /// Deactivate a customer
    pub async fn deactivate(&self, customer_id: Uuid) -> Result<CustomerResponse, SalesError> {
        self.toggle(customer_id, false).await
    }

    async fn toggle(&self, customer_id: Uuid, activate: bool) -> Result<CustomerResponse, SalesError> {
        let id = CustomerId::from_uuid(customer_id);

        let mut customer = self
            .customer_repo
            .find_by_id(id)
            .await?
            .ok_or(SalesError::CustomerNotFound(customer_id))?;

        if activate {
            customer.activate();
        } else {
            customer.deactivate();
        }

        self.customer_repo.update(&customer).await?;

        Ok(CustomerResponse::from(customer))
    }
}
