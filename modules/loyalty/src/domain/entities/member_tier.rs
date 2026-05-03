//! MemberTier — Bronze/Silver/Gold/etc. defined per program. The aggregate
//! enforces a non-negative threshold; tiers are sorted by `threshold_points`
//! ascending and a member sits in the highest tier whose threshold they've
//! met. `benefits_json` is opaque to this module — the storefront consumes it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::LoyaltyError;
use crate::domain::value_objects::{LoyaltyProgramId, MemberTierId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberTier {
    id: MemberTierId,
    program_id: LoyaltyProgramId,
    name: String,
    threshold_points: i64,
    benefits: JsonValue,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
}

impl MemberTier {
    pub fn create(
        program_id: LoyaltyProgramId,
        name: impl Into<String>,
        threshold_points: i64,
        benefits: JsonValue,
        sort_order: i32,
    ) -> Result<Self, LoyaltyError> {
        if threshold_points < 0 {
            return Err(LoyaltyError::NegativeThreshold(threshold_points));
        }
        Ok(Self {
            id: MemberTierId::new(),
            program_id,
            name: name.into(),
            threshold_points,
            benefits,
            sort_order,
            is_active: true,
            created_at: Utc::now(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: MemberTierId,
        program_id: LoyaltyProgramId,
        name: String,
        threshold_points: i64,
        benefits: JsonValue,
        sort_order: i32,
        is_active: bool,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            program_id,
            name,
            threshold_points,
            benefits,
            sort_order,
            is_active,
            created_at,
        }
    }

    pub fn id(&self) -> MemberTierId {
        self.id
    }
    pub fn program_id(&self) -> LoyaltyProgramId {
        self.program_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn threshold_points(&self) -> i64 {
        self.threshold_points
    }
    pub fn benefits(&self) -> &JsonValue {
        &self.benefits
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_rejects_negative_threshold() {
        let err =
            MemberTier::create(LoyaltyProgramId::new(), "Bronze", -1, json!({}), 0).unwrap_err();
        assert!(matches!(err, LoyaltyError::NegativeThreshold(-1)));
    }

    #[test]
    fn create_zero_threshold_succeeds() {
        let t = MemberTier::create(LoyaltyProgramId::new(), "Bronze", 0, json!({}), 0).unwrap();
        assert_eq!(t.threshold_points(), 0);
    }
}
