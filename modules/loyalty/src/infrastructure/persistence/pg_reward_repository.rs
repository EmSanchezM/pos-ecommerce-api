use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::Reward;
use crate::domain::repositories::RewardRepository;
use crate::domain::value_objects::{LoyaltyProgramId, RewardId, RewardType};

pub struct PgRewardRepository {
    pool: PgPool,
}

impl PgRewardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RewardRepository for PgRewardRepository {
    async fn save(&self, r: &Reward) -> Result<(), LoyaltyError> {
        sqlx::query(
            r#"
            INSERT INTO loyalty_rewards (
                id, program_id, name, description, cost_points,
                reward_type, reward_value, max_redemptions_per_member,
                is_active, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(r.id().into_uuid())
        .bind(r.program_id().into_uuid())
        .bind(r.name())
        .bind(r.description())
        .bind(r.cost_points())
        .bind(r.reward_type().to_string())
        .bind(r.reward_value())
        .bind(r.max_redemptions_per_member())
        .bind(r.is_active())
        .bind(r.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: RewardId) -> Result<Option<Reward>, LoyaltyError> {
        let row = sqlx::query_as::<_, RewardRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(Reward::try_from).transpose()
    }

    async fn list_by_program(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<Reward>, LoyaltyError> {
        let rows = sqlx::query_as::<_, RewardRow>(LIST_BY_PROGRAM)
            .bind(program_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Reward::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, program_id, name, description, cost_points,
       reward_type, reward_value, max_redemptions_per_member,
       is_active, created_at
FROM loyalty_rewards
WHERE id = $1
"#;

const LIST_BY_PROGRAM: &str = r#"
SELECT id, program_id, name, description, cost_points,
       reward_type, reward_value, max_redemptions_per_member,
       is_active, created_at
FROM loyalty_rewards
WHERE program_id = $1
ORDER BY cost_points ASC
"#;

#[derive(sqlx::FromRow)]
struct RewardRow {
    id: Uuid,
    program_id: Uuid,
    name: String,
    description: Option<String>,
    cost_points: i64,
    reward_type: String,
    reward_value: Decimal,
    max_redemptions_per_member: Option<i32>,
    is_active: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<RewardRow> for Reward {
    type Error = LoyaltyError;

    fn try_from(row: RewardRow) -> Result<Self, Self::Error> {
        let t = RewardType::from_str(&row.reward_type)?;
        Ok(Reward::reconstitute(
            RewardId::from_uuid(row.id),
            LoyaltyProgramId::from_uuid(row.program_id),
            row.name,
            row.description,
            row.cost_points,
            t,
            row.reward_value,
            row.max_redemptions_per_member,
            row.is_active,
            row.created_at,
        ))
    }
}
