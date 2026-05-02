use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::DemandPlanningError;
use crate::domain::entities::AbcClassification;
use crate::domain::repositories::AbcClassificationRepository;
use crate::domain::value_objects::{AbcClass, AbcClassificationId};

pub struct PgAbcClassificationRepository {
    pool: PgPool,
}

impl PgAbcClassificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AbcClassificationRepository for PgAbcClassificationRepository {
    async fn save_batch(
        &self,
        classifications: &[AbcClassification],
    ) -> Result<(), DemandPlanningError> {
        if classifications.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for c in classifications {
            sqlx::query(
                r#"
                INSERT INTO abc_classifications (
                    id, product_variant_id, store_id, period_start, period_end,
                    revenue_share, abc_class, classified_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (product_variant_id, store_id, period_start, period_end)
                DO UPDATE SET
                    revenue_share = EXCLUDED.revenue_share,
                    abc_class     = EXCLUDED.abc_class,
                    classified_at = EXCLUDED.classified_at
                "#,
            )
            .bind(c.id().into_uuid())
            .bind(c.product_variant_id())
            .bind(c.store_id())
            .bind(c.period_start())
            .bind(c.period_end())
            .bind(c.revenue_share())
            .bind(c.abc_class().to_string())
            .bind(c.classified_at())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn find_latest(
        &self,
        product_variant_id: Uuid,
        store_id: Uuid,
    ) -> Result<Option<AbcClassification>, DemandPlanningError> {
        let row = sqlx::query_as::<_, AbcRow>(
            r#"
            SELECT id, product_variant_id, store_id, period_start, period_end,
                   revenue_share, abc_class, classified_at
            FROM abc_classifications
            WHERE product_variant_id = $1 AND store_id = $2
            ORDER BY classified_at DESC
            LIMIT 1
            "#,
        )
        .bind(product_variant_id)
        .bind(store_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(AbcClassification::try_from).transpose()
    }

    async fn list(
        &self,
        store_id: Option<Uuid>,
        class: Option<AbcClass>,
    ) -> Result<Vec<AbcClassification>, DemandPlanningError> {
        let rows = match (store_id, class) {
            (Some(s), Some(c)) => {
                sqlx::query_as::<_, AbcRow>(
                    r#"
                SELECT id, product_variant_id, store_id, period_start, period_end,
                       revenue_share, abc_class, classified_at
                FROM abc_classifications
                WHERE store_id = $1 AND abc_class = $2
                ORDER BY classified_at DESC, revenue_share DESC
                "#,
                )
                .bind(s)
                .bind(c.to_string())
                .fetch_all(&self.pool)
                .await?
            }
            (Some(s), None) => {
                sqlx::query_as::<_, AbcRow>(
                    r#"
                SELECT id, product_variant_id, store_id, period_start, period_end,
                       revenue_share, abc_class, classified_at
                FROM abc_classifications
                WHERE store_id = $1
                ORDER BY classified_at DESC, revenue_share DESC
                "#,
                )
                .bind(s)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(c)) => {
                sqlx::query_as::<_, AbcRow>(
                    r#"
                SELECT id, product_variant_id, store_id, period_start, period_end,
                       revenue_share, abc_class, classified_at
                FROM abc_classifications
                WHERE abc_class = $1
                ORDER BY classified_at DESC, revenue_share DESC
                "#,
                )
                .bind(c.to_string())
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, AbcRow>(
                    r#"
                SELECT id, product_variant_id, store_id, period_start, period_end,
                       revenue_share, abc_class, classified_at
                FROM abc_classifications
                ORDER BY classified_at DESC, revenue_share DESC
                "#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        rows.into_iter().map(AbcClassification::try_from).collect()
    }
}

#[derive(sqlx::FromRow)]
struct AbcRow {
    id: Uuid,
    product_variant_id: Uuid,
    store_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
    revenue_share: Decimal,
    abc_class: String,
    classified_at: DateTime<Utc>,
}

impl TryFrom<AbcRow> for AbcClassification {
    type Error = DemandPlanningError;

    fn try_from(row: AbcRow) -> Result<Self, Self::Error> {
        let class = AbcClass::from_str(&row.abc_class)?;
        Ok(AbcClassification::reconstitute(
            AbcClassificationId::from_uuid(row.id),
            row.product_variant_id,
            row.store_id,
            row.period_start,
            row.period_end,
            row.revenue_share,
            class,
            row.classified_at,
        ))
    }
}
