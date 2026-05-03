use async_trait::async_trait;

use crate::ServiceOrdersError;
use crate::domain::entities::Quote;
use crate::domain::value_objects::{QuoteId, ServiceOrderId};

#[async_trait]
pub trait QuoteRepository: Send + Sync {
    async fn save(&self, quote: &Quote) -> Result<(), ServiceOrdersError>;
    async fn update(&self, quote: &Quote) -> Result<(), ServiceOrdersError>;
    async fn find_by_id(&self, id: QuoteId) -> Result<Option<Quote>, ServiceOrdersError>;
    async fn list_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<Quote>, ServiceOrdersError>;
    /// Returns the highest version number used so far for this order
    /// (`MAX(version)`), or 0 if no quotes exist yet. Used by
    /// `CreateQuoteUseCase` to assign the next version number.
    async fn max_version_for_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<i32, ServiceOrdersError>;
    /// Atomically marks every Draft|Sent quote of `order_id` as `Superseded`,
    /// excluding `except_id`. Used right after persisting a new draft quote.
    async fn mark_others_superseded(
        &self,
        order_id: ServiceOrderId,
        except_id: QuoteId,
    ) -> Result<(), ServiceOrdersError>;
}
