//! Payout repository trait

use async_trait::async_trait;

use crate::PaymentsError;
use crate::domain::entities::Payout;
use crate::domain::value_objects::PayoutId;
use identity::StoreId;

#[async_trait]
pub trait PayoutRepository: Send + Sync {
    async fn save(&self, payout: &Payout) -> Result<(), PaymentsError>;

    async fn find_by_id(&self, id: PayoutId) -> Result<Option<Payout>, PaymentsError>;

    async fn find_by_store(
        &self,
        store_id: StoreId,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Payout>, i64), PaymentsError>;

    async fn update(&self, payout: &Payout) -> Result<(), PaymentsError>;
}
