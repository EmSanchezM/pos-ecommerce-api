use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::BookingError;
use crate::domain::value_objects::BookingPolicyId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingPolicy {
    id: BookingPolicyId,
    store_id: Uuid,
    requires_deposit: bool,
    deposit_percentage: Option<Decimal>,
    cancellation_window_hours: i32,
    no_show_fee_amount: Option<Decimal>,
    default_buffer_minutes: i32,
    advance_booking_days_max: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl BookingPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store_id: Uuid,
        requires_deposit: bool,
        deposit_percentage: Option<Decimal>,
        cancellation_window_hours: i32,
        no_show_fee_amount: Option<Decimal>,
        default_buffer_minutes: i32,
        advance_booking_days_max: i32,
    ) -> Result<Self, BookingError> {
        if cancellation_window_hours < 0 {
            return Err(BookingError::Validation(
                "cancellation_window_hours cannot be negative".to_string(),
            ));
        }
        if default_buffer_minutes < 0 {
            return Err(BookingError::Validation(
                "default_buffer_minutes cannot be negative".to_string(),
            ));
        }
        if advance_booking_days_max <= 0 {
            return Err(BookingError::Validation(
                "advance_booking_days_max must be positive".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: BookingPolicyId::new(),
            store_id,
            requires_deposit,
            deposit_percentage,
            cancellation_window_hours,
            no_show_fee_amount,
            default_buffer_minutes,
            advance_booking_days_max,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: BookingPolicyId,
        store_id: Uuid,
        requires_deposit: bool,
        deposit_percentage: Option<Decimal>,
        cancellation_window_hours: i32,
        no_show_fee_amount: Option<Decimal>,
        default_buffer_minutes: i32,
        advance_booking_days_max: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            requires_deposit,
            deposit_percentage,
            cancellation_window_hours,
            no_show_fee_amount,
            default_buffer_minutes,
            advance_booking_days_max,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> BookingPolicyId {
        self.id
    }
    pub fn store_id(&self) -> Uuid {
        self.store_id
    }
    pub fn requires_deposit(&self) -> bool {
        self.requires_deposit
    }
    pub fn deposit_percentage(&self) -> Option<Decimal> {
        self.deposit_percentage
    }
    pub fn cancellation_window_hours(&self) -> i32 {
        self.cancellation_window_hours
    }
    pub fn no_show_fee_amount(&self) -> Option<Decimal> {
        self.no_show_fee_amount
    }
    pub fn default_buffer_minutes(&self) -> i32 {
        self.default_buffer_minutes
    }
    pub fn advance_booking_days_max(&self) -> i32 {
        self.advance_booking_days_max
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
