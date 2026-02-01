//! Get customer use case

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::CustomerResponse;
use crate::domain::repositories::CustomerRepository;
use crate::domain::value_objects::CustomerId;
use crate::SalesError;

/// Use case for retrieving a customer by ID
pub struct GetCustomerUseCase {
    customer_repo: Arc<dyn CustomerRepository>,
}

impl GetCustomerUseCase {
    pub fn new(customer_repo: Arc<dyn CustomerRepository>) -> Self {
        Self { customer_repo }
    }

    pub async fn execute(&self, customer_id: Uuid) -> Result<CustomerResponse, SalesError> {
        let id = CustomerId::from_uuid(customer_id);

        let customer = self
            .customer_repo
            .find_by_id(id)
            .await?
            .ok_or(SalesError::CustomerNotFound(customer_id))?;

        Ok(CustomerResponse::from(customer))
    }
}
