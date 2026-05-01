//! PaymentGateway repository trait

use async_trait::async_trait;

use crate::PaymentsError;
use crate::domain::entities::PaymentGateway;
use crate::domain::value_objects::PaymentGatewayId;
use identity::StoreId;

#[async_trait]
pub trait PaymentGatewayRepository: Send + Sync {
    async fn save(&self, gateway: &PaymentGateway) -> Result<(), PaymentsError>;

    async fn find_by_id(
        &self,
        id: PaymentGatewayId,
    ) -> Result<Option<PaymentGateway>, PaymentsError>;

    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<PaymentGateway>, PaymentsError>;

    async fn find_default(
        &self,
        store_id: StoreId,
    ) -> Result<Option<PaymentGateway>, PaymentsError>;

    async fn update(&self, gateway: &PaymentGateway) -> Result<(), PaymentsError>;

    async fn delete(&self, id: PaymentGatewayId) -> Result<(), PaymentsError>;

    /// Clears the `is_default` flag for every gateway of `store_id` other
    /// than `keep`. Used by the use-case layer to enforce single-default.
    async fn unset_default_except(
        &self,
        store_id: StoreId,
        keep: PaymentGatewayId,
    ) -> Result<(), PaymentsError>;
}
