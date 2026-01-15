// RecipeRepository trait - repository for recipe/BOM operations

use async_trait::async_trait;

use crate::domain::entities::{IngredientSubstitute, Recipe, RecipeIngredient};
use crate::domain::value_objects::{IngredientId, ProductId, RecipeId, VariantId};
use crate::InventoryError;

/// Repository trait for Recipe (BOM) persistence operations.
/// Handles recipes, ingredients, and substitutes.
#[async_trait]
pub trait RecipeRepository: Send + Sync {
    // =========================================================================
    // Recipe operations
    // =========================================================================

    /// Saves a new recipe to the repository
    async fn save(&self, recipe: &Recipe) -> Result<(), InventoryError>;

    /// Finds a recipe by its unique ID
    async fn find_by_id(&self, id: RecipeId) -> Result<Option<Recipe>, InventoryError>;

    /// Finds all recipes for a product (including inactive)
    async fn find_by_product(&self, product_id: ProductId) -> Result<Vec<Recipe>, InventoryError>;

    /// Finds all recipes for a variant (including inactive)
    async fn find_by_variant(&self, variant_id: VariantId) -> Result<Vec<Recipe>, InventoryError>;

    /// Finds the active recipe for a product (only one active recipe per product)
    async fn find_active_by_product(&self, product_id: ProductId) -> Result<Option<Recipe>, InventoryError>;

    /// Finds the active recipe for a variant (only one active recipe per variant)
    async fn find_active_by_variant(&self, variant_id: VariantId) -> Result<Option<Recipe>, InventoryError>;

    /// Updates an existing recipe
    async fn update(&self, recipe: &Recipe) -> Result<(), InventoryError>;

    /// Deletes a recipe by ID
    async fn delete(&self, id: RecipeId) -> Result<(), InventoryError>;

    // =========================================================================
    // Ingredient operations
    // =========================================================================

    /// Saves a new recipe ingredient
    async fn save_ingredient(&self, ingredient: &RecipeIngredient) -> Result<(), InventoryError>;

    /// Finds an ingredient by its unique ID
    async fn find_ingredient_by_id(&self, id: IngredientId) -> Result<Option<RecipeIngredient>, InventoryError>;

    /// Finds all ingredients for a recipe, ordered by sort_order
    async fn find_ingredients_by_recipe(&self, recipe_id: RecipeId) -> Result<Vec<RecipeIngredient>, InventoryError>;

    /// Updates an existing ingredient
    async fn update_ingredient(&self, ingredient: &RecipeIngredient) -> Result<(), InventoryError>;

    /// Deletes an ingredient by ID
    /// Returns error if ingredient has substitutes (must delete substitutes first)
    async fn delete_ingredient(&self, id: IngredientId) -> Result<(), InventoryError>;

    // =========================================================================
    // Substitute operations
    // =========================================================================

    /// Saves a new ingredient substitute
    async fn save_substitute(&self, substitute: &IngredientSubstitute) -> Result<(), InventoryError>;

    /// Finds all substitutes for an ingredient, ordered by priority
    async fn find_substitutes_by_ingredient(&self, ingredient_id: IngredientId) -> Result<Vec<IngredientSubstitute>, InventoryError>;

    /// Deletes a substitute by ID
    async fn delete_substitute(&self, id: crate::domain::value_objects::SubstituteId) -> Result<(), InventoryError>;
}
