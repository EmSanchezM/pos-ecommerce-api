//! LoyaltyProgram — per-store program configuration. `points_per_currency_unit`
//! is the conversion rate the auto-earn subscriber will use: at value `0.10`,
//! a sale of L 250 mints 25 points. `expiration_days = None` keeps points
//! forever.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::LoyaltyProgramId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyProgram {
    id: LoyaltyProgramId,
    store_id: Uuid,
    name: String,
    description: Option<String>,
    points_per_currency_unit: Decimal,
    expiration_days: Option<i32>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LoyaltyProgram {
    pub fn create(
        store_id: Uuid,
        name: impl Into<String>,
        description: Option<String>,
        points_per_currency_unit: Decimal,
        expiration_days: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: LoyaltyProgramId::new(),
            store_id,
            name: name.into(),
            description,
            points_per_currency_unit,
            expiration_days,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: LoyaltyProgramId,
        store_id: Uuid,
        name: String,
        description: Option<String>,
        points_per_currency_unit: Decimal,
        expiration_days: Option<i32>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            description,
            points_per_currency_unit,
            expiration_days,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> LoyaltyProgramId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn points_per_currency_unit(&self) -> Decimal {
        self.points_per_currency_unit
    }
    pub fn expiration_days(&self) -> Option<i32> {
        self.expiration_days
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
