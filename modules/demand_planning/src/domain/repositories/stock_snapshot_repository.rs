//! Read-only projection over `inventory_stock` so the suggestion generator
//! can compare on-hand quantities against reorder points.

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::DemandPlanningError;

#[derive(Debug, Clone, Copy)]
pub struct StockSnapshot {
    pub current_qty: Decimal,
    pub reserved_qty: Decimal,
}

impl StockSnapshot {
    pub fn available(&self) -> Decimal {
        self.current_qty - self.reserved_qty
    }
}

#[async_trait]
pub trait StockSnapshotRepository: Send + Sync {
    /// Return on-hand and reserved quantities for a (variant, store) tuple.
    /// Returns `None` if no row exists for that variant in the store yet.
    async fn snapshot(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<StockSnapshot>, DemandPlanningError>;
}
