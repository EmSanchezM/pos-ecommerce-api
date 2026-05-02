//! ClassifyAbcUseCase — sorts variants by their revenue contribution over a
//! 90-day window and assigns class A (≤80% cumulative), B (≤95%), C (rest).
//! Designed for monthly execution from the recompute job.

use std::sync::Arc;

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

use crate::DemandPlanningError;
use crate::domain::entities::AbcClassification;
use crate::domain::repositories::{AbcClassificationRepository, SalesHistoryRepository};
use crate::domain::value_objects::AbcClass;

/// Window used for ABC analysis. Three months balances "captures the season"
/// against "doesn't follow noise".
const WINDOW_DAYS: i64 = 90;

/// Cumulative-revenue thresholds. A: 0..=80%, B: 80..=95%, C: 95..=100%.
const A_THRESHOLD: f64 = 0.80;
const B_THRESHOLD: f64 = 0.95;

pub struct ClassifyAbcUseCase {
    history: Arc<dyn SalesHistoryRepository>,
    repo: Arc<dyn AbcClassificationRepository>,
}

impl ClassifyAbcUseCase {
    pub fn new(
        history: Arc<dyn SalesHistoryRepository>,
        repo: Arc<dyn AbcClassificationRepository>,
    ) -> Self {
        Self { history, repo }
    }

    pub async fn execute(&self) -> Result<usize, DemandPlanningError> {
        let to = Utc::now().date_naive();
        let from = to - Duration::days(WINDOW_DAYS);

        let revenue_rows = self.history.revenue_by_variant(None, from, to).await?;
        if revenue_rows.is_empty() {
            return Ok(0);
        }

        // Group by store_id so each store gets its own A/B/C distribution.
        use std::collections::HashMap;
        let mut by_store: HashMap<uuid::Uuid, Vec<(uuid::Uuid, Decimal)>> = HashMap::new();
        for row in &revenue_rows {
            by_store
                .entry(row.store_id)
                .or_default()
                .push((row.product_variant_id, row.revenue));
        }

        let mut to_save: Vec<AbcClassification> = Vec::new();
        for (store_id, mut rows) in by_store {
            rows.sort_by(|a, b| b.1.cmp(&a.1));
            let total: Decimal = rows.iter().map(|(_, r)| *r).sum();
            if total <= Decimal::ZERO {
                continue;
            }
            let total_f = total.to_f64().unwrap_or(0.0);
            if total_f == 0.0 {
                continue;
            }

            let mut cumulative = 0.0_f64;
            for (variant_id, revenue) in rows {
                let revenue_f = revenue.to_f64().unwrap_or(0.0);
                let share_f = revenue_f / total_f;
                cumulative += share_f;
                let class = if cumulative <= A_THRESHOLD {
                    AbcClass::A
                } else if cumulative <= B_THRESHOLD {
                    AbcClass::B
                } else {
                    AbcClass::C
                };
                let share = revenue / total;
                to_save.push(AbcClassification::create(
                    variant_id, store_id, from, to, share, class,
                ));
            }
        }

        let written = to_save.len();
        if written > 0 {
            self.repo.save_batch(&to_save).await?;
        }
        Ok(written)
    }
}
