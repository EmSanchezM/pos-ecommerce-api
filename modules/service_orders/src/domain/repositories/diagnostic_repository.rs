use async_trait::async_trait;

use crate::ServiceOrdersError;
use crate::domain::entities::Diagnostic;
use crate::domain::value_objects::ServiceOrderId;

#[async_trait]
pub trait DiagnosticRepository: Send + Sync {
    async fn save(&self, diagnostic: &Diagnostic) -> Result<(), ServiceOrdersError>;
    async fn list_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<Diagnostic>, ServiceOrdersError>;
}
