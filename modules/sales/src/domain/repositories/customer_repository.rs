//! Customer repository trait

use async_trait::async_trait;

use crate::domain::entities::Customer;
use crate::domain::value_objects::CustomerId;
use crate::SalesError;
use identity::StoreId;

/// Filter for querying customers
#[derive(Debug, Clone, Default)]
pub struct CustomerFilter {
    pub store_id: Option<StoreId>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

/// Repository trait for Customer persistence
#[async_trait]
pub trait CustomerRepository: Send + Sync {
    /// Saves a new customer
    async fn save(&self, customer: &Customer) -> Result<(), SalesError>;

    /// Finds a customer by ID
    async fn find_by_id(&self, id: CustomerId) -> Result<Option<Customer>, SalesError>;

    /// Finds a customer by code within a store
    async fn find_by_code(
        &self,
        store_id: StoreId,
        code: &str,
    ) -> Result<Option<Customer>, SalesError>;

    /// Finds a customer by email within a store
    async fn find_by_email(
        &self,
        store_id: StoreId,
        email: &str,
    ) -> Result<Option<Customer>, SalesError>;

    /// Updates an existing customer
    async fn update(&self, customer: &Customer) -> Result<(), SalesError>;

    /// Finds customers with pagination
    async fn find_paginated(
        &self,
        filter: CustomerFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Customer>, i64), SalesError>;

    /// Generates a unique customer code for a store
    async fn generate_customer_code(&self, store_id: StoreId) -> Result<String, SalesError>;
}
