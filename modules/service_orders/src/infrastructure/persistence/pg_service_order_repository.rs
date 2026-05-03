use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::ServiceOrder;
use crate::domain::repositories::{ListServiceOrdersFilters, ServiceOrderRepository};
use crate::domain::value_objects::{
    AssetId, ServiceOrderId, ServiceOrderPriority, ServiceOrderStatus,
};

pub struct PgServiceOrderRepository {
    pool: PgPool,
}

impl PgServiceOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServiceOrderRepository for PgServiceOrderRepository {
    async fn save(&self, o: &ServiceOrder) -> Result<(), ServiceOrdersError> {
        sqlx::query(
            r#"
            INSERT INTO service_orders (
                id, store_id, asset_id, customer_id,
                customer_name, customer_email, customer_phone,
                status, priority, intake_notes, intake_at, intake_by_user_id,
                promised_at, delivered_at, generated_sale_id,
                canceled_reason, canceled_at, public_token, total_amount,
                created_at, updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16, $17, $18, $19, $20, $21
            )
            "#,
        )
        .bind(o.id().into_uuid())
        .bind(o.store_id())
        .bind(o.asset_id().into_uuid())
        .bind(o.customer_id())
        .bind(o.customer_name())
        .bind(o.customer_email())
        .bind(o.customer_phone())
        .bind(o.status().as_str())
        .bind(o.priority().as_str())
        .bind(o.intake_notes())
        .bind(o.intake_at())
        .bind(o.intake_by_user_id())
        .bind(o.promised_at())
        .bind(o.delivered_at())
        .bind(o.generated_sale_id())
        .bind(o.canceled_reason())
        .bind(o.canceled_at())
        .bind(o.public_token())
        .bind(o.total_amount())
        .bind(o.created_at())
        .bind(o.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, o: &ServiceOrder) -> Result<(), ServiceOrdersError> {
        let result = sqlx::query(
            r#"
            UPDATE service_orders
               SET status            = $2,
                   priority          = $3,
                   intake_notes      = $4,
                   promised_at       = $5,
                   delivered_at      = $6,
                   generated_sale_id = $7,
                   canceled_reason   = $8,
                   canceled_at       = $9,
                   total_amount      = $10,
                   updated_at        = $11
             WHERE id = $1
            "#,
        )
        .bind(o.id().into_uuid())
        .bind(o.status().as_str())
        .bind(o.priority().as_str())
        .bind(o.intake_notes())
        .bind(o.promised_at())
        .bind(o.delivered_at())
        .bind(o.generated_sale_id())
        .bind(o.canceled_reason())
        .bind(o.canceled_at())
        .bind(o.total_amount())
        .bind(o.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ServiceOrdersError::ServiceOrderNotFound(o.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ServiceOrderId,
    ) -> Result<Option<ServiceOrder>, ServiceOrdersError> {
        let row = sqlx::query_as::<_, OrderRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(ServiceOrder::try_from).transpose()
    }

    async fn find_by_public_token(
        &self,
        token: &str,
    ) -> Result<Option<ServiceOrder>, ServiceOrdersError> {
        let row = sqlx::query_as::<_, OrderRow>(SELECT_BY_TOKEN)
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;
        row.map(ServiceOrder::try_from).transpose()
    }

    async fn list(
        &self,
        filters: ListServiceOrdersFilters,
    ) -> Result<Vec<ServiceOrder>, ServiceOrdersError> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, store_id, asset_id, customer_id, customer_name, \
             customer_email, customer_phone, status, priority, intake_notes, \
             intake_at, intake_by_user_id, promised_at, delivered_at, \
             generated_sale_id, canceled_reason, canceled_at, public_token, \
             total_amount, created_at, updated_at \
             FROM service_orders WHERE 1=1",
        );
        if let Some(store_id) = filters.store_id {
            qb.push(" AND store_id = ").push_bind(store_id);
        }
        if let Some(customer_id) = filters.customer_id {
            qb.push(" AND customer_id = ").push_bind(customer_id);
        }
        if let Some(asset_id) = filters.asset_id {
            qb.push(" AND asset_id = ").push_bind(asset_id.into_uuid());
        }
        if let Some(status) = filters.status {
            qb.push(" AND status = ")
                .push_bind(status.as_str().to_string());
        }
        if let Some(from) = filters.from {
            qb.push(" AND intake_at >= ").push_bind(from);
        }
        if let Some(to) = filters.to {
            qb.push(" AND intake_at < ").push_bind(to);
        }
        qb.push(" ORDER BY intake_at DESC");
        if let Some(limit) = filters.limit {
            qb.push(" LIMIT ").push_bind(limit);
        } else {
            qb.push(" LIMIT 200");
        }
        let rows: Vec<OrderRow> = qb.build_query_as().fetch_all(&self.pool).await?;
        rows.into_iter().map(ServiceOrder::try_from).collect()
    }

    async fn list_by_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<Vec<ServiceOrder>, ServiceOrdersError> {
        let rows = sqlx::query_as::<_, OrderRow>(LIST_BY_ASSET)
            .bind(asset_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(ServiceOrder::try_from).collect()
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, store_id, asset_id, customer_id,
       customer_name, customer_email, customer_phone,
       status, priority, intake_notes, intake_at, intake_by_user_id,
       promised_at, delivered_at, generated_sale_id,
       canceled_reason, canceled_at, public_token, total_amount,
       created_at, updated_at
FROM service_orders
WHERE id = $1
"#;

const SELECT_BY_TOKEN: &str = r#"
SELECT id, store_id, asset_id, customer_id,
       customer_name, customer_email, customer_phone,
       status, priority, intake_notes, intake_at, intake_by_user_id,
       promised_at, delivered_at, generated_sale_id,
       canceled_reason, canceled_at, public_token, total_amount,
       created_at, updated_at
FROM service_orders
WHERE public_token = $1
"#;

const LIST_BY_ASSET: &str = r#"
SELECT id, store_id, asset_id, customer_id,
       customer_name, customer_email, customer_phone,
       status, priority, intake_notes, intake_at, intake_by_user_id,
       promised_at, delivered_at, generated_sale_id,
       canceled_reason, canceled_at, public_token, total_amount,
       created_at, updated_at
FROM service_orders
WHERE asset_id = $1
ORDER BY intake_at DESC
"#;

#[derive(sqlx::FromRow)]
struct OrderRow {
    id: Uuid,
    store_id: Uuid,
    asset_id: Uuid,
    customer_id: Option<Uuid>,
    customer_name: String,
    customer_email: String,
    customer_phone: Option<String>,
    status: String,
    priority: String,
    intake_notes: Option<String>,
    intake_at: DateTime<Utc>,
    intake_by_user_id: Option<Uuid>,
    promised_at: Option<DateTime<Utc>>,
    delivered_at: Option<DateTime<Utc>>,
    generated_sale_id: Option<Uuid>,
    canceled_reason: Option<String>,
    canceled_at: Option<DateTime<Utc>>,
    public_token: String,
    total_amount: Decimal,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<OrderRow> for ServiceOrder {
    type Error = ServiceOrdersError;
    fn try_from(r: OrderRow) -> Result<Self, ServiceOrdersError> {
        Ok(ServiceOrder::reconstitute(
            ServiceOrderId::from_uuid(r.id),
            r.store_id,
            AssetId::from_uuid(r.asset_id),
            r.customer_id,
            r.customer_name,
            r.customer_email,
            r.customer_phone,
            ServiceOrderStatus::from_str(&r.status)?,
            ServiceOrderPriority::from_str(&r.priority)?,
            r.intake_notes,
            r.intake_at,
            r.intake_by_user_id,
            r.promised_at,
            r.delivered_at,
            r.generated_sale_id,
            r.canceled_reason,
            r.canceled_at,
            r.public_token,
            r.total_amount,
            r.created_at,
            r.updated_at,
        ))
    }
}
