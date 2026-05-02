use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::ReplenishmentSuggestion;
use crate::domain::repositories::ReplenishmentSuggestionRepository;
use crate::domain::value_objects::{SuggestionId, SuggestionStatus};

pub struct PgReplenishmentSuggestionRepository {
    pool: PgPool,
}

impl PgReplenishmentSuggestionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReplenishmentSuggestionRepository for PgReplenishmentSuggestionRepository {
    async fn save(&self, s: &ReplenishmentSuggestion) -> Result<(), DemandPlanningError> {
        sqlx::query(
            r#"
            INSERT INTO replenishment_suggestions (
                id, product_variant_id, store_id, current_stock, forecast_qty,
                recommended_qty, suggested_vendor_id, status, generated_at,
                decided_at, decided_by, generated_purchase_order_id, dismiss_reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.product_variant_id())
        .bind(s.store_id())
        .bind(s.current_stock())
        .bind(s.forecast_qty())
        .bind(s.recommended_qty())
        .bind(s.suggested_vendor_id())
        .bind(s.status().to_string())
        .bind(s.generated_at())
        .bind(s.decided_at())
        .bind(s.decided_by())
        .bind(s.generated_purchase_order_id())
        .bind(s.dismiss_reason())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, s: &ReplenishmentSuggestion) -> Result<(), DemandPlanningError> {
        let result = sqlx::query(
            r#"
            UPDATE replenishment_suggestions
            SET status                       = $2,
                decided_at                   = $3,
                decided_by                   = $4,
                generated_purchase_order_id  = $5,
                dismiss_reason               = $6
            WHERE id = $1
            "#,
        )
        .bind(s.id().into_uuid())
        .bind(s.status().to_string())
        .bind(s.decided_at())
        .bind(s.decided_by())
        .bind(s.generated_purchase_order_id())
        .bind(s.dismiss_reason())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(DemandPlanningError::SuggestionNotFound(s.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: SuggestionId,
    ) -> Result<Option<ReplenishmentSuggestion>, DemandPlanningError> {
        let row = sqlx::query_as::<_, SuggestionRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(ReplenishmentSuggestion::try_from).transpose()
    }

    async fn list(
        &self,
        store_id: Option<Uuid>,
        status: Option<SuggestionStatus>,
    ) -> Result<Vec<ReplenishmentSuggestion>, DemandPlanningError> {
        let rows = match (store_id, status) {
            (Some(s), Some(st)) => {
                sqlx::query_as::<_, SuggestionRow>(LIST_BY_STORE_STATUS)
                    .bind(s)
                    .bind(st.to_string())
                    .fetch_all(&self.pool)
                    .await?
            }
            (Some(s), None) => {
                sqlx::query_as::<_, SuggestionRow>(LIST_BY_STORE)
                    .bind(s)
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, Some(st)) => {
                sqlx::query_as::<_, SuggestionRow>(LIST_BY_STATUS)
                    .bind(st.to_string())
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, None) => {
                sqlx::query_as::<_, SuggestionRow>(LIST_ALL)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
        rows.into_iter()
            .map(ReplenishmentSuggestion::try_from)
            .collect()
    }

    async fn has_pending_for(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<bool, DemandPlanningError> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT 1::BIGINT
            FROM replenishment_suggestions
            WHERE product_variant_id = $1 AND store_id = $2 AND status = 'pending'
            LIMIT 1
            "#,
        )
        .bind(product_variant_id)
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.is_some())
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, product_variant_id, store_id, current_stock, forecast_qty,
       recommended_qty, suggested_vendor_id, status, generated_at,
       decided_at, decided_by, generated_purchase_order_id, dismiss_reason
FROM replenishment_suggestions
WHERE id = $1
"#;

const LIST_BY_STORE_STATUS: &str = r#"
SELECT id, product_variant_id, store_id, current_stock, forecast_qty,
       recommended_qty, suggested_vendor_id, status, generated_at,
       decided_at, decided_by, generated_purchase_order_id, dismiss_reason
FROM replenishment_suggestions
WHERE store_id = $1 AND status = $2
ORDER BY generated_at DESC
"#;

const LIST_BY_STORE: &str = r#"
SELECT id, product_variant_id, store_id, current_stock, forecast_qty,
       recommended_qty, suggested_vendor_id, status, generated_at,
       decided_at, decided_by, generated_purchase_order_id, dismiss_reason
FROM replenishment_suggestions
WHERE store_id = $1
ORDER BY generated_at DESC
"#;

const LIST_BY_STATUS: &str = r#"
SELECT id, product_variant_id, store_id, current_stock, forecast_qty,
       recommended_qty, suggested_vendor_id, status, generated_at,
       decided_at, decided_by, generated_purchase_order_id, dismiss_reason
FROM replenishment_suggestions
WHERE status = $1
ORDER BY generated_at DESC
"#;

const LIST_ALL: &str = r#"
SELECT id, product_variant_id, store_id, current_stock, forecast_qty,
       recommended_qty, suggested_vendor_id, status, generated_at,
       decided_at, decided_by, generated_purchase_order_id, dismiss_reason
FROM replenishment_suggestions
ORDER BY generated_at DESC
"#;

#[derive(sqlx::FromRow)]
struct SuggestionRow {
    id: Uuid,
    product_variant_id: Uuid,
    store_id: Uuid,
    current_stock: Decimal,
    forecast_qty: Decimal,
    recommended_qty: Decimal,
    suggested_vendor_id: Option<Uuid>,
    status: String,
    generated_at: DateTime<Utc>,
    decided_at: Option<DateTime<Utc>>,
    decided_by: Option<Uuid>,
    generated_purchase_order_id: Option<Uuid>,
    dismiss_reason: Option<String>,
}

impl TryFrom<SuggestionRow> for ReplenishmentSuggestion {
    type Error = DemandPlanningError;

    fn try_from(row: SuggestionRow) -> Result<Self, Self::Error> {
        let status = SuggestionStatus::from_str(&row.status)?;
        Ok(ReplenishmentSuggestion::reconstitute(
            SuggestionId::from_uuid(row.id),
            row.product_variant_id,
            row.store_id,
            row.current_stock,
            row.forecast_qty,
            row.recommended_qty,
            row.suggested_vendor_id,
            status,
            row.generated_at,
            row.decided_at,
            row.decided_by,
            row.generated_purchase_order_id,
            row.dismiss_reason,
        ))
    }
}
