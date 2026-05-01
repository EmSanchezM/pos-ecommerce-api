use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::CatalogError;
use crate::domain::entities::ProductReview;
use crate::domain::repositories::ProductReviewRepository;
use crate::domain::value_objects::{ProductListingId, ProductReviewId};
use identity::UserId;
use sales::CustomerId;

pub struct PgProductReviewRepository {
    pool: PgPool,
}

impl PgProductReviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str = r#"
    id, listing_id, customer_id, rating, title, comment,
    is_verified_purchase, is_approved,
    approved_by_id, approved_at, created_at, updated_at
"#;

#[async_trait]
impl ProductReviewRepository for PgProductReviewRepository {
    async fn save(&self, r: &ProductReview) -> Result<(), CatalogError> {
        sqlx::query(
            r#"INSERT INTO product_reviews
              (id, listing_id, customer_id, rating, title, comment,
               is_verified_purchase, is_approved,
               approved_by_id, approved_at, created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"#,
        )
        .bind(r.id().into_uuid())
        .bind(r.listing_id().into_uuid())
        .bind(r.customer_id().into_uuid())
        .bind(r.rating())
        .bind(r.title())
        .bind(r.comment())
        .bind(r.is_verified_purchase())
        .bind(r.is_approved())
        .bind(r.approved_by_id().map(|u| u.into_uuid()))
        .bind(r.approved_at())
        .bind(r.created_at())
        .bind(r.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db)
                if db
                    .constraint()
                    .map(|c| c.contains("listing_customer"))
                    .unwrap_or(false) =>
            {
                CatalogError::DuplicateReview
            }
            _ => CatalogError::Database(e),
        })?;
        Ok(())
    }

    async fn find_by_id(&self, id: ProductReviewId) -> Result<Option<ProductReview>, CatalogError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM product_reviews WHERE id = $1 LIMIT 1");
        let row = sqlx::query_as::<_, ReviewRow>(&sql)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn find_approved_by_listing(
        &self,
        listing_id: ProductListingId,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ProductReview>, i64), CatalogError> {
        let offset = (page - 1) * page_size;
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM product_reviews
             WHERE listing_id = $1 AND is_approved = true
             ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        );
        let rows = sqlx::query_as::<_, ReviewRow>(&sql)
            .bind(listing_id.into_uuid())
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM product_reviews WHERE listing_id = $1 AND is_approved = true",
        )
        .bind(listing_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;
        Ok((rows.into_iter().map(Into::into).collect(), total.0))
    }

    async fn find_pending(
        &self,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ProductReview>, i64), CatalogError> {
        let offset = (page - 1) * page_size;
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM product_reviews
             WHERE is_approved = false
             ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        );
        let rows = sqlx::query_as::<_, ReviewRow>(&sql)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
        let total: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM product_reviews WHERE is_approved = false")
                .fetch_one(&self.pool)
                .await?;
        Ok((rows.into_iter().map(Into::into).collect(), total.0))
    }

    async fn find_by_customer_and_listing(
        &self,
        customer_id: CustomerId,
        listing_id: ProductListingId,
    ) -> Result<Option<ProductReview>, CatalogError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM product_reviews
             WHERE customer_id = $1 AND listing_id = $2 LIMIT 1"
        );
        let row = sqlx::query_as::<_, ReviewRow>(&sql)
            .bind(customer_id.into_uuid())
            .bind(listing_id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn update(&self, r: &ProductReview) -> Result<(), CatalogError> {
        let result = sqlx::query(
            r#"UPDATE product_reviews SET
                rating=$2, title=$3, comment=$4,
                is_verified_purchase=$5, is_approved=$6,
                approved_by_id=$7, approved_at=$8, updated_at=$9
               WHERE id=$1"#,
        )
        .bind(r.id().into_uuid())
        .bind(r.rating())
        .bind(r.title())
        .bind(r.comment())
        .bind(r.is_verified_purchase())
        .bind(r.is_approved())
        .bind(r.approved_by_id().map(|u| u.into_uuid()))
        .bind(r.approved_at())
        .bind(r.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::ReviewNotFound(r.id().into_uuid()));
        }
        Ok(())
    }

    async fn average_rating(
        &self,
        listing_id: ProductListingId,
    ) -> Result<(Decimal, i64), CatalogError> {
        let row: (Option<Decimal>, i64) = sqlx::query_as(
            r#"SELECT
                 COALESCE(AVG(rating::numeric), 0)::numeric AS avg,
                 COUNT(*) AS count
               FROM product_reviews
               WHERE listing_id = $1 AND is_approved = true"#,
        )
        .bind(listing_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;
        Ok((row.0.unwrap_or(Decimal::ZERO), row.1))
    }

    async fn delete(&self, id: ProductReviewId) -> Result<(), CatalogError> {
        let result = sqlx::query("DELETE FROM product_reviews WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::ReviewNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn customer_purchased_product(
        &self,
        customer_id: CustomerId,
        listing_id: ProductListingId,
    ) -> Result<bool, CatalogError> {
        // Resolve listing → product, then check if customer has any completed
        // sale containing that product (via sale_items).
        let row: Option<(bool,)> = sqlx::query_as(
            r#"SELECT EXISTS (
                 SELECT 1
                 FROM sales s
                 JOIN sale_items si ON si.sale_id = s.id
                 JOIN product_listings pl ON pl.id = $2
                 WHERE s.customer_id = $1
                   AND s.status = 'completed'
                   AND si.product_id = pl.product_id
               )"#,
        )
        .bind(customer_id.into_uuid())
        .bind(listing_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0).unwrap_or(false))
    }
}

#[derive(sqlx::FromRow)]
struct ReviewRow {
    id: uuid::Uuid,
    listing_id: uuid::Uuid,
    customer_id: uuid::Uuid,
    rating: i16,
    title: Option<String>,
    comment: Option<String>,
    is_verified_purchase: bool,
    is_approved: bool,
    approved_by_id: Option<uuid::Uuid>,
    approved_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ReviewRow> for ProductReview {
    fn from(row: ReviewRow) -> Self {
        ProductReview::reconstitute(
            ProductReviewId::from_uuid(row.id),
            ProductListingId::from_uuid(row.listing_id),
            CustomerId::from_uuid(row.customer_id),
            row.rating,
            row.title,
            row.comment,
            row.is_verified_purchase,
            row.is_approved,
            row.approved_by_id.map(UserId::from_uuid),
            row.approved_at,
            row.created_at,
            row.updated_at,
        )
    }
}
