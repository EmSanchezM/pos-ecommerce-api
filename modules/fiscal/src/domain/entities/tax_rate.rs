//! TaxRate entity - represents a configurable tax rate for a store

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{TaxAppliesTo, TaxRateId, TaxType};
use identity::StoreId;

/// TaxRate entity representing a configurable tax rate for a store.
///
/// Invariants:
/// - Rate must be non-negative
/// - Only one default tax rate per store
/// - Category IDs are required when applies_to is Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRate {
    id: TaxRateId,
    store_id: StoreId,
    name: String,
    tax_type: TaxType,
    rate: Decimal,
    is_default: bool,
    is_active: bool,
    applies_to: TaxAppliesTo,
    category_ids: Vec<uuid::Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TaxRate {
    /// Creates a new TaxRate
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        name: String,
        tax_type: TaxType,
        rate: Decimal,
        is_default: bool,
        applies_to: TaxAppliesTo,
        category_ids: Vec<uuid::Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TaxRateId::new(),
            store_id,
            name,
            tax_type,
            rate,
            is_default,
            is_active: true,
            applies_to,
            category_ids,
            created_at: now,
            updated_at: now,
        }
    }

    /// Reconstitutes a TaxRate from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: TaxRateId,
        store_id: StoreId,
        name: String,
        tax_type: TaxType,
        rate: Decimal,
        is_default: bool,
        is_active: bool,
        applies_to: TaxAppliesTo,
        category_ids: Vec<uuid::Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            tax_type,
            rate,
            is_default,
            is_active,
            applies_to,
            category_ids,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // Setters / Mutation Methods
    // =========================================================================

    /// Updates the tax rate name
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    /// Updates the tax rate percentage
    pub fn set_rate(&mut self, rate: Decimal) {
        self.rate = rate;
        self.updated_at = Utc::now();
    }

    /// Sets this tax rate as the default for the store
    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
        self.updated_at = Utc::now();
    }

    /// Activates this tax rate
    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    /// Deactivates this tax rate
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> TaxRateId {
        self.id
    }

    pub fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tax_type(&self) -> TaxType {
        self.tax_type
    }

    pub fn rate(&self) -> Decimal {
        self.rate
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn applies_to(&self) -> TaxAppliesTo {
        self.applies_to
    }

    pub fn category_ids(&self) -> &[uuid::Uuid] {
        &self.category_ids
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
