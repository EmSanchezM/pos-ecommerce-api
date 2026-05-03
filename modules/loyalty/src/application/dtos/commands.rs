use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::domain::value_objects::{LoyaltyProgramId, RewardId, RewardType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLoyaltyProgramCommand {
    pub store_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Conversion rate from currency to points. Example: 1.0 → 1 pt per L 1.
    pub points_per_currency_unit: Decimal,
    /// Optional: points expire after this many days; `None` = never.
    pub expiration_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemberTierCommand {
    pub program_id: LoyaltyProgramId,
    pub name: String,
    pub threshold_points: i64,
    #[serde(default)]
    pub benefits: JsonValue,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollMemberCommand {
    pub program_id: LoyaltyProgramId,
    pub customer_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnPointsCommand {
    pub points: i64,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustPointsCommand {
    /// Signed delta — admins use this for corrections.
    pub points: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemRewardCommand {
    pub reward_id: RewardId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRewardCommand {
    pub program_id: LoyaltyProgramId,
    pub name: String,
    pub description: Option<String>,
    pub cost_points: i64,
    pub reward_type: RewardType,
    pub reward_value: Decimal,
    pub max_redemptions_per_member: Option<i32>,
}
