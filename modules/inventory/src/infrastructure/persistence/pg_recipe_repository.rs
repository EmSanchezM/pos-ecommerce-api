// PostgreSQL RecipeRepository implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::domain::entities::{IngredientSubstitute, Recipe, RecipeIngredient};
use crate::domain::repositories::RecipeRepository;
use crate::domain::value_objects::{
    IngredientId, ProductId, RecipeId, SubstituteId, UnitOfMeasure, VariantId,
};
use crate::InventoryError;

/// PostgreSQL implementation of RecipeRepository
pub struct PgRecipeRepository {
    pool: PgPool,
}

impl PgRecipeRepository {
    /// Creates a new PgRecipeRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecipeRepository for PgRecipeRepository {
    // =========================================================================
    // Recipe operations
    // =========================================================================

    async fn save(&self, recipe: &Recipe) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO product_recipes (
                id, product_id, variant_id, name, description, version, yield_quantity,
                is_active, preparation_time_minutes, calculate_cost_from_ingredients,
                notes, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(recipe.id().into_uuid())
        .bind(recipe.product_id().map(|id| id.into_uuid()))
        .bind(recipe.variant_id().map(|id| id.into_uuid()))
        .bind(recipe.name())
        .bind(recipe.description())
        .bind(recipe.version())
        .bind(recipe.yield_quantity())
        .bind(recipe.is_active())
        .bind(recipe.preparation_time_minutes())
        .bind(recipe.calculate_cost_from_ingredients())
        .bind(recipe.notes())
        .bind(recipe.metadata())
        .bind(recipe.created_at())
        .bind(recipe.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: RecipeId) -> Result<Option<Recipe>, InventoryError> {
        let row = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, product_id, variant_id, name, description, version, yield_quantity,
                   is_active, preparation_time_minutes, calculate_cost_from_ingredients,
                   notes, metadata, created_at, updated_at
            FROM product_recipes
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }


    async fn find_by_product(&self, product_id: ProductId) -> Result<Vec<Recipe>, InventoryError> {
        let rows = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, product_id, variant_id, name, description, version, yield_quantity,
                   is_active, preparation_time_minutes, calculate_cost_from_ingredients,
                   notes, metadata, created_at, updated_at
            FROM product_recipes
            WHERE product_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(product_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_by_variant(&self, variant_id: VariantId) -> Result<Vec<Recipe>, InventoryError> {
        let rows = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, product_id, variant_id, name, description, version, yield_quantity,
                   is_active, preparation_time_minutes, calculate_cost_from_ingredients,
                   notes, metadata, created_at, updated_at
            FROM product_recipes
            WHERE variant_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(variant_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn find_active_by_product(&self, product_id: ProductId) -> Result<Option<Recipe>, InventoryError> {
        // Only one active recipe per product (enforced by business logic)
        let row = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, product_id, variant_id, name, description, version, yield_quantity,
                   is_active, preparation_time_minutes, calculate_cost_from_ingredients,
                   notes, metadata, created_at, updated_at
            FROM product_recipes
            WHERE product_id = $1 AND is_active = TRUE
            LIMIT 1
            "#,
        )
        .bind(product_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_active_by_variant(&self, variant_id: VariantId) -> Result<Option<Recipe>, InventoryError> {
        // Only one active recipe per variant (enforced by business logic)
        let row = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, product_id, variant_id, name, description, version, yield_quantity,
                   is_active, preparation_time_minutes, calculate_cost_from_ingredients,
                   notes, metadata, created_at, updated_at
            FROM product_recipes
            WHERE variant_id = $1 AND is_active = TRUE
            LIMIT 1
            "#,
        )
        .bind(variant_id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_paginated(
        &self,
        is_active: Option<bool>,
        search: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Recipe>, i64), InventoryError> {
        let offset = (page - 1) * page_size;

        // Build dynamic query based on filters
        let rows = sqlx::query_as::<_, RecipeRow>(
            r#"
            SELECT id, product_id, variant_id, name, description, version, yield_quantity, preparation_time_minutes,
                   calculate_cost_from_ingredients, notes, metadata, is_active, created_at, updated_at
            FROM recipes
            WHERE ($1::bool IS NULL OR is_active = $1)
              AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%' OR description ILIKE '%' || $2 || '%')
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(is_active)
        .bind(search)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let recipes: Result<Vec<Recipe>, _> = rows.into_iter().map(|r| r.try_into()).collect();
        let recipes = recipes?;

        // Get total count
        let total = self.count_filtered(is_active, search).await?;

        Ok((recipes, total))
    }

    async fn count_filtered(
        &self,
        is_active: Option<bool>,
        search: Option<&str>,
    ) -> Result<i64, InventoryError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM recipes
            WHERE ($1::bool IS NULL OR is_active = $1)
              AND ($2::text IS NULL OR name ILIKE '%' || $2 || '%' OR description ILIKE '%' || $2 || '%')
            "#,
        )
        .bind(is_active)
        .bind(search)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    async fn update(&self, recipe: &Recipe) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            UPDATE product_recipes
            SET name = $2, description = $3, version = $4, yield_quantity = $5,
                is_active = $6, preparation_time_minutes = $7, calculate_cost_from_ingredients = $8,
                notes = $9, metadata = $10, updated_at = $11
            WHERE id = $1
            "#,
        )
        .bind(recipe.id().into_uuid())
        .bind(recipe.name())
        .bind(recipe.description())
        .bind(recipe.version())
        .bind(recipe.yield_quantity())
        .bind(recipe.is_active())
        .bind(recipe.preparation_time_minutes())
        .bind(recipe.calculate_cost_from_ingredients())
        .bind(recipe.notes())
        .bind(recipe.metadata())
        .bind(recipe.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::RecipeNotFound(recipe.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete(&self, id: RecipeId) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM product_recipes
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::RecipeNotFound(id.into_uuid()));
        }

        Ok(())
    }


    // =========================================================================
    // Ingredient operations
    // =========================================================================

    async fn save_ingredient(&self, ingredient: &RecipeIngredient) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO recipe_ingredients (
                id, recipe_id, ingredient_product_id, ingredient_variant_id, quantity,
                unit_of_measure, is_optional, can_substitute, sort_order, preparation_step,
                estimated_cost_per_unit, estimated_waste_percentage, notes, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(ingredient.id().into_uuid())
        .bind(ingredient.recipe_id().into_uuid())
        .bind(ingredient.ingredient_product_id().map(|id| id.into_uuid()))
        .bind(ingredient.ingredient_variant_id().map(|id| id.into_uuid()))
        .bind(ingredient.quantity())
        .bind(ingredient.unit_of_measure().to_string())
        .bind(ingredient.is_optional())
        .bind(ingredient.can_substitute())
        .bind(ingredient.sort_order())
        .bind(ingredient.preparation_step())
        .bind(ingredient.estimated_cost_per_unit())
        .bind(ingredient.estimated_waste_percentage())
        .bind(ingredient.notes())
        .bind(ingredient.created_at())
        .bind(ingredient.updated_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_ingredient_by_id(&self, id: IngredientId) -> Result<Option<RecipeIngredient>, InventoryError> {
        let row = sqlx::query_as::<_, IngredientRow>(
            r#"
            SELECT id, recipe_id, ingredient_product_id, ingredient_variant_id, quantity,
                   unit_of_measure, is_optional, can_substitute, sort_order, preparation_step,
                   estimated_cost_per_unit, estimated_waste_percentage, notes, created_at, updated_at
            FROM recipe_ingredients
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| r.try_into()).transpose()
    }

    async fn find_ingredients_by_recipe(&self, recipe_id: RecipeId) -> Result<Vec<RecipeIngredient>, InventoryError> {
        // Order by sort_order for preparation sequence
        let rows = sqlx::query_as::<_, IngredientRow>(
            r#"
            SELECT id, recipe_id, ingredient_product_id, ingredient_variant_id, quantity,
                   unit_of_measure, is_optional, can_substitute, sort_order, preparation_step,
                   estimated_cost_per_unit, estimated_waste_percentage, notes, created_at, updated_at
            FROM recipe_ingredients
            WHERE recipe_id = $1
            ORDER BY sort_order ASC
            "#,
        )
        .bind(recipe_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn update_ingredient(&self, ingredient: &RecipeIngredient) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            UPDATE recipe_ingredients
            SET quantity = $2, unit_of_measure = $3, is_optional = $4, can_substitute = $5,
                sort_order = $6, preparation_step = $7, estimated_cost_per_unit = $8,
                estimated_waste_percentage = $9, notes = $10, updated_at = $11
            WHERE id = $1
            "#,
        )
        .bind(ingredient.id().into_uuid())
        .bind(ingredient.quantity())
        .bind(ingredient.unit_of_measure().to_string())
        .bind(ingredient.is_optional())
        .bind(ingredient.can_substitute())
        .bind(ingredient.sort_order())
        .bind(ingredient.preparation_step())
        .bind(ingredient.estimated_cost_per_unit())
        .bind(ingredient.estimated_waste_percentage())
        .bind(ingredient.notes())
        .bind(ingredient.updated_at())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::IngredientNotFound(ingredient.id().into_uuid()));
        }

        Ok(())
    }

    async fn delete_ingredient(&self, id: IngredientId) -> Result<(), InventoryError> {
        // Note: This will fail if there are substitutes (RESTRICT delete)
        let result = sqlx::query(
            r#"
            DELETE FROM recipe_ingredients
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::IngredientNotFound(id.into_uuid()));
        }

        Ok(())
    }


    // =========================================================================
    // Substitute operations
    // =========================================================================

    async fn save_substitute(&self, substitute: &IngredientSubstitute) -> Result<(), InventoryError> {
        sqlx::query(
            r#"
            INSERT INTO recipe_ingredient_substitutes (
                id, recipe_ingredient_id, substitute_product_id, substitute_variant_id,
                conversion_ratio, priority, notes, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(substitute.id().into_uuid())
        .bind(substitute.recipe_ingredient_id().into_uuid())
        .bind(substitute.substitute_product_id().map(|id| id.into_uuid()))
        .bind(substitute.substitute_variant_id().map(|id| id.into_uuid()))
        .bind(substitute.conversion_ratio())
        .bind(substitute.priority())
        .bind(substitute.notes())
        .bind(substitute.created_at())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_substitutes_by_ingredient(&self, ingredient_id: IngredientId) -> Result<Vec<IngredientSubstitute>, InventoryError> {
        // Order by priority (lower number = higher preference)
        let rows = sqlx::query_as::<_, SubstituteRow>(
            r#"
            SELECT id, recipe_ingredient_id, substitute_product_id, substitute_variant_id,
                   conversion_ratio, priority, notes, created_at
            FROM recipe_ingredient_substitutes
            WHERE recipe_ingredient_id = $1
            ORDER BY priority ASC
            "#,
        )
        .bind(ingredient_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    async fn delete_substitute(&self, id: SubstituteId) -> Result<(), InventoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM recipe_ingredient_substitutes
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(InventoryError::SubstituteNotFound(id.into_uuid()));
        }

        Ok(())
    }
}

// =============================================================================
// Row types for database mapping
// =============================================================================

/// Internal row type for mapping recipe database results
#[derive(sqlx::FromRow)]
struct RecipeRow {
    id: uuid::Uuid,
    product_id: Option<uuid::Uuid>,
    variant_id: Option<uuid::Uuid>,
    name: String,
    description: Option<String>,
    version: i32,
    yield_quantity: Decimal,
    is_active: bool,
    preparation_time_minutes: Option<i32>,
    calculate_cost_from_ingredients: bool,
    notes: Option<String>,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<RecipeRow> for Recipe {
    type Error = InventoryError;

    fn try_from(row: RecipeRow) -> Result<Self, Self::Error> {
        Recipe::reconstitute(
            RecipeId::from_uuid(row.id),
            row.product_id.map(ProductId::from_uuid),
            row.variant_id.map(VariantId::from_uuid),
            row.name,
            row.description,
            row.version,
            row.yield_quantity,
            row.is_active,
            row.preparation_time_minutes,
            row.calculate_cost_from_ingredients,
            row.notes,
            row.metadata,
            row.created_at,
            row.updated_at,
        )
    }
}

/// Internal row type for mapping ingredient database results
#[derive(sqlx::FromRow)]
struct IngredientRow {
    id: uuid::Uuid,
    recipe_id: uuid::Uuid,
    ingredient_product_id: Option<uuid::Uuid>,
    ingredient_variant_id: Option<uuid::Uuid>,
    quantity: Decimal,
    unit_of_measure: String,
    is_optional: bool,
    can_substitute: bool,
    sort_order: i32,
    preparation_step: Option<String>,
    estimated_cost_per_unit: Option<Decimal>,
    estimated_waste_percentage: Decimal,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<IngredientRow> for RecipeIngredient {
    type Error = InventoryError;

    fn try_from(row: IngredientRow) -> Result<Self, Self::Error> {
        let unit_of_measure: UnitOfMeasure = row.unit_of_measure.parse()?;
        
        RecipeIngredient::reconstitute(
            IngredientId::from_uuid(row.id),
            RecipeId::from_uuid(row.recipe_id),
            row.ingredient_product_id.map(ProductId::from_uuid),
            row.ingredient_variant_id.map(VariantId::from_uuid),
            row.quantity,
            unit_of_measure,
            row.is_optional,
            row.can_substitute,
            row.sort_order,
            row.preparation_step,
            row.estimated_cost_per_unit,
            row.estimated_waste_percentage,
            row.notes,
            row.created_at,
            row.updated_at,
        )
    }
}

/// Internal row type for mapping substitute database results
#[derive(sqlx::FromRow)]
struct SubstituteRow {
    id: uuid::Uuid,
    recipe_ingredient_id: uuid::Uuid,
    substitute_product_id: Option<uuid::Uuid>,
    substitute_variant_id: Option<uuid::Uuid>,
    conversion_ratio: Decimal,
    priority: i32,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<SubstituteRow> for IngredientSubstitute {
    type Error = InventoryError;

    fn try_from(row: SubstituteRow) -> Result<Self, Self::Error> {
        IngredientSubstitute::reconstitute(
            SubstituteId::from_uuid(row.id),
            IngredientId::from_uuid(row.recipe_ingredient_id),
            row.substitute_product_id.map(ProductId::from_uuid),
            row.substitute_variant_id.map(VariantId::from_uuid),
            row.conversion_ratio,
            row.priority,
            row.notes,
            row.created_at,
        )
    }
}
