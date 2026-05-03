//! RewardRedemption — voucher row created when a member spends points on a
//! reward. Pairs the ledger row that performed the deduction with the catalog
//! row that was bought, and optionally tracks which sale eventually consumed
//! the voucher (set later by the storefront/POS — `None` until then).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{
    LoyaltyMemberId, PointsLedgerEntryId, RewardId, RewardRedemptionId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRedemption {
    id: RewardRedemptionId,
    member_id: LoyaltyMemberId,
    reward_id: RewardId,
    ledger_entry_id: PointsLedgerEntryId,
    /// The sale this voucher was applied to. `None` until the POS uses it.
    applied_to_sale_id: Option<Uuid>,
    redeemed_at: DateTime<Utc>,
}

impl RewardRedemption {
    pub fn create(
        member_id: LoyaltyMemberId,
        reward_id: RewardId,
        ledger_entry_id: PointsLedgerEntryId,
    ) -> Self {
        Self {
            id: RewardRedemptionId::new(),
            member_id,
            reward_id,
            ledger_entry_id,
            applied_to_sale_id: None,
            redeemed_at: Utc::now(),
        }
    }

    pub fn reconstitute(
        id: RewardRedemptionId,
        member_id: LoyaltyMemberId,
        reward_id: RewardId,
        ledger_entry_id: PointsLedgerEntryId,
        applied_to_sale_id: Option<Uuid>,
        redeemed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            member_id,
            reward_id,
            ledger_entry_id,
            applied_to_sale_id,
            redeemed_at,
        }
    }

    pub fn id(&self) -> RewardRedemptionId {
        self.id
    }
    pub fn member_id(&self) -> LoyaltyMemberId {
        self.member_id
    }
    pub fn reward_id(&self) -> RewardId {
        self.reward_id
    }
    pub fn ledger_entry_id(&self) -> PointsLedgerEntryId {
        self.ledger_entry_id
    }
    pub fn applied_to_sale_id(&self) -> Option<Uuid> {
        self.applied_to_sale_id
    }
    pub fn redeemed_at(&self) -> DateTime<Utc> {
        self.redeemed_at
    }
}
