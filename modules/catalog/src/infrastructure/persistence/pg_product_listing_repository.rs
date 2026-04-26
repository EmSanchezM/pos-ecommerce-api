use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::CatalogError;
use crate::domain::entities::ProductListing;
use crate::domain::repositories::{CatalogFilter, ProductListingRepository};
use crate::domain::value_objects::ProductListingId;
use identity::StoreId;

pub struct PgProductListingRepository {
    pool: PgPool,
}

impl PgProductListingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_COLUMNS: &str = r#"
    id, store_id, product_id, slug, title,
    short_description, long_description,
    is_published, is_featured,
    seo_title, seo_description, seo_keywords,
    sort_order, view_count, created_at, updated_at
"#;

#[async_trait]
impl ProductListingRepository for PgProductListingRepository {
    async fn save(&self, l: &ProductListing) -> Result<(), CatalogError> {
        sqlx::query(
            r#"INSERT INTO product_listings
              (id, store_id, product_id, slug, title,
               short_description, long_description,
               is_published, is_featured,
               seo_title, seo_description, seo_keywords,
               sort_order, view_count, created_at, updated_at)
              VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)"#,
        )
        .bind(l.id().into_uuid())
        .bind(l.store_id().into_uuid())
        .bind(l.product_id())
        .bind(l.slug())
        .bind(l.title())
        .bind(l.short_description())
        .bind(l.long_description())
        .bind(l.is_published())
        .bind(l.is_featured())
        .bind(l.seo_title())
        .bind(l.seo_description())
        .bind(l.seo_keywords())
        .bind(l.sort_order())
        .bind(l.view_count())
        .bind(l.created_at())
        .bind(l.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: ProductListingId,
    ) -> Result<Option<ProductListing>, CatalogError> {
        let sql = format!("SELECT {SELECT_COLUMNS} FROM product_listings WHERE id = $1 LIMIT 1");
        let row = sqlx::query_as::<_, ListingRow>(&sql)
            .bind(id.into_uuid())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_slug(
        &self,
        store_id: StoreId,
        slug: &str,
    ) -> Result<Option<ProductListing>, CatalogError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM product_listings WHERE store_id = $1 AND slug = $2 LIMIT 1"
        );
        let row = sqlx::query_as::<_, ListingRow>(&sql)
            .bind(store_id.into_uuid())
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn find_by_product_id(
        &self,
        product_id: Uuid,
    ) -> Result<Option<ProductListing>, CatalogError> {
        let sql =
            format!("SELECT {SELECT_COLUMNS} FROM product_listings WHERE product_id = $1 LIMIT 1");
        let row = sqlx::query_as::<_, ListingRow>(&sql)
            .bind(product_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(Into::into))
    }

