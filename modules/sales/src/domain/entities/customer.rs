//! Customer entity - represents a customer in the sales system

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{CustomerId, CustomerType};
use crate::SalesError;
use identity::{StoreId, UserId};

/// Address embedded value object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Address {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

impl Address {
    pub fn new(
        line1: Option<String>,
        line2: Option<String>,
        city: Option<String>,
        state: Option<String>,
        postal_code: Option<String>,
        country: Option<String>,
    ) -> Self {
        Self {
            line1,
            line2,
            city,
            state,
            postal_code,
            country,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.line1.is_none()
            && self.line2.is_none()
            && self.city.is_none()
            && self.state.is_none()
            && self.postal_code.is_none()
            && self.country.is_none()
    }
}

/// Customer entity representing a buyer in the sales system.
///
/// Invariants:
/// - Code must be unique per store
/// - Email must be unique per store (if provided)
/// - First name must not be empty
/// - Last name must not be empty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    id: CustomerId,
    store_id: StoreId,
    customer_type: CustomerType,
    code: String,
    first_name: String,
    last_name: String,
    company_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    tax_id: Option<String>,
    billing_address: Address,
    user_id: Option<UserId>,
    is_active: bool,
    total_purchases: Decimal,
    purchase_count: i32,
    last_purchase_at: Option<DateTime<Utc>>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Customer {
    /// Creates a new Customer
    pub fn create(
        store_id: StoreId,
        code: String,
        first_name: String,
        last_name: String,
        customer_type: CustomerType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: CustomerId::new(),
            store_id,
            customer_type,
            code,
            first_name,
            last_name,
            company_name: None,
            email: None,
            phone: None,
            tax_id: None,
            billing_address: Address::default(),
            user_id: None,
            is_active: true,
            total_purchases: Decimal::ZERO,
            purchase_count: 0,
            last_purchase_at: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a Customer from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CustomerId,
        store_id: StoreId,
        customer_type: CustomerType,
        code: String,
        first_name: String,
        last_name: String,
        company_name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        tax_id: Option<String>,
        billing_address: Address,
        user_id: Option<UserId>,
        is_active: bool,
        total_purchases: Decimal,
        purchase_count: i32,
        last_purchase_at: Option<DateTime<Utc>>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            customer_type,
            code,
            first_name,
            last_name,
            company_name,
            email,
            phone,
            tax_id,
            billing_address,
            user_id,
            is_active,
            total_purchases,
            purchase_count,
            last_purchase_at,
            notes,
            created_at,
            updated_at,
        }
    }

    /// Activates the customer
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivates the customer
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Validates that the customer is active for operations
    pub fn validate_active(&self) -> Result<(), SalesError> {
        if !self.is_active {
            return Err(SalesError::CustomerNotActive(self.id.into_uuid()));
        }
        Ok(())
    }

    /// Records a purchase to update customer statistics
    pub fn record_purchase(&mut self, amount: Decimal) {
        self.total_purchases += amount;
        self.purchase_count += 1;
        self.last_purchase_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Returns the customer's full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Returns the display name (company name for business, full name for individual)
    pub fn display_name(&self) -> String {
        if self.customer_type.is_business() {
            self.company_name
                .clone()
                .unwrap_or_else(|| self.full_name())
        } else {
            self.full_name()
        }
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> CustomerId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn customer_type(&self) -> CustomerType {
        self.customer_type
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn company_name(&self) -> Option<&str> {
        self.company_name.as_deref()
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    pub fn phone(&self) -> Option<&str> {
        self.phone.as_deref()
    }

    pub fn tax_id(&self) -> Option<&str> {
        self.tax_id.as_deref()
    }

    pub fn billing_address(&self) -> &Address {
        &self.billing_address
    }

    pub fn user_id(&self) -> Option<UserId> {
        self.user_id
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn total_purchases(&self) -> Decimal {
        self.total_purchases
    }

    pub fn purchase_count(&self) -> i32 {
        self.purchase_count
    }

    pub fn last_purchase_at(&self) -> Option<DateTime<Utc>> {
        self.last_purchase_at
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_customer_type(&mut self, customer_type: CustomerType) {
        self.customer_type = customer_type;
        self.updated_at = Utc::now();
    }

    pub fn set_first_name(&mut self, first_name: String) {
        self.first_name = first_name;
        self.updated_at = Utc::now();
    }

    pub fn set_last_name(&mut self, last_name: String) {
        self.last_name = last_name;
        self.updated_at = Utc::now();
    }

    pub fn set_company_name(&mut self, company_name: Option<String>) {
        self.company_name = company_name;
        self.updated_at = Utc::now();
    }

    pub fn set_email(&mut self, email: Option<String>) {
        self.email = email;
        self.updated_at = Utc::now();
    }

    pub fn set_phone(&mut self, phone: Option<String>) {
        self.phone = phone;
        self.updated_at = Utc::now();
    }

    pub fn set_tax_id(&mut self, tax_id: Option<String>) {
        self.tax_id = tax_id;
        self.updated_at = Utc::now();
    }

    pub fn set_billing_address(&mut self, address: Address) {
        self.billing_address = address;
        self.updated_at = Utc::now();
    }

    pub fn set_user_id(&mut self, user_id: Option<UserId>) {
        self.user_id = user_id;
        self.updated_at = Utc::now();
    }

    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_customer() {
        let customer = Customer::create(
            StoreId::new(),
            "CUS-001".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            CustomerType::Individual,
        );

        assert_eq!(customer.code(), "CUS-001");
        assert_eq!(customer.first_name(), "John");
        assert_eq!(customer.last_name(), "Doe");
        assert_eq!(customer.full_name(), "John Doe");
        assert!(customer.is_active());
        assert_eq!(customer.total_purchases(), Decimal::ZERO);
        assert_eq!(customer.purchase_count(), 0);
    }

    #[test]
    fn test_activate_deactivate() {
        let mut customer = Customer::create(
            StoreId::new(),
            "CUS-001".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            CustomerType::Individual,
        );

        customer.deactivate();
        assert!(!customer.is_active());

        customer.activate();
        assert!(customer.is_active());
    }

    #[test]
    fn test_validate_active() {
        let mut customer = Customer::create(
            StoreId::new(),
            "CUS-001".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            CustomerType::Individual,
        );

        assert!(customer.validate_active().is_ok());

        customer.deactivate();
        assert!(matches!(
            customer.validate_active(),
            Err(SalesError::CustomerNotActive(_))
        ));
    }

    #[test]
    fn test_record_purchase() {
        let mut customer = Customer::create(
            StoreId::new(),
            "CUS-001".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            CustomerType::Individual,
        );

        customer.record_purchase(dec!(100.00));
        assert_eq!(customer.total_purchases(), dec!(100.00));
        assert_eq!(customer.purchase_count(), 1);
        assert!(customer.last_purchase_at().is_some());

        customer.record_purchase(dec!(50.00));
        assert_eq!(customer.total_purchases(), dec!(150.00));
        assert_eq!(customer.purchase_count(), 2);
    }

    #[test]
    fn test_display_name() {
        let mut individual = Customer::create(
            StoreId::new(),
            "CUS-001".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            CustomerType::Individual,
        );
        assert_eq!(individual.display_name(), "John Doe");

        let mut business = Customer::create(
            StoreId::new(),
            "CUS-002".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            CustomerType::Business,
        );
        business.set_company_name(Some("Acme Corp".to_string()));
        assert_eq!(business.display_name(), "Acme Corp");
    }
}
