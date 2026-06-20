use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use tenancy::PlanTier;

use crate::SubscriptionError;
use crate::domain::entities::SubscriptionPlan;
use crate::domain::repositories::SubscriptionPlanRepository;
use crate::domain::value_objects::{BillingInterval, PlanCode, SubscriptionPlanId};

pub struct PgSubscriptionPlanRepository {
    pool: PgPool,
}

impl PgSubscriptionPlanRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriptionPlanRepository for PgSubscriptionPlanRepository {
    async fn save(&self, p: &SubscriptionPlan) -> Result<(), SubscriptionError> {
        save_q(&self.pool, p).await
    }

    async fn update(&self, p: &SubscriptionPlan) -> Result<(), SubscriptionError> {
        update_q(&self.pool, p).await
    }

    async fn find_by_id(
        &self,
        id: SubscriptionPlanId,
    ) -> Result<Option<SubscriptionPlan>, SubscriptionError> {
        find_by_id_q(&self.pool, id).await
    }

    async fn find_by_code(
        &self,
        code: &PlanCode,
    ) -> Result<Option<SubscriptionPlan>, SubscriptionError> {
        find_by_code_q(&self.pool, code).await
    }

    async fn save_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        plan: &SubscriptionPlan,
    ) -> Result<(), SubscriptionError> {
        save_q(&mut **tx, plan).await
    }

    async fn update_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        plan: &SubscriptionPlan,
    ) -> Result<(), SubscriptionError> {
        update_q(&mut **tx, plan).await
    }

    async fn find_by_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: SubscriptionPlanId,
    ) -> Result<Option<SubscriptionPlan>, SubscriptionError> {
        find_by_id_q(&mut **tx, id).await
    }

    async fn find_by_code_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        code: &PlanCode,
    ) -> Result<Option<SubscriptionPlan>, SubscriptionError> {
        find_by_code_q(&mut **tx, code).await
    }

    async fn find_active(&self) -> Result<Vec<SubscriptionPlan>, SubscriptionError> {
        let rows = sqlx::query_as::<_, PlanRow>(SELECT_ACTIVE)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(SubscriptionPlan::try_from).collect()
    }

    async fn list_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<SubscriptionPlan>, i64), SubscriptionError> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = (page - 1) * page_size;

        let (total,): (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM subscription_plans"#)
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query_as::<_, PlanRow>(LIST_PAGINATED)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
        let plans: Result<Vec<SubscriptionPlan>, SubscriptionError> =
            rows.into_iter().map(SubscriptionPlan::try_from).collect();
        Ok((plans?, total))
    }
}

// ---- Executor-generic query helpers ----------------------------------------
// Each query body lives here once; the trait's pool-backed and transactional
// methods both delegate, passing either `&self.pool` or `&mut **tx`.

async fn save_q<'e, E: PgExecutor<'e>>(
    exec: E,
    p: &SubscriptionPlan,
) -> Result<(), SubscriptionError> {
    sqlx::query(
        r#"
        INSERT INTO subscription_plans (
            id, code, name, description, tier, interval,
            price_cents, currency, trial_days, is_active, sort_order,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(p.id().into_uuid())
    .bind(p.code().as_str())
    .bind(p.name())
    .bind(p.description())
    .bind(p.tier().as_str())
    .bind(p.interval().as_str())
    .bind(p.price_cents())
    .bind(p.currency())
    .bind(p.trial_days())
    .bind(p.is_active())
    .bind(p.sort_order())
    .bind(p.created_at())
    .bind(p.updated_at())
    .execute(exec)
    .await?;
    Ok(())
}

async fn update_q<'e, E: PgExecutor<'e>>(
    exec: E,
    p: &SubscriptionPlan,
) -> Result<(), SubscriptionError> {
    let result = sqlx::query(
        r#"
        UPDATE subscription_plans
           SET name        = $2,
               description = $3,
               trial_days  = $4,
               is_active   = $5,
               sort_order  = $6,
               updated_at  = $7
         WHERE id = $1
        "#,
    )
    .bind(p.id().into_uuid())
    .bind(p.name())
    .bind(p.description())
    .bind(p.trial_days())
    .bind(p.is_active())
    .bind(p.sort_order())
    .bind(p.updated_at())
    .execute(exec)
    .await?;
    if result.rows_affected() == 0 {
        return Err(SubscriptionError::PlanNotFound(p.id().into_uuid()));
    }
    Ok(())
}

async fn find_by_id_q<'e, E: PgExecutor<'e>>(
    exec: E,
    id: SubscriptionPlanId,
) -> Result<Option<SubscriptionPlan>, SubscriptionError> {
    let row = sqlx::query_as::<_, PlanRow>(SELECT_ALL_COLS)
        .bind(id.into_uuid())
        .fetch_optional(exec)
        .await?;
    row.map(SubscriptionPlan::try_from).transpose()
}

async fn find_by_code_q<'e, E: PgExecutor<'e>>(
    exec: E,
    code: &PlanCode,
) -> Result<Option<SubscriptionPlan>, SubscriptionError> {
    let row = sqlx::query_as::<_, PlanRow>(SELECT_BY_CODE)
        .bind(code.as_str())
        .fetch_optional(exec)
        .await?;
    row.map(SubscriptionPlan::try_from).transpose()
}

const SELECT_ALL_COLS: &str = r#"
SELECT id, code, name, description, tier, interval,
       price_cents, currency, trial_days, is_active, sort_order,
       created_at, updated_at
FROM subscription_plans
WHERE id = $1
"#;

const SELECT_BY_CODE: &str = r#"
SELECT id, code, name, description, tier, interval,
       price_cents, currency, trial_days, is_active, sort_order,
       created_at, updated_at
FROM subscription_plans
WHERE code = $1
"#;

const SELECT_ACTIVE: &str = r#"
SELECT id, code, name, description, tier, interval,
       price_cents, currency, trial_days, is_active, sort_order,
       created_at, updated_at
FROM subscription_plans
WHERE is_active = TRUE
ORDER BY sort_order ASC, created_at ASC
"#;

const LIST_PAGINATED: &str = r#"
SELECT id, code, name, description, tier, interval,
       price_cents, currency, trial_days, is_active, sort_order,
       created_at, updated_at
FROM subscription_plans
ORDER BY sort_order ASC, created_at ASC
LIMIT $1 OFFSET $2
"#;

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: Uuid,
    code: String,
    name: String,
    description: Option<String>,
    tier: String,
    interval: String,
    price_cents: i64,
    currency: String,
    trial_days: Option<i32>,
    is_active: bool,
    sort_order: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PlanRow> for SubscriptionPlan {
    type Error = SubscriptionError;
    fn try_from(r: PlanRow) -> Result<Self, SubscriptionError> {
        // Tier strings are stored exactly as `PlanTier::as_str` produces
        // them; bridge the tenancy error type into ours.
        let tier = PlanTier::from_str(&r.tier).map_err(|e| {
            SubscriptionError::Validation(format!("invalid tier in subscription_plans row: {e}"))
        })?;
        Ok(SubscriptionPlan::reconstitute(
            SubscriptionPlanId::from_uuid(r.id),
            PlanCode::new(r.code)?,
            r.name,
            r.description,
            tier,
            BillingInterval::from_str(&r.interval)?,
            r.price_cents,
            r.currency,
            r.trial_days,
            r.is_active,
            r.sort_order,
            r.created_at,
            r.updated_at,
        ))
    }
}
