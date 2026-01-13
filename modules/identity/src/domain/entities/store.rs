// Store entity - represents a physical store or e-commerce site

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::StoreId;

/// Store entity representing a physical store (POS) or e-commerce site
///
/// Stores are the multi-tenancy boundary for the system. Users can be
/// assigned to multiple stores with different roles in each.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    id: StoreId,
    name: String,
    address: String,
    is_ecommerce: bool,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Store {
    /// Creates a new Store with all fields specified
    pub fn new(
        id: StoreId,
        name: String,
        address: String,
        is_ecommerce: bool,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            address,
            is_ecommerce,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Creates a new active physical store (is_ecommerce defaults to false)
    pub fn create(name: String, address: String) -> Self {
        let now = Utc::now();
        Self {
            id: StoreId::new(),
            name,
            address,
            is_ecommerce: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new active e-commerce store
    pub fn create_ecommerce(name: String, address: String) -> Self {
        let now = Utc::now();
        Self {
            id: StoreId::new(),
            name,
            address,
            is_ecommerce: true,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    // Getters

    pub fn id(&self) -> &StoreId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn is_ecommerce(&self) -> bool {
        self.is_ecommerce
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // Setters / Mutators

    /// Updates the store name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Updates the store address
    pub fn set_address(&mut self, address: String) {
        self.address = address;
        self.updated_at = Utc::now();
    }

    /// Sets whether this is an e-commerce store
    pub fn set_ecommerce(&mut self, is_ecommerce: bool) {
        self.is_ecommerce = is_ecommerce;
        self.updated_at = Utc::now();
    }

    /// Activates the store
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivates the store
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }
}

impl PartialEq for Store {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Store {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_create_defaults_to_physical() {
        let store = Store::create("Main Store".to_string(), "123 Main St".to_string());

        assert_eq!(store.name(), "Main Store");
        assert_eq!(store.address(), "123 Main St");
        assert!(!store.is_ecommerce()); // Default is false (physical store)
        assert!(store.is_active());
    }

    #[test]
    fn test_store_create_ecommerce() {
        let store = Store::create_ecommerce("Online Shop".to_string(), "www.shop.com".to_string());

        assert_eq!(store.name(), "Online Shop");
        assert!(store.is_ecommerce());
        assert!(store.is_active());
    }

    #[test]
    fn test_store_set_name() {
        let mut store = Store::create("Old Name".to_string(), "Address".to_string());
        let original_updated = store.updated_at();

        std::thread::sleep(std::time::Duration::from_millis(10));
        store.set_name("New Name".to_string());

        assert_eq!(store.name(), "New Name");
        assert!(store.updated_at() > original_updated);
    }

    #[test]
    fn test_store_set_address() {
        let mut store = Store::create("Store".to_string(), "Old Address".to_string());
        store.set_address("New Address".to_string());
        assert_eq!(store.address(), "New Address");
    }

    #[test]
    fn test_store_set_ecommerce() {
        let mut store = Store::create("Store".to_string(), "Address".to_string());
        assert!(!store.is_ecommerce());

        store.set_ecommerce(true);
        assert!(store.is_ecommerce());

        store.set_ecommerce(false);
        assert!(!store.is_ecommerce());
    }

    #[test]
    fn test_store_activate_deactivate() {
        let mut store = Store::create("Store".to_string(), "Address".to_string());
        assert!(store.is_active());

        store.deactivate();
        assert!(!store.is_active());

        store.activate();
        assert!(store.is_active());
    }

    #[test]
    fn test_store_equality_by_id() {
        let store1 = Store::create("Store 1".to_string(), "Address 1".to_string());
        let store2 = Store::new(
            *store1.id(),
            "Different Name".to_string(),
            "Different Address".to_string(),
            true,
            false,
            Utc::now(),
            Utc::now(),
        );

        // Stores are equal if they have the same ID
        assert_eq!(store1, store2);
    }

    #[test]
    fn test_store_inequality_different_ids() {
        let store1 = Store::create("Store".to_string(), "Address".to_string());
        let store2 = Store::create("Store".to_string(), "Address".to_string());

        // Different IDs mean different stores
        assert_ne!(store1, store2);
    }
}
