use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::TenancyError;
use crate::domain::entities::OrganizationPlan;
use crate::domain::repositories::OrganizationPlanRepository;
use crate::domain::value_objects::{OrganizationId, OrganizationPlanId, PlanTier};

pub struct PgOrganizationPlanRepository {
    pool: PgPool,
}

impl PgOrganizationPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationPlanRepository for PgOrganizationPlanRepository {
    async fn upsert(&self, p: &OrganizationPlan) -> Result<(), TenancyError> {
        sqlx::query(
            r#"
            INSERT INTO organization_plans (
                id, organization_id, tier, feature_flags,
                seat_limit, store_limit, starts_at, expires_at,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id) DO UPDATE SET
                tier          = EXCLUDED.tier,
                feature_flags = EXCLUDED.feature_flags,
                seat_limit    = EXCLUDED.seat_limit,
                store_limit   = EXCLUDED.store_limit,
                expires_at    = EXCLUDED.expires_at,
                updated_at    = EXCLUDED.updated_at
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.organization_id().into_uuid())
        .bind(p.tier().as_str())
        .bind(p.feature_flags())
        .bind(p.seat_limit())
        .bind(p.store_limit())
        .bind(p.starts_at())
        .bind(p.expires_at())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<OrganizationPlan>, TenancyError> {
        let row = sqlx::query_as::<_, PlanRow>(
            r#"
            SELECT id, organization_id, tier, feature_flags,
                   seat_limit, store_limit, starts_at, expires_at,
                   created_at, updated_at
            FROM organization_plans
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(OrganizationPlan::try_from).transpose()
    }
}

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: Uuid,
    organization_id: Uuid,
    tier: String,
    feature_flags: JsonValue,
    seat_limit: Option<i32>,
    store_limit: Option<i32>,
    starts_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PlanRow> for OrganizationPlan {
    type Error = TenancyError;
    fn try_from(r: PlanRow) -> Result<Self, TenancyError> {
        Ok(OrganizationPlan::reconstitute(
            OrganizationPlanId::from_uuid(r.id),
            OrganizationId::from_uuid(r.organization_id),
            PlanTier::from_str(&r.tier)?,
            r.feature_flags,
            r.seat_limit,
            r.store_limit,
            r.starts_at,
            r.expires_at,
            r.created_at,
            r.updated_at,
        ))
    }
}
