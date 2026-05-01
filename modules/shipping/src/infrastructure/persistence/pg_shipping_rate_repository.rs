use async_trait::async_trait;
use sqlx::PgPool;

use crate::ShippingError;
use crate::domain::entities::ShippingRate;
use crate::domain::repositories::ShippingRateRepository;
use crate::domain::value_objects::{
    ShippingMethodId, ShippingRateId, ShippingRateType, ShippingZoneId,
};

pub struct PgShippingRateRepository {
    pool: PgPool,
}

impl PgShippingRateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShippingRateRepository for PgShippingRateRepository {
    async fn save(&self, r: &ShippingRate) -> Result<(), ShippingError> {
        sqlx::query(
            r#"INSERT INTO shipping_rates
              (id, shipping_method_id, shipping_zone_id, rate_type,
               base_rate, per_kg_rate, free_shipping_threshold,
               min_order_amount, max_weight_kg, currency,
               available_days, available_hour_start, available_hour_end,
               is_active, created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)"#,
        )
        .bind(r.id().into_uuid())
        .bind(r.shipping_method_id().into_uuid())
        .bind(r.shipping_zone_id().into_uuid())
        .bind(r.rate_type().to_string())
        .bind(r.base_rate())
        .bind(r.per_kg_rate())
        .bind(r.free_shipping_threshold())
        .bind(r.min_order_amount())
        .bind(r.max_weight_kg())
        .bind(r.currency())
        .bind(r.available_days())
        .bind(r.available_hour_start())
        .bind(r.available_hour_end())
        .bind(r.is_active())
        .bind(r.created_at())
        .bind(r.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: ShippingRateId) -> Result<Option<ShippingRate>, ShippingError> {
        let row = sqlx::query_as::<_, RateRow>(SELECT_SQL)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_method_and_zone(
        &self,
        method_id: ShippingMethodId,
        zone_id: ShippingZoneId,
    ) -> Result<Vec<ShippingRate>, ShippingError> {
        let rows = sqlx::query_as::<_, RateRow>(
            r#"SELECT id, shipping_method_id, shipping_zone_id, rate_type,
                   base_rate, per_kg_rate, free_shipping_threshold,
                   min_order_amount, max_weight_kg, currency,
                   available_days, available_hour_start, available_hour_end,
                   is_active, created_at, updated_at
               FROM shipping_rates
               WHERE shipping_method_id = $1 AND shipping_zone_id = $2
               ORDER BY base_rate ASC"#,
        )
        .bind(method_id.into_uuid())
        .bind(zone_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_by_zone(
        &self,
        zone_id: ShippingZoneId,
    ) -> Result<Vec<ShippingRate>, ShippingError> {
        let rows = sqlx::query_as::<_, RateRow>(
            r#"SELECT id, shipping_method_id, shipping_zone_id, rate_type,
                   base_rate, per_kg_rate, free_shipping_threshold,
                   min_order_amount, max_weight_kg, currency,
                   available_days, available_hour_start, available_hour_end,
                   is_active, created_at, updated_at
               FROM shipping_rates WHERE shipping_zone_id = $1"#,
        )
        .bind(zone_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update(&self, r: &ShippingRate) -> Result<(), ShippingError> {
        let result = sqlx::query(
            r#"UPDATE shipping_rates
               SET base_rate=$2, per_kg_rate=$3, free_shipping_threshold=$4,
                   available_days=$5, available_hour_start=$6, available_hour_end=$7,
                   is_active=$8, updated_at=$9
               WHERE id=$1"#,
        )
        .bind(r.id().into_uuid())
        .bind(r.base_rate())
        .bind(r.per_kg_rate())
        .bind(r.free_shipping_threshold())
        .bind(r.available_days())
        .bind(r.available_hour_start())
        .bind(r.available_hour_end())
        .bind(r.is_active())
        .bind(r.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShippingRateNotFound(r.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: ShippingRateId) -> Result<(), ShippingError> {
        let result = sqlx::query("DELETE FROM shipping_rates WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ShippingError::ShippingRateNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

const SELECT_SQL: &str = r#"
SELECT id, shipping_method_id, shipping_zone_id, rate_type,
       base_rate, per_kg_rate, free_shipping_threshold,
       min_order_amount, max_weight_kg, currency,
       available_days, available_hour_start, available_hour_end,
       is_active, created_at, updated_at
FROM shipping_rates WHERE id = $1
"#;

#[derive(sqlx::FromRow)]
struct RateRow {
    id: uuid::Uuid,
    shipping_method_id: uuid::Uuid,
    shipping_zone_id: uuid::Uuid,
    rate_type: String,
    base_rate: rust_decimal::Decimal,
    per_kg_rate: rust_decimal::Decimal,
    free_shipping_threshold: Option<rust_decimal::Decimal>,
    min_order_amount: Option<rust_decimal::Decimal>,
    max_weight_kg: Option<rust_decimal::Decimal>,
    currency: String,
    available_days: Option<Vec<i16>>,
    available_hour_start: Option<i16>,
    available_hour_end: Option<i16>,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<RateRow> for ShippingRate {
    type Error = ShippingError;
    fn try_from(row: RateRow) -> Result<Self, Self::Error> {
        let rate_type: ShippingRateType = row
            .rate_type
            .parse()
            .map_err(|_| ShippingError::InvalidRateType)?;
        Ok(ShippingRate::reconstitute(
            ShippingRateId::from_uuid(row.id),
            ShippingMethodId::from_uuid(row.shipping_method_id),
            ShippingZoneId::from_uuid(row.shipping_zone_id),
            rate_type,
            row.base_rate,
            row.per_kg_rate,
            row.free_shipping_threshold,
            row.min_order_amount,
            row.max_weight_kg,
            row.currency,
            row.available_days,
            row.available_hour_start,
            row.available_hour_end,
            row.is_active,
            row.created_at,
            row.updated_at,
        ))
    }
}
