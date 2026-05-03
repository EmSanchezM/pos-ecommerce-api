use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::entities::{
    LoyaltyMember, LoyaltyProgram, MemberTier, PointsLedgerEntry, Reward, RewardRedemption,
};
use crate::domain::value_objects::{PointsTransactionType, RewardType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyProgramResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub points_per_currency_unit: Decimal,
    pub expiration_days: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&LoyaltyProgram> for LoyaltyProgramResponse {
    fn from(p: &LoyaltyProgram) -> Self {
        Self {
            id: p.id().into_uuid(),
            store_id: p.store_id(),
            name: p.name().to_string(),
            description: p.description().map(|s| s.to_string()),
            points_per_currency_unit: p.points_per_currency_unit(),
            expiration_days: p.expiration_days(),
            is_active: p.is_active(),
            created_at: p.created_at(),
            updated_at: p.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberTierResponse {
    pub id: Uuid,
    pub program_id: Uuid,
    pub name: String,
    pub threshold_points: i64,
    pub benefits: JsonValue,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&MemberTier> for MemberTierResponse {
    fn from(t: &MemberTier) -> Self {
        Self {
            id: t.id().into_uuid(),
            program_id: t.program_id().into_uuid(),
            name: t.name().to_string(),
            threshold_points: t.threshold_points(),
            benefits: t.benefits().clone(),
            sort_order: t.sort_order(),
            is_active: t.is_active(),
            created_at: t.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyMemberResponse {
    pub id: Uuid,
    pub program_id: Uuid,
    pub customer_id: Uuid,
    pub current_tier_id: Option<Uuid>,
    pub current_balance: i64,
    pub lifetime_points: i64,
    pub enrolled_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&LoyaltyMember> for LoyaltyMemberResponse {
    fn from(m: &LoyaltyMember) -> Self {
        Self {
            id: m.id().into_uuid(),
            program_id: m.program_id().into_uuid(),
            customer_id: m.customer_id(),
            current_tier_id: m.current_tier_id().map(|t| t.into_uuid()),
            current_balance: m.current_balance(),
            lifetime_points: m.lifetime_points(),
            enrolled_at: m.enrolled_at(),
            updated_at: m.updated_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsLedgerEntryResponse {
    pub id: Uuid,
    pub member_id: Uuid,
    pub txn_type: PointsTransactionType,
    pub points: i64,
    pub balance_after: i64,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
    pub created_by: Option<Uuid>,
}

impl From<&PointsLedgerEntry> for PointsLedgerEntryResponse {
    fn from(e: &PointsLedgerEntry) -> Self {
        Self {
            id: e.id().into_uuid(),
            member_id: e.member_id().into_uuid(),
            txn_type: e.txn_type(),
            points: e.points(),
            balance_after: e.balance_after(),
            source_type: e.source_type().map(|s| s.to_string()),
            source_id: e.source_id(),
            occurred_at: e.occurred_at(),
            expires_at: e.expires_at(),
            reason: e.reason().map(|s| s.to_string()),
            created_by: e.created_by(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardResponse {
    pub id: Uuid,
    pub program_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cost_points: i64,
    pub reward_type: RewardType,
    pub reward_value: Decimal,
    pub max_redemptions_per_member: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&Reward> for RewardResponse {
    fn from(r: &Reward) -> Self {
        Self {
            id: r.id().into_uuid(),
            program_id: r.program_id().into_uuid(),
            name: r.name().to_string(),
            description: r.description().map(|s| s.to_string()),
            cost_points: r.cost_points(),
            reward_type: r.reward_type(),
            reward_value: r.reward_value(),
            max_redemptions_per_member: r.max_redemptions_per_member(),
            is_active: r.is_active(),
            created_at: r.created_at(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRedemptionResponse {
    pub id: Uuid,
    pub member_id: Uuid,
    pub reward_id: Uuid,
    pub ledger_entry_id: Uuid,
    pub applied_to_sale_id: Option<Uuid>,
    pub redeemed_at: DateTime<Utc>,
}

impl From<&RewardRedemption> for RewardRedemptionResponse {
    fn from(r: &RewardRedemption) -> Self {
        Self {
            id: r.id().into_uuid(),
            member_id: r.member_id().into_uuid(),
            reward_id: r.reward_id().into_uuid(),
            ledger_entry_id: r.ledger_entry_id().into_uuid(),
            applied_to_sale_id: r.applied_to_sale_id(),
            redeemed_at: r.redeemed_at(),
        }
    }
}
