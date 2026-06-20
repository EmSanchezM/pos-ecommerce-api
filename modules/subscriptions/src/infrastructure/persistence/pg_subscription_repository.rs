use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::domain::entities::Subscription;
use crate::domain::repositories::SubscriptionRepository;
use crate::domain::value_objects::{SubscriptionId, SubscriptionPlanId, SubscriptionStatus};

pub struct PgSubscriptionRepository {
    pool: PgPool,
}

impl PgSubscriptionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriptionRepository for PgSubscriptionRepository {
    async fn save(&self, s: &Subscription) -> Result<(), SubscriptionError> {
        sqlx::query(
            r#"
            INSERT INTO subscriptions (
                id, organization_id, plan_id, status,
                current_period_start, current_period_end, trial_end,
                cancel_at_period_end, canceled_at, version,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.organization_id().into_uuid())
        .bind(s.plan_id().into_uuid())
        .bind(s.status().as_str())
        .bind(s.current_period_start())
        .bind(s.current_period_end())
        .bind(s.trial_end())
        .bind(s.cancel_at_period_end())
        .bind(s.canceled_at())
        .bind(s.version())
        .bind(s.created_at())
        .bind(s.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_with_version(&self, s: &Subscription) -> Result<(), SubscriptionError> {
        update_with_version_q(&self.pool, s).await
    }

    async fn find_by_id(
        &self,
        id: SubscriptionId,
    ) -> Result<Option<Subscription>, SubscriptionError> {
        find_by_id_q(&self.pool, id).await
    }

    async fn find_active_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Option<Subscription>, SubscriptionError> {
        find_active_by_organization_q(&self.pool, organization_id).await
    }

    async fn update_with_version_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        s: &Subscription,
    ) -> Result<(), SubscriptionError> {
        update_with_version_q(&mut **tx, s).await
    }

    async fn find_by_id_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: SubscriptionId,
    ) -> Result<Option<Subscription>, SubscriptionError> {
        find_by_id_q(&mut **tx, id).await
    }

    async fn find_active_by_organization_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        organization_id: OrganizationId,
    ) -> Result<Option<Subscription>, SubscriptionError> {
        find_active_by_organization_q(&mut **tx, organization_id).await
    }

    async fn find_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Subscription>, SubscriptionError> {
        let rows = sqlx::query_as::<_, SubscriptionRow>(SELECT_ALL_BY_ORG)
            .bind(organization_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Subscription::try_from).collect()
    }

    async fn list_due_for_billing(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<Subscription>, SubscriptionError> {
        let rows = sqlx::query_as::<_, SubscriptionRow>(SELECT_DUE_FOR_BILLING)
            .bind(now)
            .bind(limit.max(1))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Subscription::try_from).collect()
    }

