use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::SubscriptionError;
use crate::domain::entities::DunningAttempt;
use crate::domain::repositories::DunningAttemptRepository;
use crate::domain::value_objects::{BillingCycleId, DunningAttemptId, DunningOutcome};

pub struct PgDunningAttemptRepository {
    pool: PgPool,
}

impl PgDunningAttemptRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DunningAttemptRepository for PgDunningAttemptRepository {
    async fn save(&self, a: &DunningAttempt) -> Result<(), SubscriptionError> {
        sqlx::query(
            r#"
            INSERT INTO dunning_attempts (
                id, billing_cycle_id, attempt_number, scheduled_at,
                executed_at, outcome, failure_reason, transaction_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.billing_cycle_id().into_uuid())
        .bind(a.attempt_number())
        .bind(a.scheduled_at())
        .bind(a.executed_at())
        .bind(a.outcome().as_str())
        .bind(a.failure_reason())
        .bind(a.transaction_id())
        .bind(a.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, a: &DunningAttempt) -> Result<(), SubscriptionError> {
        let result = sqlx::query(
            r#"
            UPDATE dunning_attempts
               SET executed_at    = $2,
                   outcome        = $3,
                   failure_reason = $4,
                   transaction_id = $5
             WHERE id = $1
            "#,
        )
        .bind(a.id().into_uuid())
        .bind(a.executed_at())
        .bind(a.outcome().as_str())
        .bind(a.failure_reason())
        .bind(a.transaction_id())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(SubscriptionError::DunningAttemptNotFound(
                a.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: DunningAttemptId,
    ) -> Result<Option<DunningAttempt>, SubscriptionError> {
        let row = sqlx::query_as::<_, AttemptRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(DunningAttempt::try_from).transpose()
    }

    async fn find_by_billing_cycle(
        &self,
        billing_cycle_id: BillingCycleId,
    ) -> Result<Vec<DunningAttempt>, SubscriptionError> {
        let rows = sqlx::query_as::<_, AttemptRow>(SELECT_BY_CYCLE)
            .bind(billing_cycle_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(DunningAttempt::try_from).collect()
    }

    async fn find_by_transaction_id(
        &self,
        transaction_id: Uuid,
    ) -> Result<Option<DunningAttempt>, SubscriptionError> {
        let row = sqlx::query_as::<_, AttemptRow>(SELECT_BY_TRANSACTION)
            .bind(transaction_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(DunningAttempt::try_from).transpose()
    }

    async fn find_due(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<DunningAttempt>, SubscriptionError> {
        let rows = sqlx::query_as::<_, AttemptRow>(SELECT_DUE)
            .bind(now)
            .bind(limit.max(1))
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(DunningAttempt::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, billing_cycle_id, attempt_number, scheduled_at,
       executed_at, outcome, failure_reason, transaction_id, created_at
FROM dunning_attempts
WHERE id = $1
"#;

const SELECT_BY_CYCLE: &str = r#"
SELECT id, billing_cycle_id, attempt_number, scheduled_at,
       executed_at, outcome, failure_reason, transaction_id, created_at
FROM dunning_attempts
WHERE billing_cycle_id = $1
ORDER BY attempt_number ASC
"#;

const SELECT_BY_TRANSACTION: &str = r#"
SELECT id, billing_cycle_id, attempt_number, scheduled_at,
       executed_at, outcome, failure_reason, transaction_id, created_at
FROM dunning_attempts
WHERE transaction_id = $1
"#;

// `transaction_id IS NULL` filters out attempts that already fired and are
// awaiting a webhook — the dunning job should not retry them.
const SELECT_DUE: &str = r#"
SELECT id, billing_cycle_id, attempt_number, scheduled_at,
       executed_at, outcome, failure_reason, transaction_id, created_at
FROM dunning_attempts
WHERE outcome = 'pending'
  AND transaction_id IS NULL
  AND scheduled_at <= $1
ORDER BY scheduled_at ASC
LIMIT $2
"#;

#[derive(sqlx::FromRow)]
struct AttemptRow {
    id: Uuid,
    billing_cycle_id: Uuid,
    attempt_number: i16,
    scheduled_at: DateTime<Utc>,
    executed_at: Option<DateTime<Utc>>,
    outcome: String,
    failure_reason: Option<String>,
    transaction_id: Option<Uuid>,
    created_at: DateTime<Utc>,
}

impl TryFrom<AttemptRow> for DunningAttempt {
    type Error = SubscriptionError;
    fn try_from(r: AttemptRow) -> Result<Self, SubscriptionError> {
        Ok(DunningAttempt::reconstitute(
            DunningAttemptId::from_uuid(r.id),
            BillingCycleId::from_uuid(r.billing_cycle_id),
            r.attempt_number,
            r.scheduled_at,
            r.executed_at,
            DunningOutcome::from_str(&r.outcome)?,
            r.failure_reason,
            r.transaction_id,
            r.created_at,
        ))
    }
}
