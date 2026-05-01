use async_trait::async_trait;
use sqlx::PgPool;

use crate::CatalogError;
use crate::domain::entities::ProductImage;
use crate::domain::repositories::ProductImageRepository;
use crate::domain::value_objects::{ImageStorageProviderId, ProductImageId, ProductListingId};

pub struct PgProductImageRepository {
    pool: PgPool,
}

impl PgProductImageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str = r#"
    id, listing_id, url, storage_key, storage_provider_id,
    alt_text, sort_order, is_primary,
    content_type, size_bytes, created_at, updated_at
"#;

#[async_trait]
impl ProductImageRepository for PgProductImageRepository {
    async fn save(&self, i: &ProductImage) -> Result<(), CatalogError> {
        sqlx::query(
            r#"INSERT INTO product_images
              (id, listing_id, url, storage_key, storage_provider_id,
               alt_text, sort_order, is_primary, content_type, size_bytes,
               created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
              ON CONFLICT (id) DO UPDATE SET
                alt_text = EXCLUDED.alt_text,
                sort_order = EXCLUDED.sort_order,
                is_primary = EXCLUDED.is_primary,
                updated_at = EXCLUDED.updated_at"#,
        )
        .bind(i.id().into_uuid())
        .bind(i.listing_id().into_uuid())
        .bind(i.url())
        .bind(i.storage_key())
        .bind(i.storage_provider_uuid())
        .bind(i.alt_text())
        .bind(i.sort_order())
        .bind(i.is_primary())
        .bind(i.content_type())
        .bind(i.size_bytes())
        .bind(i.created_at())
        .bind(i.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: ProductImageId) -> Result<Option<ProductImage>, CatalogError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM product_images WHERE id = $1 LIMIT 1");
        let row = sqlx::query_as::<_, ImageRow>(&sql)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_listing(
        &self,
        listing_id: ProductListingId,
    ) -> Result<Vec<ProductImage>, CatalogError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM product_images
             WHERE listing_id = $1 ORDER BY sort_order ASC, created_at ASC"
        );
        let rows = sqlx::query_as::<_, ImageRow>(&sql)
            .bind(listing_id.into_uuid())
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count_by_listing(&self, listing_id: ProductListingId) -> Result<i64, CatalogError> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM product_images WHERE listing_id = $1")
                .bind(listing_id.into_uuid())
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    async fn delete(&self, id: ProductImageId) -> Result<(), CatalogError> {
        let result = sqlx::query("DELETE FROM product_images WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::ImageNotFound(id.into_uuid()));
        }
        Ok(())
    }

    async fn reorder(
        &self,
        listing_id: ProductListingId,
        image_ids: Vec<ProductImageId>,
    ) -> Result<(), CatalogError> {
        // Single UPDATE per image. For 20-item-max galleries this is fine.
        let mut tx = self.pool.begin().await?;
        for (idx, id) in image_ids.iter().enumerate() {
            sqlx::query(
                "UPDATE product_images SET sort_order = $3, updated_at = NOW()
                 WHERE id = $1 AND listing_id = $2",
            )
            .bind(id.into_uuid())
            .bind(listing_id.into_uuid())
            .bind(idx as i32)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn unset_primary_except(
        &self,
        listing_id: ProductListingId,
        keep: ProductImageId,
    ) -> Result<(), CatalogError> {
        sqlx::query(
            "UPDATE product_images SET is_primary = false, updated_at = NOW()
             WHERE listing_id = $1 AND id <> $2 AND is_primary = true",
        )
        .bind(listing_id.into_uuid())
        .bind(keep.into_uuid())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ImageRow {
    id: uuid::Uuid,
    listing_id: uuid::Uuid,
    url: String,
    storage_key: String,
    storage_provider_id: Option<uuid::Uuid>,
    alt_text: Option<String>,
    sort_order: i32,
    is_primary: bool,
    content_type: Option<String>,
    size_bytes: Option<i64>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ImageRow> for ProductImage {
    fn from(row: ImageRow) -> Self {
        ProductImage::reconstitute(
            ProductImageId::from_uuid(row.id),
            ProductListingId::from_uuid(row.listing_id),
            row.url,
            row.storage_key,
            row.storage_provider_id
                .map(ImageStorageProviderId::from_uuid),
            row.alt_text,
            row.sort_order,
            row.is_primary,
            row.content_type,
            row.size_bytes,
            row.created_at,
            row.updated_at,
        )
    }
}
