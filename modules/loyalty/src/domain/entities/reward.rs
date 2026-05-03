//! Reward — catalog row describing what a member can redeem points for.
//! `reward_value` is the magnitude (e.g. `50` for L 50 off, `10` for 10 %).
//! Concrete redemptions live in `RewardRedemption`.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::LoyaltyError;
use crate::domain::value_objects::{LoyaltyProgramId, RewardId, RewardType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    id: RewardId,
    program_id: LoyaltyProgramId,
    name: String,
    description: Option<String>,
    cost_points: i64,
    reward_type: RewardType,
    reward_value: Decimal,
    /// Optional cap per member (NULL = unlimited).
    max_redemptions_per_member: Option<i32>,
    is_active: bool,
    created_at: DateTime<Utc>,
}

impl Reward {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        program_id: LoyaltyProgramId,
        name: impl Into<String>,
        description: Option<String>,
        cost_points: i64,
        reward_type: RewardType,
        reward_value: Decimal,
        max_redemptions_per_member: Option<i32>,
    ) -> Result<Self, LoyaltyError> {
        if cost_points <= 0 {
            return Err(LoyaltyError::NegativeAmount(cost_points));
        }
        Ok(Self {
            id: RewardId::new(),
            program_id,
            name: name.into(),
            description,
            cost_points,
            reward_type,
            reward_value,
            max_redemptions_per_member,
            is_active: true,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: RewardId,
        program_id: LoyaltyProgramId,
        name: String,
        description: Option<String>,
        cost_points: i64,
        reward_type: RewardType,
        reward_value: Decimal,
        max_redemptions_per_member: Option<i32>,
        is_active: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            program_id,
            name,
            description,
            cost_points,
            reward_type,
            reward_value,
            max_redemptions_per_member,
            is_active,
            created_at,
        }
    }

    pub fn id(&self) -> RewardId {
        self.id
    }
    pub fn program_id(&self) -> LoyaltyProgramId {
        self.program_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn cost_points(&self) -> i64 {
        self.cost_points
    }
    pub fn reward_type(&self) -> RewardType {
        self.reward_type
    }
    pub fn reward_value(&self) -> Decimal {
        self.reward_value
    }
    pub fn max_redemptions_per_member(&self) -> Option<i32> {
        self.max_redemptions_per_member
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
