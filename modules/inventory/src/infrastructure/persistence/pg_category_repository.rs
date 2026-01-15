// PostgreSQL CategoryRepository implementation

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entities::ProductCategory;
use crate::domain::repositories::CategoryRepository;
use crate::domain::value_objects::CategoryId;
use crate::InventoryError;

/// PostgreSQL implementation of CategoryRepository
pub struct PgCategoryRepository {
    pool: PgPool,
}

impl PgCategoryRepository {
    /// Creates a new PgCategoryRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepository {
    async fn save(&self, category: &ProductCategory) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO product_categories (id, parent_id, name, description, slug, icon, sort_order, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(category.id().into_uuid())
        .bind(category.parent_id().map(|id| id.into_uuid()))
        .bind(category.name())
        .bind(category.description())
        .bind(category.slug())
        .bind(category.icon())
        .bind(category.sort_order())
        .bind(category.is_active())
        .bind(category.created_at())
        .bind(category.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: CategoryId) -> Result<Option<ProductCategory>, InventoryError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, parent_id, name, description, slug, icon, sort_order, is_active, created_at, updated_at
            FROM product_categories
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }


    async fn find_by_slug(&self, slug: &str) -> Result<Option<ProductCategory>, InventoryError> {
        let row = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, parent_id, name, description, slug, icon, sort_order, is_active, created_at, updated_at
            FROM product_categories
            WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_root_categories(&self) -> Result<Vec<ProductCategory>, InventoryError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, parent_id, name, description, slug, icon, sort_order, is_active, created_at, updated_at
            FROM product_categories
            WHERE parent_id IS NULL
            ORDER BY sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_children(&self, parent_id: CategoryId) -> Result<Vec<ProductCategory>, InventoryError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, parent_id, name, description, slug, icon, sort_order, is_active, created_at, updated_at
            FROM product_categories
            WHERE parent_id = $1
            ORDER BY sort_order, name
            "#,
        )
        .bind(parent_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_all_active(&self) -> Result<Vec<ProductCategory>, InventoryError> {
        let rows = sqlx::query_as::<_, CategoryRow>(
            r#"
            SELECT id, parent_id, name, description, slug, icon, sort_order, is_active, created_at, updated_at
            FROM product_categories
            WHERE is_active = TRUE
            ORDER BY COALESCE(parent_id, id), sort_order, name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update(&self, category: &ProductCategory) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            UPDATE product_categories
            SET parent_id = $2, name = $3, description = $4, slug = $5, icon = $6, sort_order = $7, is_active = $8, updated_at = $9
            WHERE id = $1
            "#,
        )
        .bind(category.id().into_uuid())
        .bind(category.parent_id().map(|id| id.into_uuid()))
        .bind(category.name())
        .bind(category.description())
        .bind(category.slug())
        .bind(category.icon())
        .bind(category.sort_order())
        .bind(category.is_active())
        .bind(category.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::CategoryNotFound(category.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete(&self, id: CategoryId) -> Result<(), InventoryError> {
        // First, orphan any child categories by setting their parent_id to NULL
        sqlx::query(
            r#"
            UPDATE product_categories
            SET parent_id = NULL, updated_at = NOW()
            WHERE parent_id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        // Then delete the category
        let result = sqlx::query(
            r#"
            DELETE FROM product_categories
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::CategoryNotFound(id.into_uuid()));
        }

        Ok(())
    }
}

/// Internal row type for mapping category database results
#[derive(sqlx::FromRow)]
struct CategoryRow {
    id: uuid::Uuid,
    parent_id: Option<uuid::Uuid>,
    name: String,
    description: Option<String>,
    slug: String,
    icon: Option<String>,
    sort_order: i32,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CategoryRow> for ProductCategory {
    fn from(row: CategoryRow) -> Self {
        ProductCategory::reconstitute(
            CategoryId::from_uuid(row.id),
            row.parent_id.map(CategoryId::from_uuid),
            row.name,
            row.description,
            row.slug,
            row.icon,
            row.sort_order,
            row.is_active,
            row.created_at,
            row.updated_at,
        )
    }
}
