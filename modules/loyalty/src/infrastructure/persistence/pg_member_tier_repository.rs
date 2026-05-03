use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::MemberTier;
use crate::domain::repositories::MemberTierRepository;
use crate::domain::value_objects::{LoyaltyProgramId, MemberTierId};

pub struct PgMemberTierRepository {
    pool: PgPool,
}

impl PgMemberTierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MemberTierRepository for PgMemberTierRepository {
    async fn save(&self, t: &MemberTier) -> Result<(), LoyaltyError> {
        sqlx::query(
            r#"
            INSERT INTO loyalty_member_tiers (
                id, program_id, name, threshold_points,
                benefits, sort_order, is_active, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(t.id().into_uuid())
        .bind(t.program_id().into_uuid())
        .bind(t.name())
        .bind(t.threshold_points())
        .bind(t.benefits())
        .bind(t.sort_order())
        .bind(t.is_active())
        .bind(t.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: MemberTierId) -> Result<Option<MemberTier>, LoyaltyError> {
        let row = sqlx::query_as::<_, TierRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(MemberTier::from))
    }

    async fn list_by_program(
        &self,
        program_id: LoyaltyProgramId,
    ) -> Result<Vec<MemberTier>, LoyaltyError> {
        let rows = sqlx::query_as::<_, TierRow>(LIST_BY_PROGRAM)
            .bind(program_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(MemberTier::from).collect())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, program_id, name, threshold_points, benefits,
       sort_order, is_active, created_at
FROM loyalty_member_tiers
WHERE id = $1
"#;

const LIST_BY_PROGRAM: &str = r#"
SELECT id, program_id, name, threshold_points, benefits,
       sort_order, is_active, created_at
FROM loyalty_member_tiers
WHERE program_id = $1
ORDER BY threshold_points ASC, sort_order ASC
"#;

#[derive(sqlx::FromRow)]
struct TierRow {
    id: Uuid,
    program_id: Uuid,
    name: String,
    threshold_points: i64,
    benefits: JsonValue,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
}

impl From<TierRow> for MemberTier {
    fn from(row: TierRow) -> Self {
        MemberTier::reconstitute(
            MemberTierId::from_uuid(row.id),
            LoyaltyProgramId::from_uuid(row.program_id),
            row.name,
            row.threshold_points,
            row.benefits,
            row.sort_order,
            row.is_active,
            row.created_at,
        )
    }
}
