use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::LoyaltyMember;
use crate::domain::repositories::LoyaltyMemberRepository;
use crate::domain::value_objects::{LoyaltyMemberId, LoyaltyProgramId, MemberTierId};

pub struct PgLoyaltyMemberRepository {
    pool: PgPool,
}

impl PgLoyaltyMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LoyaltyMemberRepository for PgLoyaltyMemberRepository {
    async fn save(&self, m: &LoyaltyMember) -> Result<(), LoyaltyError> {
        sqlx::query(
            r#"
            INSERT INTO loyalty_members (
                id, program_id, customer_id, current_tier_id,
                current_balance, lifetime_points, enrolled_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(m.id().into_uuid())
        .bind(m.program_id().into_uuid())
        .bind(m.customer_id())
        .bind(m.current_tier_id().map(|t| t.into_uuid()))
        .bind(m.current_balance())
        .bind(m.lifetime_points())
        .bind(m.enrolled_at())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: LoyaltyMemberId) -> Result<Option<LoyaltyMember>, LoyaltyError> {
        let row = sqlx::query_as::<_, MemberRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(LoyaltyMember::from))
    }

    async fn find_by_customer(
        &self,
        program_id: LoyaltyProgramId,
        customer_id: Uuid,
    ) -> Result<Option<LoyaltyMember>, LoyaltyError> {
        let row = sqlx::query_as::<_, MemberRow>(SELECT_BY_CUSTOMER)
            .bind(program_id.into_uuid())
            .bind(customer_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(LoyaltyMember::from))
    }

    async fn list_by_program(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<LoyaltyMember>, LoyaltyError> {
        let rows = sqlx::query_as::<_, MemberRow>(LIST_BY_PROGRAM)
            .bind(program_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(LoyaltyMember::from).collect())
    }

    async fn update_tier(
        &self,
        id: LoyaltyMemberId,
        tier_id: Option<MemberTierId>,
    ) -> Result<(), LoyaltyError> {
        let result = sqlx::query(
            r#"
            UPDATE loyalty_members
            SET current_tier_id = $2,
                updated_at      = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .bind(tier_id.map(|t| t.into_uuid()))
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(LoyaltyError::MemberNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, program_id, customer_id, current_tier_id,
       current_balance, lifetime_points, enrolled_at, updated_at
FROM loyalty_members
WHERE id = $1
"#;

const SELECT_BY_CUSTOMER: &str = r#"
SELECT id, program_id, customer_id, current_tier_id,
       current_balance, lifetime_points, enrolled_at, updated_at
FROM loyalty_members
WHERE program_id = $1 AND customer_id = $2
"#;

const LIST_BY_PROGRAM: &str = r#"
SELECT id, program_id, customer_id, current_tier_id,
       current_balance, lifetime_points, enrolled_at, updated_at
FROM loyalty_members
WHERE program_id = $1
ORDER BY enrolled_at DESC
"#;

#[derive(sqlx::FromRow)]
struct MemberRow {
    id: Uuid,
    program_id: Uuid,
    customer_id: Uuid,
    current_tier_id: Option<Uuid>,
    current_balance: i64,
    lifetime_points: i64,
    enrolled_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<MemberRow> for LoyaltyMember {
    fn from(row: MemberRow) -> Self {
        LoyaltyMember::reconstitute(
            LoyaltyMemberId::from_uuid(row.id),
            LoyaltyProgramId::from_uuid(row.program_id),
            row.customer_id,
            row.current_tier_id.map(MemberTierId::from_uuid),
            row.current_balance,
            row.lifetime_points,
            row.enrolled_at,
            row.updated_at,
        )
    }
}
