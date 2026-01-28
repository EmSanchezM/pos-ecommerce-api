// Vendor entity - represents a supplier/vendor in the purchasing system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::VendorId;
use crate::PurchasingError;
use inventory::Currency;

/// Vendor entity representing a supplier in the purchasing system.
///
/// Invariants:
/// - Code must be unique
/// - Tax ID must be unique
/// - Name must not be empty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    id: VendorId,
    code: String,
    name: String,
    legal_name: String,
    tax_id: String,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    payment_terms_days: i32,
    currency: Currency,
    is_active: bool,
    notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Vendor {
    /// Creates a new Vendor
    pub fn create(
        code: String,
        name: String,
        legal_name: String,
        tax_id: String,
        currency: Currency,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: VendorId::new(),
            code,
            name,
            legal_name,
            tax_id,
            email: None,
            phone: None,
            address: None,
            payment_terms_days: 30,
            currency,
            is_active: true,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a Vendor from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: VendorId,
        code: String,
        name: String,
        legal_name: String,
        tax_id: String,
        email: Option<String>,
        phone: Option<String>,
        address: Option<String>,
        payment_terms_days: i32,
        currency: Currency,
        is_active: bool,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            code,
            name,
            legal_name,
            tax_id,
            email,
            phone,
            address,
            payment_terms_days,
            currency,
            is_active,
            notes,
            created_at,
            updated_at,
        }
    }

    /// Activates the vendor
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivates the vendor
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    /// Validates that the vendor is active for operations
    pub fn validate_active(&self) -> Result<(), PurchasingError> {
        if !self.is_active {
            return Err(PurchasingError::VendorNotActive(self.id.into_uuid()));
        }
        Ok(())
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> VendorId {
        self.id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn legal_name(&self) -> &str {
        &self.legal_name
    }

    pub fn tax_id(&self) -> &str {
        &self.tax_id
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    pub fn phone(&self) -> Option<&str> {
        self.phone.as_deref()
    }

    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    pub fn payment_terms_days(&self) -> i32 {
        self.payment_terms_days
    }

    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    pub fn is_active(&self) -> bool {
        self.is_active
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

    pub fn set_code(&mut self, code: String) {
        self.code = code;
        self.updated_at = Utc::now();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn set_legal_name(&mut self, legal_name: String) {
        self.legal_name = legal_name;
        self.updated_at = Utc::now();
    }

    pub fn set_tax_id(&mut self, tax_id: String) {
        self.tax_id = tax_id;
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

    pub fn set_address(&mut self, address: Option<String>) {
        self.address = address;
        self.updated_at = Utc::now();
    }

    pub fn set_payment_terms_days(&mut self, days: i32) {
        self.payment_terms_days = days;
        self.updated_at = Utc::now();
    }

    pub fn set_currency(&mut self, currency: Currency) {
        self.currency = currency;
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

    #[test]
    fn test_create_vendor() {
        let vendor = Vendor::create(
            "PROV-001".to_string(),
            "Proveedor Test".to_string(),
            "Proveedor Test S.A.".to_string(),
            "0801-1234-12345".to_string(),
            Currency::new("HNL").unwrap(),
        );

        assert_eq!(vendor.code(), "PROV-001");
        assert_eq!(vendor.name(), "Proveedor Test");
        assert_eq!(vendor.legal_name(), "Proveedor Test S.A.");
        assert_eq!(vendor.tax_id(), "0801-1234-12345");
        assert_eq!(vendor.payment_terms_days(), 30);
        assert!(vendor.is_active());
    }

    #[test]
    fn test_activate_deactivate() {
        let mut vendor = Vendor::create(
            "PROV-001".to_string(),
            "Test".to_string(),
            "Test S.A.".to_string(),
            "123".to_string(),
            Currency::new("HNL").unwrap(),
        );

        vendor.deactivate();
        assert!(!vendor.is_active());

        vendor.activate();
        assert!(vendor.is_active());
    }

    #[test]
    fn test_validate_active() {
        let mut vendor = Vendor::create(
            "PROV-001".to_string(),
            "Test".to_string(),
            "Test S.A.".to_string(),
            "123".to_string(),
            Currency::new("HNL").unwrap(),
        );

        assert!(vendor.validate_active().is_ok());

        vendor.deactivate();
        assert!(matches!(
            vendor.validate_active(),
            Err(PurchasingError::VendorNotActive(_))
        ));
    }
}
