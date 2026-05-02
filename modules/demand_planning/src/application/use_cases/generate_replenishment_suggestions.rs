//! GenerateReplenishmentSuggestionsUseCase — for every active reorder policy,
//! check if the on-hand stock has fallen below the trigger point and emit a
//! pending suggestion if so. Skips variants that already have a pending
//! suggestion to avoid duplicates.
//!
//! Trigger logic (v1):
//!     trigger = min_qty + safety_stock_qty
//!     available = current_stock - reserved
//!     if available <= trigger → recommended = max(0, max_qty - available)

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::DemandPlanningError;
use crate::domain::entities::ReplenishmentSuggestion;
use crate::domain::repositories::{
    DemandForecastRepository, ReorderPolicyRepository, ReplenishmentSuggestionRepository,
    StockSnapshotRepository,
};

pub struct GenerateReplenishmentSuggestionsUseCase {
    policies: Arc<dyn ReorderPolicyRepository>,
    snapshots: Arc<dyn StockSnapshotRepository>,
    forecasts: Arc<dyn DemandForecastRepository>,
    suggestions: Arc<dyn ReplenishmentSuggestionRepository>,
}

impl GenerateReplenishmentSuggestionsUseCase {
    pub fn new(
        policies: Arc<dyn ReorderPolicyRepository>,
        snapshots: Arc<dyn StockSnapshotRepository>,
        forecasts: Arc<dyn DemandForecastRepository>,
        suggestions: Arc<dyn ReplenishmentSuggestionRepository>,
    ) -> Self {
        Self {
            policies,
            snapshots,
            forecasts,
            suggestions,
        }
    }

    pub async fn execute(&self) -> Result<usize, DemandPlanningError> {
        let active = self.policies.list_active(None).await?;
        let mut written = 0usize;

        for policy in active {
            // Skip if there's already a pending suggestion outstanding.
            let already = self
                .suggestions
                .has_pending_for(policy.product_variant_id(), policy.store_id())
                .await?;
            if already {
                continue;
            }

            let snapshot = self
                .snapshots
                .snapshot(policy.product_variant_id(), policy.store_id())
                .await?
                .map(|s| s.available())
                .unwrap_or(Decimal::ZERO);

            let trigger = policy.min_qty() + policy.safety_stock_qty();
            if snapshot > trigger {
                continue;
            }

            let recommended = (policy.max_qty() - snapshot).max(Decimal::ZERO);
            if recommended <= Decimal::ZERO {
                continue;
            }

            let forecast_qty = self
                .forecasts
                .find_latest(policy.product_variant_id(), policy.store_id())
                .await?
                .map(|f| f.forecasted_qty())
                .unwrap_or(Decimal::ZERO);

            let suggestion = ReplenishmentSuggestion::create(
                policy.product_variant_id(),
                policy.store_id(),
                snapshot,
                forecast_qty,
                recommended,
                policy.preferred_vendor_id(),
            )?;
            self.suggestions.save(&suggestion).await?;
            written += 1;
        }

        Ok(written)
    }
}