    async fn update(&self, l: &ProductListing) -> Result<(), CatalogError> {
        let result = sqlx::query(
            r#"UPDATE product_listings SET
                slug=$2, title=$3,
                short_description=$4, long_description=$5,
                is_published=$6, is_featured=$7,
                seo_title=$8, seo_description=$9, seo_keywords=$10,
                sort_order=$11, view_count=$12, updated_at=$13
               WHERE id=$1"#,
        )
        .bind(l.id().into_uuid())
        .bind(l.slug())
        .bind(l.title())
        .bind(l.short_description())
        .bind(l.long_description())
        .bind(l.is_published())
        .bind(l.is_featured())
        .bind(l.seo_title())
        .bind(l.seo_description())
        .bind(l.seo_keywords())
        .bind(l.sort_order())
        .bind(l.view_count())
        .bind(l.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::ListingNotFound(l.id().into_uuid()));
        }
        Ok(())
    }

    async fn find_paginated(
        &self,
        filter: CatalogFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ProductListing>, i64), CatalogError> {
        let offset = (page - 1) * page_size;

        // We join inventory.products lazily ONLY when min_price/max_price filters
        // are present, since that requires the products.price column.
        let needs_product_join = filter.min_price.is_some() || filter.max_price.is_some();
        let from_clause = if needs_product_join {
            "FROM product_listings l JOIN products p ON p.id = l.product_id"
        } else {
            "FROM product_listings l"
        };
        let select_cols = "l.id, l.store_id, l.product_id, l.slug, l.title,
             l.short_description, l.long_description,
             l.is_published, l.is_featured,
             l.seo_title, l.seo_description, l.seo_keywords,
             l.sort_order, l.view_count, l.created_at, l.updated_at";

        let mut data =
            QueryBuilder::<Postgres>::new(format!("SELECT {select_cols} {from_clause} WHERE 1=1"));
        let mut count =
            QueryBuilder::<Postgres>::new(format!("SELECT COUNT(*) {from_clause} WHERE 1=1"));
        push_filters(&mut data, &filter);
        push_filters(&mut count, &filter);

        let order_by = match filter.sort_by.as_deref() {
            // The inventory.products column is `base_price` (not `price`).
            Some("price_asc") if needs_product_join => " ORDER BY p.base_price ASC",
            Some("price_desc") if needs_product_join => " ORDER BY p.base_price DESC",
            Some("popular") => " ORDER BY l.view_count DESC",
            Some("newest") => " ORDER BY l.created_at DESC",
            _ => " ORDER BY l.sort_order ASC, l.created_at DESC",
        };
        data.push(order_by);
        data.push(" LIMIT ");
        data.push_bind(page_size);
        data.push(" OFFSET ");
        data.push_bind(offset);

        let rows: Vec<ListingRow> = data
            .build_query_as::<ListingRow>()
            .fetch_all(&self.pool)
            .await?;
        let total: (i64,) = count
            .build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await?;
        Ok((rows.into_iter().map(Into::into).collect(), total.0))
    }

    async fn find_featured(
        &self,
        store_id: StoreId,
        limit: i64,
    ) -> Result<Vec<ProductListing>, CatalogError> {
        let sql = format!(
            "SELECT {SELECT_COLUMNS} FROM product_listings
             WHERE store_id = $1 AND is_published = true AND is_featured = true
             ORDER BY sort_order ASC, created_at DESC
             LIMIT $2"
        );
        let rows = sqlx::query_as::<_, ListingRow>(&sql)
            .bind(store_id.into_uuid())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn increment_view_count(&self, id: ProductListingId) -> Result<(), CatalogError> {
        sqlx::query("UPDATE product_listings SET view_count = view_count + 1 WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: ProductListingId) -> Result<(), CatalogError> {
        let result = sqlx::query("DELETE FROM product_listings WHERE id = $1")
            .bind(id.into_uuid())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(CatalogError::ListingNotFound(id.into_uuid()));
        }
        Ok(())
    }
}

fn push_filters<'q>(qb: &mut QueryBuilder<'q, Postgres>, f: &'q CatalogFilter) {
    if let Some(store_id) = f.store_id {
        qb.push(" AND l.store_id = ");
        qb.push_bind(store_id.into_uuid());
    }
    if let Some(category_id) = f.category_id {
        // Category filter requires the products join.
        qb.push(" AND EXISTS (SELECT 1 FROM products px WHERE px.id = l.product_id AND px.category_id = ");
        qb.push_bind(category_id);
        qb.push(")");
    }
    if let Some(p) = f.is_published {
        qb.push(" AND l.is_published = ");
        qb.push_bind(p);
    }
    if let Some(f_flag) = f.is_featured {
        qb.push(" AND l.is_featured = ");
        qb.push_bind(f_flag);
    }
    if let Some(min) = f.min_price {
        qb.push(" AND p.base_price >= ");
        qb.push_bind(min);
    }
    if let Some(max) = f.max_price {
        qb.push(" AND p.base_price <= ");
        qb.push_bind(max);
    }
    if let Some(search) = &f.search {
        let pattern = format!("%{}%", search);
        qb.push(" AND (l.title ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR l.short_description ILIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR l.slug ILIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }
}

#[derive(sqlx::FromRow)]
struct ListingRow {
    id: uuid::Uuid,
    store_id: uuid::Uuid,
    product_id: uuid::Uuid,
    slug: String,
    title: String,
    short_description: Option<String>,
    long_description: Option<String>,
    is_published: bool,
    is_featured: bool,
    seo_title: Option<String>,
    seo_description: Option<String>,
    seo_keywords: Vec<String>,
    sort_order: i32,
    view_count: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ListingRow> for ProductListing {
    fn from(row: ListingRow) -> Self {
        ProductListing::reconstitute(
            ProductListingId::from_uuid(row.id),
            StoreId::from_uuid(row.store_id),
            row.product_id,
            row.slug,
            row.title,
            row.short_description,
            row.long_description,
            row.is_published,
            row.is_featured,
            row.seo_title,
            row.seo_description,
            row.seo_keywords,
            row.sort_order,
            row.view_count,
            row.created_at,
            row.updated_at,
        )
    }
}
