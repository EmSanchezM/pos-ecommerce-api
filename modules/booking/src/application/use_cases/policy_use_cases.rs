use std::sync::Arc;

use uuid::Uuid;

use crate::BookingError;
use crate::application::dtos::UpsertBookingPolicyCommand;
use crate::domain::entities::BookingPolicy;
use crate::domain::repositories::BookingPolicyRepository;

pub struct UpsertBookingPolicyUseCase {
    policies: Arc<dyn BookingPolicyRepository>,
}

impl UpsertBookingPolicyUseCase {
    pub fn new(policies: Arc<dyn BookingPolicyRepository>) -> Self {
        Self { policies }
    }

    pub async fn execute(
        &self,
        cmd: UpsertBookingPolicyCommand,
    ) -> Result<BookingPolicy, BookingError> {
        let policy = BookingPolicy::new(
            cmd.store_id,
            cmd.requires_deposit,
            cmd.deposit_percentage,
            cmd.cancellation_window_hours,
            cmd.no_show_fee_amount,
            cmd.default_buffer_minutes,
            cmd.advance_booking_days_max,
        )?;
        self.policies.upsert(&policy).await?;
        Ok(policy)
    }
}

pub struct GetBookingPolicyUseCase {
    policies: Arc<dyn BookingPolicyRepository>,
}

impl GetBookingPolicyUseCase {
    pub fn new(policies: Arc<dyn BookingPolicyRepository>) -> Self {
        Self { policies }
    }

    pub async fn execute(&self, store_id: Uuid) -> Result<BookingPolicy, BookingError> {
        self.policies
            .find_by_store(store_id)
            .await?
            .ok_or(BookingError::PolicyNotFound(store_id))
    }
}
