use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ServiceOrdersError;
use crate::domain::entities::ServiceOrderItem;
use crate::domain::repositories::ServiceOrderItemRepository;
use crate::domain::value_objects::{ServiceOrderId, ServiceOrderItemId, ServiceOrderItemType};

pub struct PgServiceOrderItemRepository {
    pool: PgPool,
}

impl PgServiceOrderItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServiceOrderItemRepository for PgServiceOrderItemRepository {
    async fn save(&self, i: &ServiceOrderItem) -> Result<(), ServiceOrdersError> {
        sqlx::query(
            r#"
            INSERT INTO service_order_items (
                id, service_order_id, item_type, description,
                quantity, unit_price, total,
                product_id, variant_id, tax_rate, tax_amount, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(i.id().into_uuid())
        .bind(i.service_order_id().into_uuid())
        .bind(i.item_type().as_str())
        .bind(i.description())
        .bind(i.quantity())
        .bind(i.unit_price())
        .bind(i.total())
        .bind(i.product_id())
        .bind(i.variant_id())
        .bind(i.tax_rate())
        .bind(i.tax_amount())
        .bind(i.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, i: &ServiceOrderItem) -> Result<(), ServiceOrdersError> {
        let result = sqlx::query(
            r#"
            UPDATE service_order_items
               SET description = $2,
                   quantity    = $3,
                   unit_price  = $4,
                   total       = $5,
                   product_id  = $6,
                   variant_id  = $7,
                   tax_rate    = $8,
                   tax_amount  = $9
             WHERE id = $1
            "#,
        )
        .bind(i.id().into_uuid())
        .bind(i.description())
        .bind(i.quantity())
        .bind(i.unit_price())
        .bind(i.total())
        .bind(i.product_id())
        .bind(i.variant_id())
        .bind(i.tax_rate())
        .bind(i.tax_amount())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(ServiceOrdersError::ItemNotFound(i.id().into_uuid()));
        }
        Ok(())
    }

    async fn delete(&self, id: ServiceOrderItemId) -> Result<(), ServiceOrdersError> {
        let result = sqlx::query("DELETE FROM service_order_items WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(ServiceOrdersError::ItemNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ServiceOrderItemId,
    ) -> Result<Option<ServiceOrderItem>, ServiceOrdersError> {
        let row = sqlx::query_as::<_, ItemRow>(SELECT_BY_ID)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(ServiceOrderItem::try_from).transpose()
    }

    async fn list_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Vec<ServiceOrderItem>, ServiceOrdersError> {
        let rows = sqlx::query_as::<_, ItemRow>(LIST_BY_ORDER)
            .bind(order_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter().map(ServiceOrderItem::try_from).collect()
    }

    async fn subtotal_by_order(
        &self,
        order_id: ServiceOrderId,
    ) -> Result<Decimal, ServiceOrdersError> {
        let row: (Option<Decimal>,) = sqlx::query_as(
            "SELECT COALESCE(SUM(total), 0)::NUMERIC FROM service_order_items WHERE service_order_id = $1",
        )
        .bind(order_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0.unwrap_or(Decimal::ZERO))
    }
}

const SELECT_BY_ID: &str = r#"
SELECT id, service_order_id, item_type, description,
       quantity, unit_price, total, product_id, variant_id,
       tax_rate, tax_amount, created_at
FROM service_order_items
WHERE id = $1
"#;

const LIST_BY_ORDER: &str = r#"
SELECT id, service_order_id, item_type, description,
       quantity, unit_price, total, product_id, variant_id,
       tax_rate, tax_amount, created_at
FROM service_order_items
WHERE service_order_id = $1
ORDER BY created_at
"#;

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    service_order_id: Uuid,
    item_type: String,
    description: String,
    quantity: Decimal,
    unit_price: Decimal,
    total: Decimal,
    product_id: Option<Uuid>,
    variant_id: Option<Uuid>,
    tax_rate: Decimal,
    tax_amount: Decimal,
    created_at: DateTime<Utc>,
}

impl TryFrom<ItemRow> for ServiceOrderItem {
    type Error = ServiceOrdersError;
    fn try_from(r: ItemRow) -> Result<Self, ServiceOrdersError> {
        Ok(ServiceOrderItem::reconstitute(
            ServiceOrderItemId::from_uuid(r.id),
            ServiceOrderId::from_uuid(r.service_order_id),
            ServiceOrderItemType::from_str(&r.item_type)?,
            r.description,
            r.quantity,
            r.unit_price,
            r.total,
            r.product_id,
            r.variant_id,
            r.tax_rate,
            r.tax_amount,
            r.created_at,
        ))
    }
}
