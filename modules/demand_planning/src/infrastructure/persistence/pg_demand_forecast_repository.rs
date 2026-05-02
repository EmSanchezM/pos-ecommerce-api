use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::DemandForecast;
use crate::domain::repositories::DemandForecastRepository;
use crate::domain::value_objects::{ForecastId, ForecastMethod, ForecastPeriod};

pub struct PgDemandForecastRepository {
    pool: PgPool,
}

impl PgDemandForecastRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DemandForecastRepository for PgDemandForecastRepository {
    async fn save(&self, f: &DemandForecast) -> Result<(), DemandPlanningError> {
        sqlx::query(
            r#"
            INSERT INTO demand_forecasts (
                id, product_variant_id, store_id, period, period_start, period_end,
                method, forecasted_qty, confidence_low, confidence_high, computed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (product_variant_id, store_id, period, period_start, method)
            DO UPDATE SET
                period_end       = EXCLUDED.period_end,
                forecasted_qty   = EXCLUDED.forecasted_qty,
                confidence_low   = EXCLUDED.confidence_low,
                confidence_high  = EXCLUDED.confidence_high,
                computed_at      = EXCLUDED.computed_at
            "#,
        )
        .bind(f.id().into_uuid())
        .bind(f.product_variant_id())
        .bind(f.store_id())
        .bind(f.period().to_string())
        .bind(f.period_start())
        .bind(f.period_end())
        .bind(f.method().to_string())
        .bind(f.forecasted_qty())
        .bind(f.confidence_low())
        .bind(f.confidence_high())
        .bind(f.computed_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_latest(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<DemandForecast>, DemandPlanningError> {
        let row = sqlx::query_as::<_, ForecastRow>(SELECT_LATEST)
            .bind(product_variant_id)
            .bind(store_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(DemandForecast::try_from).transpose()
    }

    async fn find_latest_by_method(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
        method: ForecastMethod,
    ) -> Result<Option<DemandForecast>, DemandPlanningError> {
        let row = sqlx::query_as::<_, ForecastRow>(SELECT_LATEST_BY_METHOD)
            .bind(product_variant_id)
            .bind(store_id)
            .bind(method.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.map(DemandForecast::try_from).transpose()
    }

    async fn list_for_variant(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Vec<DemandForecast>, DemandPlanningError> {
        let rows = sqlx::query_as::<_, ForecastRow>(SELECT_LIST)
            .bind(product_variant_id)
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(DemandForecast::try_from).collect()
    }

    async fn delete_older_than(&self, cutoff: DateTime<Utc>) -> Result<u64, DemandPlanningError> {
        let result = sqlx::query("DELETE FROM demand_forecasts WHERE computed_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

const SELECT_LATEST: &str = r#"
SELECT id, product_variant_id, store_id, period, period_start, period_end,
       method, forecasted_qty, confidence_low, confidence_high, computed_at
FROM demand_forecasts
WHERE product_variant_id = $1 AND store_id = $2
ORDER BY computed_at DESC
LIMIT 1
"#;

const SELECT_LATEST_BY_METHOD: &str = r#"
SELECT id, product_variant_id, store_id, period, period_start, period_end,
       method, forecasted_qty, confidence_low, confidence_high, computed_at
FROM demand_forecasts
WHERE product_variant_id = $1 AND store_id = $2 AND method = $3
ORDER BY computed_at DESC
LIMIT 1
"#;

const SELECT_LIST: &str = r#"
SELECT id, product_variant_id, store_id, period, period_start, period_end,
       method, forecasted_qty, confidence_low, confidence_high, computed_at
FROM demand_forecasts
WHERE product_variant_id = $1 AND store_id = $2
ORDER BY computed_at DESC
"#;

#[derive(sqlx::FromRow)]
struct ForecastRow {
    id: Uuid,
    product_variant_id: Uuid,
    store_id: Uuid,
    period: String,
    period_start: NaiveDate,
    period_end: NaiveDate,
    method: String,
    forecasted_qty: Decimal,
    confidence_low: Decimal,
    confidence_high: Decimal,
    computed_at: DateTime<Utc>,
}

impl TryFrom<ForecastRow> for DemandForecast {
    type Error = DemandPlanningError;

    fn try_from(row: ForecastRow) -> Result<Self, Self::Error> {
        let period = ForecastPeriod::from_str(&row.period)?;
        let method = ForecastMethod::from_str(&row.method)?;
        Ok(DemandForecast::reconstitute(
            ForecastId::from_uuid(row.id),
            row.product_variant_id,
            row.store_id,
            period,
            row.period_start,
            row.period_end,
            method,
            row.forecasted_qty,
            row.confidence_low,
            row.confidence_high,
            row.computed_at,
        ))
    }
}
