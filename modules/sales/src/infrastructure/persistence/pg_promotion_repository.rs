//! PostgreSQL PromotionRepository implementation

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::SalesError;
use crate::domain::entities::Promotion;
use crate::domain::repositories::{PromotionFilter, PromotionRepository};
use crate::domain::value_objects::{PromotionId, PromotionStatus, PromotionType};
use identity::UserId;

/// Row type for reading promotions from the database
#[derive(Debug, sqlx::FromRow)]
struct PromotionRow {
    id: Uuid,
    code: String,
    name: String,
    description: Option<String>,
    promotion_type: String,
    status: String,
    discount_value: Decimal,
    buy_quantity: Option<i32>,
    get_quantity: Option<i32>,
    minimum_purchase: Decimal,
    maximum_discount: Option<Decimal>,
    usage_limit: Option<i32>,
    usage_count: i32,
    per_customer_limit: Option<i32>,
    applies_to: String,
    product_ids: Option<Vec<Uuid>>,
    category_ids: Option<Vec<Uuid>>,
    start_date: DateTime<Utc>,
    end_date: Option<DateTime<Utc>>,
    store_id: Option<Uuid>,
    created_by_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<PromotionRow> for Promotion {
    type Error = SalesError;

    fn try_from(row: PromotionRow) -> Result<Self, Self::Error> {
        let promotion_type = PromotionType::from_str(&row.promotion_type)?;
        let status = PromotionStatus::from_str(&row.status)?;

        Ok(Promotion::reconstitute(
            PromotionId::from_uuid(row.id),
            row.code,
            row.name,
            row.description,
            promotion_type,
            status,
            row.discount_value,
            row.buy_quantity,
            row.get_quantity,
            row.minimum_purchase,
            row.maximum_discount,
            row.usage_limit,
            row.usage_count,
            row.per_customer_limit,
            row.applies_to,
            row.product_ids.unwrap_or_default(),
            row.category_ids.unwrap_or_default(),
            row.start_date,
            row.end_date,
            row.store_id,
            UserId::from_uuid(row.created_by_id),
            row.created_at,
            row.updated_at,
        ))
    }
}

pub struct PgPromotionRepository {
    pool: PgPool,
}

impl PgPromotionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PromotionRepository for PgPromotionRepository {
    async fn save(&self, promotion: &Promotion) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            INSERT INTO promotions (
                id, code, name, description, promotion_type, status,
                discount_value, buy_quantity, get_quantity, minimum_purchase,
                maximum_discount, usage_limit, usage_count, per_customer_limit,
                applies_to, product_ids, category_ids, start_date, end_date,
                store_id, created_by_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            "#,
        )
        .bind(promotion.id().into_uuid())
        .bind(promotion.code())
        .bind(promotion.name())
        .bind(promotion.description())
        .bind(promotion.promotion_type().to_string())
        .bind(promotion.status().to_string())
        .bind(promotion.discount_value())
        .bind(promotion.buy_quantity())
        .bind(promotion.get_quantity())
        .bind(promotion.minimum_purchase())
        .bind(promotion.maximum_discount())
        .bind(promotion.usage_limit())
        .bind(promotion.usage_count())
        .bind(promotion.per_customer_limit())
        .bind(promotion.applies_to())
        .bind(promotion.product_ids())
        .bind(promotion.category_ids())
        .bind(promotion.start_date())
        .bind(promotion.end_date())
        .bind(promotion.store_id())
        .bind(promotion.created_by_id().into_uuid())
        .bind(promotion.created_at())
        .bind(promotion.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: PromotionId) -> Result<Option<Promotion>, SalesError> {
        let row = sqlx::query_as::<_, PromotionRow>(r#"SELECT * FROM promotions WHERE id = $1"#)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;

        row.map(Promotion::try_from).transpose()
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Promotion>, SalesError> {
        let row = sqlx::query_as::<_, PromotionRow>(r#"SELECT * FROM promotions WHERE code = $1"#)
            .bind(code)
            .fetch_optional(&self.pool)
            .await?;

        row.map(Promotion::try_from).transpose()
    }

    async fn update(&self, promotion: &Promotion) -> Result<(), SalesError> {
        sqlx::query(
            r#"
            UPDATE promotions SET
                name = $2, description = $3, status = $4,
                discount_value = $5, buy_quantity = $6, get_quantity = $7,
                minimum_purchase = $8, maximum_discount = $9,
                usage_limit = $10, usage_count = $11, per_customer_limit = $12,
                applies_to = $13, product_ids = $14, category_ids = $15,
                start_date = $16, end_date = $17, store_id = $18,
                updated_at = $19
            WHERE id = $1
            "#,
        )
        .bind(promotion.id().into_uuid())
        .bind(promotion.name())
        .bind(promotion.description())
        .bind(promotion.status().to_string())
        .bind(promotion.discount_value())
        .bind(promotion.buy_quantity())
        .bind(promotion.get_quantity())
        .bind(promotion.minimum_purchase())
        .bind(promotion.maximum_discount())
        .bind(promotion.usage_limit())
        .bind(promotion.usage_count())
        .bind(promotion.per_customer_limit())
        .bind(promotion.applies_to())
        .bind(promotion.product_ids())
        .bind(promotion.category_ids())
        .bind(promotion.start_date())
        .bind(promotion.end_date())
        .bind(promotion.store_id())
        .bind(promotion.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: PromotionFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Promotion>, i64), SalesError> {
        let offset = (page - 1) * page_size;

        let mut conditions = vec!["1=1".to_string()];
        if let Some(ref status) = filter.status {
            conditions.push(format!("status = '{}'", status));
        }
        if let Some(store_id) = filter.store_id {
            conditions.push(format!("(store_id = '{}' OR store_id IS NULL)", store_id));
        }
        if let Some(ref search) = filter.search {
            conditions.push(format!(
                "(code ILIKE '%{}%' OR name ILIKE '%{}%')",
                search, search
            ));
        }

        let where_clause = conditions.join(" AND ");

        let count_query = format!(
            "SELECT COUNT(*) as count FROM promotions WHERE {}",
            where_clause
        );
        let count: (i64,) = sqlx::query_as(&count_query).fetch_one(&self.pool).await?;

        let data_query = format!(
            "SELECT * FROM promotions WHERE {} ORDER BY created_at DESC LIMIT {} OFFSET {}",
            where_clause, page_size, offset
        );
        let rows: Vec<PromotionRow> = sqlx::query_as(&data_query).fetch_all(&self.pool).await?;

        let promotions: Result<Vec<Promotion>, _> =
            rows.into_iter().map(Promotion::try_from).collect();

        Ok((promotions?, count.0))
    }

    async fn find_active_by_store(
        &self,
        store_id: Option<Uuid>,
    ) -> Result<Vec<Promotion>, SalesError> {
        let rows = if let Some(sid) = store_id {
            sqlx::query_as::<_, PromotionRow>(
                r#"
                SELECT * FROM promotions
                WHERE status = 'active'
                  AND start_date <= NOW()
                  AND (end_date IS NULL OR end_date > NOW())
                  AND (store_id = $1 OR store_id IS NULL)
                ORDER BY created_at DESC
                "#,
            )
            .bind(sid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, PromotionRow>(
                r#"
                SELECT * FROM promotions
                WHERE status = 'active'
                  AND start_date <= NOW()
                  AND (end_date IS NULL OR end_date > NOW())
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter().map(Promotion::try_from).collect()
    }
}
