//! LoyaltyMember — links a `customer` to a `program`. Caches running totals
//! (`current_balance`, `lifetime_points`) so member lookups don't have to
//! aggregate the ledger every time. The Pg implementation updates these
//! atomically with the ledger insert via `UPDATE ... SET balance = balance + $`
//! to avoid read-then-write races between concurrent earn/redeem calls.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{LoyaltyMemberId, LoyaltyProgramId, MemberTierId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyMember {
    id: LoyaltyMemberId,
    program_id: LoyaltyProgramId,
    customer_id: Uuid,
    current_tier_id: Option<MemberTierId>,
    /// Sum of unredeemed and unexpired points (signed sum of ledger entries).
    current_balance: i64,
    /// Cumulative earn (excludes redemptions / expirations) — drives tier.
    lifetime_points: i64,
    enrolled_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LoyaltyMember {
    pub fn enroll(program_id: LoyaltyProgramId, customer_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: LoyaltyMemberId::new(),
            program_id,
            customer_id,
            current_tier_id: None,
            current_balance: 0,
            lifetime_points: 0,
            enrolled_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: LoyaltyMemberId,
        program_id: LoyaltyProgramId,
        customer_id: Uuid,
        current_tier_id: Option<MemberTierId>,
        current_balance: i64,
        lifetime_points: i64,
        enrolled_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            program_id,
            customer_id,
            current_tier_id,
            current_balance,
            lifetime_points,
            enrolled_at,
            updated_at,
        }
    }

    pub fn set_tier(&mut self, tier_id: MemberTierId) {
        self.current_tier_id = Some(tier_id);
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> LoyaltyMemberId {
        self.id
    }
    pub fn program_id(&self) -> LoyaltyProgramId {
        self.program_id
    }
    pub fn customer_id(&self) -> Uuid {
        self.customer_id
    }
    pub fn current_tier_id(&self) -> Option<MemberTierId> {
        self.current_tier_id
    }
    pub fn current_balance(&self) -> i64 {
        self.current_balance
    }
    pub fn lifetime_points(&self) -> i64 {
        self.lifetime_points
    }
    pub fn enrolled_at(&self) -> DateTime<Utc> {
        self.enrolled_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
