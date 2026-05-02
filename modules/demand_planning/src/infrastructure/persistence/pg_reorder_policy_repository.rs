use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::ReorderPolicy;
use crate::domain::repositories::ReorderPolicyRepository;
use crate::domain::value_objects::ReorderPolicyId;

pub struct PgReorderPolicyRepository {
    pool: PgPool,
}

impl PgReorderPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReorderPolicyRepository for PgReorderPolicyRepository {
    async fn save(&self, p: &ReorderPolicy) -> Result<(), DemandPlanningError> {
        sqlx::query(
            r#"
            INSERT INTO reorder_policies (
                id, product_variant_id, store_id, min_qty, max_qty,
                lead_time_days, safety_stock_qty, review_cycle_days,
                preferred_vendor_id, is_active, version, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.product_variant_id())
        .bind(p.store_id())
        .bind(p.min_qty())
        .bind(p.max_qty())
        .bind(p.lead_time_days())
        .bind(p.safety_stock_qty())
        .bind(p.review_cycle_days())
        .bind(p.preferred_vendor_id())
        .bind(p.is_active())
        .bind(p.version())
        .bind(p.created_at())
        .bind(p.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, p: &ReorderPolicy) -> Result<(), DemandPlanningError> {
        // Optimistic lock: matched only if version equals the in-memory one,
        // and bumped on persist. If 0 rows are affected, another writer won.
        let result = sqlx::query(
            r#"
            UPDATE reorder_policies
            SET min_qty             = $2,
                max_qty             = $3,
                lead_time_days      = $4,
                safety_stock_qty    = $5,
                review_cycle_days   = $6,
                preferred_vendor_id = $7,
                is_active           = $8,
                version             = version + 1,
                updated_at          = $9
            WHERE id = $1 AND version = $10
            "#,
        )
        .bind(p.id().into_uuid())
        .bind(p.min_qty())
        .bind(p.max_qty())
        .bind(p.lead_time_days())
        .bind(p.safety_stock_qty())
        .bind(p.review_cycle_days())
        .bind(p.preferred_vendor_id())
        .bind(p.is_active())
        .bind(p.updated_at())
        .bind(p.version())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            let exists: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM reorder_policies WHERE id = $1")
                    .bind(p.id().into_uuid())
                    .fetch_optional(&self.pool)
                    .await?;
            if exists.is_none() {
                return Err(DemandPlanningError::ReorderPolicyNotFound(
                    p.id().into_uuid(),
                ));
            }
            return Err(DemandPlanningError::PolicyVersionConflict(
                p.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ReorderPolicyId,
    ) -> Result<Option<ReorderPolicy>, DemandPlanningError> {
        let row = sqlx::query_as::<_, PolicyRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(ReorderPolicy::from))
    }

    async fn find_by_variant_store(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<ReorderPolicy>, DemandPlanningError> {
        let row = sqlx::query_as::<_, PolicyRow>(SELECT_BY_VARIANT_STORE)
            .bind(product_variant_id)
            .bind(store_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(ReorderPolicy::from))
    }

    async fn list_active(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<ReorderPolicy>, DemandPlanningError> {
        let rows = match store_id {
            Some(s) => {
                sqlx::query_as::<_, PolicyRow>(LIST_ACTIVE_BY_STORE)
                    .bind(s)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as::<_, PolicyRow>(LIST_ACTIVE)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
        Ok(rows.into_iter().map(ReorderPolicy::from).collect())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, product_variant_id, store_id, min_qty, max_qty,
       lead_time_days, safety_stock_qty, review_cycle_days,
       preferred_vendor_id, is_active, version, created_at, updated_at
FROM reorder_policies
WHERE id = $1
"#;

const SELECT_BY_VARIANT_STORE: &str = r#"
SELECT id, product_variant_id, store_id, min_qty, max_qty,
       lead_time_days, safety_stock_qty, review_cycle_days,
       preferred_vendor_id, is_active, version, created_at, updated_at
FROM reorder_policies
WHERE product_variant_id = $1 AND store_id = $2
"#;

const LIST_ACTIVE: &str = r#"
SELECT id, product_variant_id, store_id, min_qty, max_qty,
       lead_time_days, safety_stock_qty, review_cycle_days,
       preferred_vendor_id, is_active, version, created_at, updated_at
FROM reorder_policies
WHERE is_active = TRUE
ORDER BY updated_at DESC
"#;

const LIST_ACTIVE_BY_STORE: &str = r#"
SELECT id, product_variant_id, store_id, min_qty, max_qty,
       lead_time_days, safety_stock_qty, review_cycle_days,
       preferred_vendor_id, is_active, version, created_at, updated_at
FROM reorder_policies
WHERE is_active = TRUE AND store_id = $1
ORDER BY updated_at DESC
"#;

#[derive(sqlx::FromRow)]
struct PolicyRow {
    id: Uuid,
    product_variant_id: Uuid,
    store_id: Uuid,
    min_qty: Decimal,
    max_qty: Decimal,
    lead_time_days: i32,
    safety_stock_qty: Decimal,
    review_cycle_days: i32,
    preferred_vendor_id: Option<Uuid>,
    is_active: bool,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PolicyRow> for ReorderPolicy {
    fn from(row: PolicyRow) -> Self {
        ReorderPolicy::reconstitute(
            ReorderPolicyId::from_uuid(row.id),
            row.product_variant_id,
            row.store_id,
            row.min_qty,
            row.max_qty,
            row.lead_time_days,
            row.safety_stock_qty,
            row.review_cycle_days,
            row.preferred_vendor_id,
            row.is_active,
            row.version,
            row.created_at,
            row.updated_at,
        )
    }
}
