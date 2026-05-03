use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::RewardRedemption;
use crate::domain::repositories::RewardRedemptionRepository;
use crate::domain::value_objects::{
    LoyaltyMemberId, PointsLedgerEntryId, RewardId, RewardRedemptionId,
};

pub struct PgRewardRedemptionRepository {
    pool: PgPool,
}

impl PgRewardRedemptionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RewardRedemptionRepository for PgRewardRedemptionRepository {
    async fn save(&self, r: &RewardRedemption) -> Result<(), LoyaltyError> {
        sqlx::query(
            r#"
            INSERT INTO loyalty_reward_redemptions (
                id, member_id, reward_id, ledger_entry_id,
                applied_to_sale_id, redeemed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(r.id().into_uuid())
        .bind(r.member_id().into_uuid())
        .bind(r.reward_id().into_uuid())
        .bind(r.ledger_entry_id().into_uuid())
        .bind(r.applied_to_sale_id())
        .bind(r.redeemed_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: RewardRedemptionId,
    ) -> Result<Option<RewardRedemption>, LoyaltyError> {
        let row = sqlx::query_as::<_, RedemptionRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(RewardRedemption::from))
    }

    async fn count_for_member_reward(
        &self,
        member_id: LoyaltyMemberId,
        reward_id: RewardId,
    ) -> Result<i64, LoyaltyError> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM loyalty_reward_redemptions
            WHERE member_id = $1 AND reward_id = $2
            "#,
        )
        .bind(member_id.into_uuid())
        .bind(reward_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    async fn list_for_member(
        &self,
        member_id: LoyaltyMemberId,
    ) -> Result<Vec<RewardRedemption>, LoyaltyError> {
        let rows = sqlx::query_as::<_, RedemptionRow>(LIST_FOR_MEMBER)
            .bind(member_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(RewardRedemption::from).collect())
    }

    async fn mark_applied(
        &self,
        id: RewardRedemptionId,
        sale_id: Uuid,
    ) -> Result<(), LoyaltyError> {
        let result = sqlx::query(
            "UPDATE loyalty_reward_redemptions SET applied_to_sale_id = $2 WHERE id = $1",
        )
        .bind(id.into_uuid())
        .bind(sale_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(LoyaltyError::RewardNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, member_id, reward_id, ledger_entry_id, applied_to_sale_id, redeemed_at
FROM loyalty_reward_redemptions
WHERE id = $1
"#;

const LIST_FOR_MEMBER: &str = r#"
SELECT id, member_id, reward_id, ledger_entry_id, applied_to_sale_id, redeemed_at
FROM loyalty_reward_redemptions
WHERE member_id = $1
ORDER BY redeemed_at DESC
"#;

#[derive(sqlx::FromRow)]
struct RedemptionRow {
    id: Uuid,
    member_id: Uuid,
    reward_id: Uuid,
    ledger_entry_id: Uuid,
    applied_to_sale_id: Option<Uuid>,
    redeemed_at: DateTime<Utc>,
}

impl From<RedemptionRow> for RewardRedemption {
    fn from(row: RedemptionRow) -> Self {
        RewardRedemption::reconstitute(
            RewardRedemptionId::from_uuid(row.id),
            LoyaltyMemberId::from_uuid(row.member_id),
            RewardId::from_uuid(row.reward_id),
            PointsLedgerEntryId::from_uuid(row.ledger_entry_id),
            row.applied_to_sale_id,
            row.redeemed_at,
        )
    }
}
