//! ShippingZone - geographic area description.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::ShippingZoneId;
use identity::StoreId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingZone {
    id: ShippingZoneId,
    store_id: StoreId,
    name: String,
    countries: Vec<String>,
    states: Vec<String>,
    zip_codes: Vec<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ShippingZone {
    pub fn create(
        store_id: StoreId,
        name: String,
        countries: Vec<String>,
        states: Vec<String>,
        zip_codes: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ShippingZoneId::new(),
            store_id,
            name,
            countries,
            states,
            zip_codes,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ShippingZoneId,
        store_id: StoreId,
        name: String,
        countries: Vec<String>,
        states: Vec<String>,
        zip_codes: Vec<String>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            countries,
            states,
            zip_codes,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Returns true when the destination is inside this zone.
    /// Precedence: zip_codes (most specific) > states > countries.
    pub fn matches(&self, country: &str, state: &str, zip: Option<&str>) -> bool {
        if let Some(z) = zip
            && self.zip_codes.iter().any(|c| c.eq_ignore_ascii_case(z))
        {
            return true;
        }
        if !self.states.is_empty() && self.states.iter().any(|s| s.eq_ignore_ascii_case(state)) {
            return true;
        }
        // Fall back to country-only when no states/zips configured
        if self.states.is_empty()
            && self.zip_codes.is_empty()
            && self
                .countries
                .iter()
                .any(|c| c.eq_ignore_ascii_case(country))
        {
            return true;
        }
        // Or when country matches AND no more specific filter is configured
        if !self.countries.is_empty()
            && self
                .countries
                .iter()
                .any(|c| c.eq_ignore_ascii_case(country))
            && (self.states.is_empty() || self.states.iter().any(|s| s.eq_ignore_ascii_case(state)))
        {
            return true;
        }
        false
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.touch();
    }
    pub fn set_countries(&mut self, countries: Vec<String>) {
        self.countries = countries;
        self.touch();
    }
    pub fn set_states(&mut self, states: Vec<String>) {
        self.states = states;
        self.touch();
    }
    pub fn set_zip_codes(&mut self, zip_codes: Vec<String>) {
        self.zip_codes = zip_codes;
        self.touch();
    }
    pub fn activate(&mut self) {
        self.is_active = true;
        self.touch();
    }
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ShippingZoneId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn countries(&self) -> &[String] {
        &self.countries
    }
    pub fn states(&self) -> &[String] {
        &self.states
    }
    pub fn zip_codes(&self) -> &[String] {
        &self.zip_codes
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
}
