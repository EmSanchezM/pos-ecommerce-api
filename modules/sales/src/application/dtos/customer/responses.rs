//! Customer response DTOs

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::{Address, Customer};

/// Response for a single customer
#[derive(Debug, Serialize)]
pub struct CustomerResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub customer_type: String,
    pub code: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub tax_id: Option<String>,
    pub billing_address: Option<AddressResponse>,
    pub is_active: bool,
    pub total_purchases: Decimal,
    pub purchase_count: i32,
    pub last_purchase_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Customer> for CustomerResponse {
    fn from(c: Customer) -> Self {
        Self {
            id: c.id().into_uuid(),
            store_id: c.store_id().into_uuid(),
            customer_type: c.customer_type().to_string(),
            code: c.code().to_string(),
            first_name: c.first_name().to_string(),
            last_name: c.last_name().to_string(),
            full_name: c.full_name(),
            company_name: c.company_name().map(String::from),
            email: c.email().map(String::from),
            phone: c.phone().map(String::from),
            tax_id: c.tax_id().map(String::from),
            billing_address: if c.billing_address().is_empty() {
                None
            } else {
                Some(AddressResponse::from(c.billing_address()))
            },
            is_active: c.is_active(),
            total_purchases: c.total_purchases(),
            purchase_count: c.purchase_count(),
            last_purchase_at: c.last_purchase_at(),
            notes: c.notes().map(String::from),
            created_at: c.created_at(),
            updated_at: c.updated_at(),
        }
    }
}

/// Response for an address
#[derive(Debug, Serialize)]
pub struct AddressResponse {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

impl From<&Address> for AddressResponse {
    fn from(a: &Address) -> Self {
        Self {
            line1: a.line1.clone(),
            line2: a.line2.clone(),
            city: a.city.clone(),
            state: a.state.clone(),
            postal_code: a.postal_code.clone(),
            country: a.country.clone(),
        }
    }
}

/// Paginated response for customer list
#[derive(Debug, Serialize)]
pub struct CustomerListResponse {
    pub data: Vec<CustomerResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
