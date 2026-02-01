//! Customer command DTOs

use serde::Deserialize;
use uuid::Uuid;

/// Command to create a new customer
#[derive(Debug, Deserialize)]
pub struct CreateCustomerCommand {
    pub store_id: Uuid,
    pub customer_type: String,
    pub first_name: String,
    pub last_name: String,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub billing_address: Option<AddressInput>,
    pub notes: Option<String>,
}

/// Command to update an existing customer
#[derive(Debug, Deserialize)]
pub struct UpdateCustomerCommand {
    pub customer_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub billing_address: Option<AddressInput>,
    pub notes: Option<String>,
}

/// Address input for customer operations
#[derive(Debug, Clone, Deserialize)]
pub struct AddressInput {
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

/// Filter for listing customers
#[derive(Debug, Default, Deserialize)]
pub struct ListCustomersQuery {
    pub store_id: Option<Uuid>,
    pub search: Option<String>,
    pub customer_type: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
