//! Create customer use case

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::{CreateCustomerCommand, CustomerResponse};
use crate::domain::entities::Customer;
use crate::domain::repositories::CustomerRepository;
use crate::domain::value_objects::CustomerType;
use crate::SalesError;
use identity::StoreId;

/// Use case for creating a new customer
pub struct CreateCustomerUseCase {
    customer_repo: Arc<dyn CustomerRepository>,
}

impl CreateCustomerUseCase {
    pub fn new(customer_repo: Arc<dyn CustomerRepository>) -> Self {
        Self { customer_repo }
    }

    pub async fn execute(&self, cmd: CreateCustomerCommand) -> Result<CustomerResponse, SalesError> {
        let store_id = StoreId::from_uuid(cmd.store_id);
        let customer_type = CustomerType::from_str(&cmd.customer_type)
            .map_err(|_| SalesError::InvalidCustomerType)?;

        // Check if email is already in use
        if let Some(ref email) = cmd.email {
            if self.customer_repo.find_by_email(store_id, email).await?.is_some() {
                return Err(SalesError::DuplicateCustomerEmail(email.clone()));
            }
        }

        // Generate unique customer code
        let code = self.customer_repo.generate_customer_code(store_id).await?;

        // Create the customer with basic info
        let customer = Customer::create(
            store_id,
            code,
            cmd.first_name,
            cmd.last_name,
            customer_type,
        );

        // Save the customer
        self.customer_repo.save(&customer).await?;

        Ok(CustomerResponse::from(customer))
    }
}
