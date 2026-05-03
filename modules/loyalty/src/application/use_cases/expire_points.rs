//! ExpirePointsUseCase — job-style. Walks every member with earn lots that
//! have passed their `expires_at` and have unredeemed-unexpired surplus, and
//! posts an `Expire` ledger entry that zeros the surplus. Designed to run
//! nightly.
//!
//! v1 uses a coarse "expire what can't already be covered by redeem+expire"
//! computation: for each member we compare lifetime_points (earn) minus
//! lifetime_redeem minus lifetime_expire to the still-unexpired earn lots;
//! the actual SQL aggregation lives in
//! `PointsLedgerRepository::find_expirable_earns`.

use std::sync::Arc;

use chrono::Utc;

use crate::LoyaltyError;
use crate::domain::repositories::PointsLedgerRepository;

#[derive(Debug, Clone, Copy, Default)]
pub struct ExpirePointsResult {
    pub members_processed: u64,
    pub points_expired: i64,
}

pub struct ExpirePointsUseCase {
    ledger: Arc<dyn PointsLedgerRepository>,
}

impl ExpirePointsUseCase {
    pub fn new(ledger: Arc<dyn PointsLedgerRepository>) -> Self {
        Self { ledger }
    }

    pub async fn execute(&self) -> Result<ExpirePointsResult, LoyaltyError> {
        let lots = self.ledger.find_expirable_earns(Utc::now()).await?;
        let mut result = ExpirePointsResult::default();
        for lot in lots {
            if lot.remaining_points <= 0 {
                continue;
            }
            self.ledger
                .post_expire(
                    lot.member_id,
                    lot.remaining_points,
                    Some(format!(
                        "Auto-expiration of {} points earned before {}",
                        lot.remaining_points, lot.expires_at
                    )),
                )
                .await?;
            result.members_processed += 1;
            result.points_expired += lot.remaining_points;
        }
        Ok(result)
    }
}
