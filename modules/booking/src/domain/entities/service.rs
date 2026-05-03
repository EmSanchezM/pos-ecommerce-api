use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::BookingError;
use crate::domain::value_objects::ServiceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    id: ServiceId,
    store_id: Uuid,
    name: String,
    description: Option<String>,
    duration_minutes: i32,
    price: Decimal,
    buffer_minutes_before: i32,
    buffer_minutes_after: i32,
    requires_deposit: bool,
    deposit_amount: Option<Decimal>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Service {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store_id: Uuid,
        name: String,
        description: Option<String>,
        duration_minutes: i32,
        price: Decimal,
        buffer_minutes_before: i32,
        buffer_minutes_after: i32,
        requires_deposit: bool,
        deposit_amount: Option<Decimal>,
    ) -> Result<Self, BookingError> {
        if duration_minutes <= 0 {
            return Err(BookingError::InvalidDuration(duration_minutes));
        }
        if buffer_minutes_before < 0 || buffer_minutes_after < 0 {
            return Err(BookingError::Validation(
                "buffer minutes cannot be negative".to_string(),
            ));
        }
        if requires_deposit && deposit_amount.is_none() {
            return Err(BookingError::Validation(
                "requires_deposit=true requires deposit_amount".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: ServiceId::new(),
            store_id,
            name,
            description,
            duration_minutes,
            price,
            buffer_minutes_before,
            buffer_minutes_after,
            requires_deposit,
            deposit_amount,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ServiceId,
        store_id: Uuid,
        name: String,
        description: Option<String>,
        duration_minutes: i32,
        price: Decimal,
        buffer_minutes_before: i32,
        buffer_minutes_after: i32,
        requires_deposit: bool,
        deposit_amount: Option<Decimal>,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            name,
            description,
            duration_minutes,
            price,
            buffer_minutes_before,
            buffer_minutes_after,
            requires_deposit,
            deposit_amount,
            is_active,
            created_at,
            updated_at,
        }
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ServiceId {
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
    pub fn duration_minutes(&self) -> i32 {
        self.duration_minutes
    }
    pub fn price(&self) -> Decimal {
        self.price
    }
    pub fn buffer_minutes_before(&self) -> i32 {
        self.buffer_minutes_before
    }
    pub fn buffer_minutes_after(&self) -> i32 {
        self.buffer_minutes_after
    }
    pub fn requires_deposit(&self) -> bool {
        self.requires_deposit
    }
    pub fn deposit_amount(&self) -> Option<Decimal> {
        self.deposit_amount
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
