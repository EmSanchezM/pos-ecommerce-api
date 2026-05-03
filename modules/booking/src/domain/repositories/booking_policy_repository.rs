use async_trait::async_trait;
use uuid::Uuid;

use crate::BookingError;
use crate::domain::entities::BookingPolicy;

#[async_trait]
pub trait BookingPolicyRepository: Send + Sync {
    /// Insert-or-update on (store_id). One policy per store.
    async fn upsert(&self, policy: &BookingPolicy) -> Result<(), BookingError>;
    async fn find_by_store(&self, store_id: Uuid) -> Result<Option<BookingPolicy>, BookingError>;
}
