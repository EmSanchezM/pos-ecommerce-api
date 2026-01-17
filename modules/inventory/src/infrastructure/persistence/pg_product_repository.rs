// PostgreSQL ProductRepository implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::{Product, ProductVariant};
use crate::domain::repositories::ProductRepository;
use crate::domain::value_objects::{Barcode, CategoryId, Currency, ProductId, Sku, UnitOfMeasure, VariantId};
use crate::InventoryError;

/// PostgreSQL implementation of ProductRepository
pub struct PgProductRepository {
    pool: PgPool,
}

impl PgProductRepository {
    /// Creates a new PgProductRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
    async fn save(&self, product: &Product) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO products (
                id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                tax_rate, tax_included, attributes, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
            "#,
        )
        .bind(product.id().into_uuid())
        .bind(product.sku().as_str())
        .bind(product.barcode().map(|b| b.as_str()))
        .bind(product.name())
        .bind(product.description())
        .bind(product.category_id().map(|id| id.into_uuid()))
        .bind(product.brand())
        .bind(product.unit_of_measure().to_string())
        .bind(product.base_price())
        .bind(product.cost_price())
        .bind(product.currency().as_str())
        .bind(product.is_perishable())
        .bind(product.is_trackable())
        .bind(product.has_variants())
        .bind(product.tax_rate())
        .bind(product.tax_included())
        .bind(product.attributes())
        .bind(product.is_active())
        .bind(product.created_at())
        .bind(product.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: ProductId) -> Result<Option<Product>, InventoryError> {
        let row = sqlx::query_as::<_, ProductRow>(
            r#"
            SELECT id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                   base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                   tax_rate, tax_included, attributes, is_active, created_at, updated_at
            FROM products
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }


    async fn find_by_sku(&self, sku: &Sku) -> Result<Option<Product>, InventoryError> {
        let row = sqlx::query_as::<_, ProductRow>(
            r#"
            SELECT id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                   base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                   tax_rate, tax_included, attributes, is_active, created_at, updated_at
            FROM products
            WHERE sku = $1
            "#,
        )
        .bind(sku.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_by_barcode(&self, barcode: &Barcode) -> Result<Option<Product>, InventoryError> {
        let row = sqlx::query_as::<_, ProductRow>(
            r#"
            SELECT id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                   base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                   tax_rate, tax_included, attributes, is_active, created_at, updated_at
            FROM products
            WHERE barcode = $1
            "#,
        )
        .bind(barcode.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn update(&self, product: &Product) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            UPDATE products
            SET sku = $2, barcode = $3, name = $4, description = $5, category_id = $6, brand = $7,
                unit_of_measure = $8, base_price = $9, cost_price = $10, currency = $11,
                is_perishable = $12, is_trackable = $13, has_variants = $14, tax_rate = $15,
                tax_included = $16, attributes = $17, is_active = $18, updated_at = $19
            WHERE id = $1
            "#,
        )
        .bind(product.id().into_uuid())
        .bind(product.sku().as_str())
        .bind(product.barcode().map(|b| b.as_str()))
        .bind(product.name())
        .bind(product.description())
        .bind(product.category_id().map(|id| id.into_uuid()))
        .bind(product.brand())
        .bind(product.unit_of_measure().to_string())
        .bind(product.base_price())
        .bind(product.cost_price())
        .bind(product.currency().as_str())
        .bind(product.is_perishable())
        .bind(product.is_trackable())
        .bind(product.has_variants())
        .bind(product.tax_rate())
        .bind(product.tax_included())
        .bind(product.attributes())
        .bind(product.is_active())
        .bind(product.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::ProductNotFound(product.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete(&self, id: ProductId) -> Result<(), InventoryError> {
        // Cascade delete is handled by the database (ON DELETE CASCADE on variants)
        let result = sqlx::query(
            r#"
            DELETE FROM products
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::ProductNotFound(id.into_uuid()));
        }

        Ok(())
    }

    async fn find_active(&self) -> Result<Vec<Product>, InventoryError> {
        let rows = sqlx::query_as::<_, ProductRow>(
            r#"
            SELECT id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                   base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                   tax_rate, tax_included, attributes, is_active, created_at, updated_at
            FROM products
            WHERE is_active = TRUE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_by_category(&self, category_id: CategoryId) -> Result<Vec<Product>, InventoryError> {
        let rows = sqlx::query_as::<_, ProductRow>(
            r#"
            SELECT id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                   base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                   tax_rate, tax_included, attributes, is_active, created_at, updated_at
            FROM products
            WHERE category_id = $1
            ORDER BY name
            "#,
        )
        .bind(category_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_paginated(
        &self,
        category_id: Option<CategoryId>,
        is_active: Option<bool>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Product>, i64), InventoryError> {
        let offset = (page - 1) * page_size;

        // Build dynamic query based on filters
        let rows = sqlx::query_as::<_, ProductRow>(
            r#"
            SELECT id, sku, barcode, name, description, category_id, brand, unit_of_measure,
                   base_price, cost_price, currency, is_perishable, is_trackable, has_variants,
                   tax_rate, tax_included, attributes, is_active, created_at, updated_at
            FROM products
            WHERE ($1::uuid IS NULL OR category_id = $1)
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::text IS NULL OR name ILIKE '%' || $3 || '%' OR description ILIKE '%' || $3 || '%')
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(category_id.map(|c| c.into_uuid()))
        .bind(is_active)
        .bind(search)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let products: Result<Vec<Product>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        let products = products?;

        // Get total count
        let total = self.count_filtered(category_id, is_active, search).await?;

        Ok((products, total))
    }

    async fn count_filtered(
        &self,
        category_id: Option<CategoryId>,
        is_active: Option<bool>,
        search: Option<&str>,
    ) -> Result<i64, InventoryError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM products
            WHERE ($1::uuid IS NULL OR category_id = $1)
              AND ($2::bool IS NULL OR is_active = $2)
              AND ($3::text IS NULL OR name ILIKE '%' || $3 || '%' OR description ILIKE '%' || $3 || '%')
            "#,
        )
        .bind(category_id.map(|c| c.into_uuid()))
        .bind(is_active)
        .bind(search)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    // =========================================================================
    // Variant operations
    // =========================================================================

    async fn save_variant(&self, variant: &ProductVariant) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO product_variants (
                id, product_id, sku, barcode, name, variant_attributes, price, cost_price, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(variant.id().into_uuid())
        .bind(variant.product_id().into_uuid())
        .bind(variant.sku().as_str())
        .bind(variant.barcode().map(|b| b.as_str()))
        .bind(variant.name())
        .bind(variant.variant_attributes())
        .bind(variant.price())
        .bind(variant.cost_price())
        .bind(variant.is_active())
        .bind(variant.created_at())
        .bind(variant.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_variant_by_id(&self, id: VariantId) -> Result<Option<ProductVariant>, InventoryError> {
        let row = sqlx::query_as::<_, VariantRow>(
            r#"
            SELECT id, product_id, sku, barcode, name, variant_attributes, price, cost_price, is_active, created_at, updated_at
            FROM product_variants
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_variant_by_sku(&self, sku: &Sku) -> Result<Option<ProductVariant>, InventoryError> {
        let row = sqlx::query_as::<_, VariantRow>(
            r#"
            SELECT id, product_id, sku, barcode, name, variant_attributes, price, cost_price, is_active, created_at, updated_at
            FROM product_variants
            WHERE sku = $1
            "#,
        )
        .bind(sku.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_variant_by_barcode(&self, barcode: &Barcode) -> Result<Option<ProductVariant>, InventoryError> {
        let row = sqlx::query_as::<_, VariantRow>(
            r#"
            SELECT id, product_id, sku, barcode, name, variant_attributes, price, cost_price, is_active, created_at, updated_at
            FROM product_variants
            WHERE barcode = $1
            "#,
        )
        .bind(barcode.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_variants_by_product(&self, product_id: ProductId) -> Result<Vec<ProductVariant>, InventoryError> {
        let rows = sqlx::query_as::<_, VariantRow>(
            r#"
            SELECT id, product_id, sku, barcode, name, variant_attributes, price, cost_price, is_active, created_at, updated_at
            FROM product_variants
            WHERE product_id = $1
            ORDER BY name
            "#,
        )
        .bind(product_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update_variant(&self, variant: &ProductVariant) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            UPDATE product_variants
            SET sku = $2, barcode = $3, name = $4, variant_attributes = $5, price = $6, cost_price = $7, is_active = $8, updated_at = $9
            WHERE id = $1
            "#,
        )
        .bind(variant.id().into_uuid())
        .bind(variant.sku().as_str())
        .bind(variant.barcode().map(|b| b.as_str()))
        .bind(variant.name())
        .bind(variant.variant_attributes())
        .bind(variant.price())
        .bind(variant.cost_price())
        .bind(variant.is_active())
        .bind(variant.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::VariantNotFound(variant.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete_variant(&self, id: VariantId) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM product_variants
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::VariantNotFound(id.into_uuid()));
        }

        Ok(())
    }

    async fn count_variants(&self, product_id: ProductId) -> Result<u32, InventoryError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM product_variants WHERE product_id = $1
            "#,
        )
        .bind(product_id.into_uuid())
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 as u32)
    }
}


/// Internal row type for mapping product database results
#[derive(sqlx::FromRow)]
struct ProductRow {
    id: uuid::Uuid,
    sku: String,
    barcode: Option<String>,
    name: String,
    description: Option<String>,
    category_id: Option<uuid::Uuid>,
    brand: Option<String>,
    unit_of_measure: String,
    base_price: Decimal,
    cost_price: Decimal,
    currency: String,
    is_perishable: bool,
    is_trackable: bool,
    has_variants: bool,
    tax_rate: Decimal,
    tax_included: bool,
    attributes: serde_json::Value,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ProductRow> for Product {
    type Error = InventoryError;

    fn try_from(row: ProductRow) -> Result<Self, Self::Error> {
        let unit_of_measure: UnitOfMeasure = row.unit_of_measure.parse()?;
        
        Ok(Product::reconstitute(
            ProductId::from_uuid(row.id),
            Sku::from_string(row.sku),
            row.barcode.map(Barcode::from_string),
            row.name,
            row.description,
            row.category_id.map(CategoryId::from_uuid),
            row.brand,
            unit_of_measure,
            row.base_price,
            row.cost_price,
            Currency::from_string(row.currency),
            row.is_perishable,
            row.is_trackable,
            row.has_variants,
            row.tax_rate,
            row.tax_included,
            row.attributes,
            row.is_active,
            row.created_at,
            row.updated_at,
        ))
    }
}

/// Internal row type for mapping variant database results
#[derive(sqlx::FromRow)]
struct VariantRow {
    id: uuid::Uuid,
    product_id: uuid::Uuid,
    sku: String,
    barcode: Option<String>,
    name: String,
    variant_attributes: serde_json::Value,
    price: Option<Decimal>,
    cost_price: Option<Decimal>,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<VariantRow> for ProductVariant {
    fn from(row: VariantRow) -> Self {
        ProductVariant::reconstitute(
            VariantId::from_uuid(row.id),
            ProductId::from_uuid(row.product_id),
            Sku::from_string(row.sku),
            row.barcode.map(Barcode::from_string),
            row.name,
            row.variant_attributes,
            row.price,
            row.cost_price,
            row.is_active,
            row.created_at,
            row.updated_at,
        )
    }
}
