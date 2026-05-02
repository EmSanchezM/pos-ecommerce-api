//! Periodic recompute of demand forecasts + replenishment suggestions.
//!
//! Production deployments should run this nightly. The default interval here
//! is 24 hours; the `DEMAND_PLANNING_RECOMPUTE_INTERVAL_SECS` env var
//! overrides it. The job runs three steps in sequence:
//!
//!   1. `RecomputeForecastUseCase` — refresh per-(variant, store) forecasts.
//!   2. `GenerateReplenishmentSuggestionsUseCase` — emit pending suggestions
//!      where stock has fallen below the trigger.
//!   3. `ClassifyAbcUseCase` — Pareto classification (cheap; runs every tick
//!      but the table only changes at the monthly boundary).

use std::sync::Arc;
use std::time::Duration;

use demand_planning::{
    AbcClassificationRepository, ClassifyAbcUseCase, DemandForecastRepository,
    GenerateReplenishmentSuggestionsUseCase, RecomputeForecastUseCase, ReorderPolicyRepository,
    ReplenishmentSuggestionRepository, SalesHistoryRepository, StockSnapshotRepository,
};

#[allow(clippy::too_many_arguments)]
pub fn spawn(
    history: Arc<dyn SalesHistoryRepository>,
    forecasts: Arc<dyn DemandForecastRepository>,
    policies: Arc<dyn ReorderPolicyRepository>,
    snapshots: Arc<dyn StockSnapshotRepository>,
    suggestions: Arc<dyn ReplenishmentSuggestionRepository>,
    abc: Arc<dyn AbcClassificationRepository>,
    interval_secs: u64,
) {
    let recompute = RecomputeForecastUseCase::new(history.clone(), forecasts.clone());
    let generate = GenerateReplenishmentSuggestionsUseCase::new(
        policies.clone(),
        snapshots.clone(),
        forecasts.clone(),
        suggestions.clone(),
    );
    let classify = ClassifyAbcUseCase::new(history.clone(), abc.clone());

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        // Skip the immediate first tick to avoid running on startup.
        interval.tick().await;

        loop {
            interval.tick().await;
            match recompute.execute().await {
                Ok(written) => {
                    if written > 0 {
                        println!("[demand-planning] wrote {} forecast rows", written);
                    }
                }
                Err(e) => eprintln!("[demand-planning] recompute error: {}", e),
            }
            match generate.execute().await {
                Ok(written) => {
                    if written > 0 {
                        println!(
                            "[demand-planning] generated {} replenishment suggestions",
                            written
                        );
                    }
                }
                Err(e) => eprintln!("[demand-planning] suggestion error: {}", e),
            }
            match classify.execute().await {
                Ok(written) => {
                    if written > 0 {
                        println!("[demand-planning] classified {} variants (ABC)", written);
                    }
                }
                Err(e) => eprintln!("[demand-planning] abc error: {}", e),
            }
        }
    });
}