    async fn list_past_due_pending_cancellation(
        &self,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Subscription>, SubscriptionError> {
        let rows = sqlx::query_as::<_, SubscriptionRow>(SELECT_PAST_DUE_AGED)
            .bind(cutoff)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(Subscription::try_from).collect()
    }
}

// ---- Executor-generic query helpers ----------------------------------------

async fn update_with_version_q<'e, E: PgExecutor<'e>>(
    exec: E,
    s: &Subscription,
) -> Result<(), SubscriptionError> {
    // Optimistic lock: the entity already advanced its `version`, so the row in
    // PG must still be at `version - 1`.
    let expected = s
        .version()
        .checked_sub(1)
        .ok_or(SubscriptionError::OptimisticLockFailed)?;
    let result = sqlx::query(
        r#"
        UPDATE subscriptions
           SET plan_id              = $2,
               status               = $3,
               current_period_start = $4,
               current_period_end   = $5,
               trial_end            = $6,
               cancel_at_period_end = $7,
               canceled_at          = $8,
               version              = $9,
               updated_at           = $10
         WHERE id = $1 AND version = $11
        "#,
    )
    .bind(s.id().into_uuid())
    .bind(s.plan_id().into_uuid())
    .bind(s.status().as_str())
    .bind(s.current_period_start())
    .bind(s.current_period_end())
    .bind(s.trial_end())
    .bind(s.cancel_at_period_end())
    .bind(s.canceled_at())
    .bind(s.version())
    .bind(s.updated_at())
    .bind(expected)
    .execute(exec)
    .await?;
    if result.rows_affected() == 0 {
        return Err(SubscriptionError::OptimisticLockFailed);
    }
    Ok(())
}

async fn find_by_id_q<'e, E: PgExecutor<'e>>(
    exec: E,
    id: SubscriptionId,
) -> Result<Option<Subscription>, SubscriptionError> {
    let row = sqlx::query_as::<_, SubscriptionRow>(SELECT_BY_ID)
        .bind(id.into_uuid())
        .fetch_optional(exec)
        .await?;
    row.map(Subscription::try_from).transpose()
}

async fn find_active_by_organization_q<'e, E: PgExecutor<'e>>(
    exec: E,
    organization_id: OrganizationId,
) -> Result<Option<Subscription>, SubscriptionError> {
    let row = sqlx::query_as::<_, SubscriptionRow>(SELECT_ACTIVE_BY_ORG)
        .bind(organization_id.into_uuid())
        .fetch_optional(exec)
        .await?;
    row.map(Subscription::try_from).transpose()
}

const ALL_COLUMNS: &str = r#"
    id, organization_id, plan_id, status,
    current_period_start, current_period_end, trial_end,
    cancel_at_period_end, canceled_at, version,
    created_at, updated_at
"#;

// We can't use `format!` for a `const &str`, so each query inlines the column
// list — keeps the Pg layer trivial to grep.
const SELECT_BY_ID: &str = r#"
SELECT id, organization_id, plan_id, status,
       current_period_start, current_period_end, trial_end,
       cancel_at_period_end, canceled_at, version,
       created_at, updated_at
FROM subscriptions
WHERE id = $1
"#;

const SELECT_ACTIVE_BY_ORG: &str = r#"
SELECT id, organization_id, plan_id, status,
       current_period_start, current_period_end, trial_end,
       cancel_at_period_end, canceled_at, version,
       created_at, updated_at
FROM subscriptions
WHERE organization_id = $1 AND status <> 'canceled'
"#;

const SELECT_ALL_BY_ORG: &str = r#"
SELECT id, organization_id, plan_id, status,
       current_period_start, current_period_end, trial_end,
       cancel_at_period_end, canceled_at, version,
       created_at, updated_at
FROM subscriptions
WHERE organization_id = $1
ORDER BY created_at DESC
"#;

const SELECT_DUE_FOR_BILLING: &str = r#"
SELECT id, organization_id, plan_id, status,
       current_period_start, current_period_end, trial_end,
       cancel_at_period_end, canceled_at, version,
       created_at, updated_at
FROM subscriptions
WHERE current_period_end <= $1
  AND status IN ('trialing', 'active')
ORDER BY current_period_end ASC
LIMIT $2
"#;

const SELECT_PAST_DUE_AGED: &str = r#"
SELECT id, organization_id, plan_id, status,
       current_period_start, current_period_end, trial_end,
       cancel_at_period_end, canceled_at, version,
       created_at, updated_at
FROM subscriptions
WHERE status = 'past_due' AND updated_at <= $1
ORDER BY updated_at ASC
"#;

// Suppress dead-code warning for the documentation constant above; useful as
// a single source of truth even though the queries inline the list.
#[allow(dead_code)]
const _ALL_COLUMNS_REFERENCE: &str = ALL_COLUMNS;

#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    id: Uuid,
    organization_id: Uuid,
    plan_id: Uuid,
    status: String,
    current_period_start: DateTime<Utc>,
    current_period_end: DateTime<Utc>,
    trial_end: Option<DateTime<Utc>>,
    cancel_at_period_end: bool,
    canceled_at: Option<DateTime<Utc>>,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<SubscriptionRow> for Subscription {
    type Error = SubscriptionError;
    fn try_from(r: SubscriptionRow) -> Result<Self, SubscriptionError> {
        Ok(Subscription::reconstitute(
            SubscriptionId::from_uuid(r.id),
            OrganizationId::from_uuid(r.organization_id),
            SubscriptionPlanId::from_uuid(r.plan_id),
            SubscriptionStatus::from_str(&r.status)?,
            r.current_period_start,
            r.current_period_end,
            r.trial_end,
            r.cancel_at_period_end,
            r.canceled_at,
            r.version,
            r.created_at,
            r.updated_at,
        ))
    }
}
