// CalculateRecipeCostUseCase - calculates the cost of a recipe from its ingredients
//
// - Sum ingredient costs with waste percentage
// - Divide by yield_quantity

use std::sync::Arc;

use rust_decimal::Decimal;

use crate::domain::repositories::RecipeRepository;
use crate::domain::value_objects::RecipeId;
use crate::InventoryError;

/// Response containing the calculated recipe cost
#[derive(Debug, Clone)]
pub struct RecipeCostResult {
    /// The recipe ID
    pub recipe_id: uuid::Uuid,
    /// Total cost of all ingredients (including waste)
    pub total_ingredient_cost: Decimal,
    /// Cost per unit produced (total_ingredient_cost / yield_quantity)
    pub cost_per_unit: Decimal,
    /// The yield quantity used in calculation
    pub yield_quantity: Decimal,
    /// Number of ingredients included in calculation
    pub ingredient_count: usize,
    /// Number of ingredients with cost data
    pub ingredients_with_cost: usize,
}

/// Use case for calculating recipe cost from ingredients
///
/// Calculates the total cost of a recipe by summing all ingredient costs
/// (including waste percentage) and dividing by the yield quantity.
///
/// Formula per ingredient: quantity * (1 + waste_percentage) * cost_per_unit
/// Total cost per unit: sum(ingredient_costs) / yield_quantity
pub struct CalculateRecipeCostUseCase<R>
where
    R: RecipeRepository,
{
    recipe_repo: Arc<R>,
}

impl<R> CalculateRecipeCostUseCase<R>
where
    R: RecipeRepository,
{
    /// Creates a new instance of CalculateRecipeCostUseCase
    pub fn new(recipe_repo: Arc<R>) -> Self {
        Self { recipe_repo }
    }

    /// Executes the use case to calculate recipe cost
    ///
    /// # Arguments
    /// * `recipe_id` - The UUID of the recipe to calculate cost for
    ///
    /// # Returns
    /// RecipeCostResult containing the calculated costs
    ///
    /// # Errors
    /// * `InventoryError::RecipeNotFound` - If the recipe doesn't exist
    pub async fn execute(&self, recipe_id: uuid::Uuid) -> Result<RecipeCostResult, InventoryError> {
        let recipe_id_vo = RecipeId::from_uuid(recipe_id);

        // Find the recipe
        let recipe = self
            .recipe_repo
            .find_by_id(recipe_id_vo)
            .await?
            .ok_or(InventoryError::RecipeNotFound(recipe_id))?;

        // Get all ingredients for this recipe
        let ingredients = self
            .recipe_repo
            .find_ingredients_by_recipe(recipe_id_vo)
            .await?;

        let ingredient_count = ingredients.len();
        let mut ingredients_with_cost = 0;
        let mut total_ingredient_cost = Decimal::ZERO;

        // Calculate total ingredient cost (Requirement 7.4)
        // Formula: sum(quantity * (1 + waste_percentage) * cost_per_unit)
        for ingredient in &ingredients {
            if let Some(effective_cost) = ingredient.calculate_effective_cost() {
                total_ingredient_cost += effective_cost;
                ingredients_with_cost += 1;
            }
        }

        // Calculate cost per unit (Requirement 6.3)
        // Formula: total_ingredient_cost / yield_quantity
        let cost_per_unit = recipe.calculate_cost(total_ingredient_cost);

        Ok(RecipeCostResult {
            recipe_id,
            total_ingredient_cost,
            cost_per_unit,
            yield_quantity: recipe.yield_quantity(),
            ingredient_count,
            ingredients_with_cost,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::domain::entities::{IngredientSubstitute, Recipe, RecipeIngredient};
    use crate::domain::value_objects::{IngredientId, ProductId, SubstituteId, UnitOfMeasure, VariantId};

    // Mock repository for testing
    struct MockRecipeRepository {
        recipes: Mutex<HashMap<RecipeId, Recipe>>,
        ingredients: Mutex<HashMap<RecipeId, Vec<RecipeIngredient>>>,
    }

    impl MockRecipeRepository {
        fn new() -> Self {
            Self {
                recipes: Mutex::new(HashMap::new()),
                ingredients: Mutex::new(HashMap::new()),
            }
        }

        fn add_recipe(&self, recipe: Recipe) {
            let mut recipes = self.recipes.lock().unwrap();
            recipes.insert(recipe.id(), recipe);
        }

        fn add_ingredient(&self, recipe_id: RecipeId, ingredient: RecipeIngredient) {
            let mut ingredients = self.ingredients.lock().unwrap();
            ingredients
                .entry(recipe_id)
                .or_insert_with(Vec::new)
                .push(ingredient);
        }
    }

    #[async_trait]
    impl RecipeRepository for MockRecipeRepository {
        async fn save(&self, recipe: &Recipe) -> Result<(), InventoryError> {
            let mut recipes = self.recipes.lock().unwrap();
            recipes.insert(recipe.id(), recipe.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: RecipeId) -> Result<Option<Recipe>, InventoryError> {
            let recipes = self.recipes.lock().unwrap();
            Ok(recipes.get(&id).cloned())
        }

        async fn find_by_product(&self, _product_id: ProductId) -> Result<Vec<Recipe>, InventoryError> {
            unimplemented!()
        }

        async fn find_by_variant(&self, _variant_id: VariantId) -> Result<Vec<Recipe>, InventoryError> {
            unimplemented!()
        }

        async fn find_active_by_product(&self, _product_id: ProductId) -> Result<Option<Recipe>, InventoryError> {
            unimplemented!()
        }

        async fn find_active_by_variant(&self, _variant_id: VariantId) -> Result<Option<Recipe>, InventoryError> {
            unimplemented!()
        }

        async fn find_paginated(
            &self,
            _is_active: Option<bool>,
            _search: Option<&str>,
            _page: i64,
            _page_size: i64,
        ) -> Result<(Vec<Recipe>, i64), InventoryError> {
            unimplemented!()
        }

        async fn count_filtered(
            &self,
            _is_active: Option<bool>,
            _search: Option<&str>,
        ) -> Result<i64, InventoryError> {
            unimplemented!()
        }

        async fn update(&self, _recipe: &Recipe) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: RecipeId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn save_ingredient(&self, ingredient: &RecipeIngredient) -> Result<(), InventoryError> {
            let mut ingredients = self.ingredients.lock().unwrap();
            ingredients
                .entry(ingredient.recipe_id())
                .or_insert_with(Vec::new)
                .push(ingredient.clone());
            Ok(())
        }

        async fn find_ingredient_by_id(&self, _id: IngredientId) -> Result<Option<RecipeIngredient>, InventoryError> {
            unimplemented!()
        }

        async fn find_ingredients_by_recipe(&self, recipe_id: RecipeId) -> Result<Vec<RecipeIngredient>, InventoryError> {
            let ingredients = self.ingredients.lock().unwrap();
            Ok(ingredients.get(&recipe_id).cloned().unwrap_or_default())
        }

        async fn update_ingredient(&self, _ingredient: &RecipeIngredient) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete_ingredient(&self, _id: IngredientId) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn save_substitute(&self, _substitute: &IngredientSubstitute) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn find_substitutes_by_ingredient(&self, _ingredient_id: IngredientId) -> Result<Vec<IngredientSubstitute>, InventoryError> {
            unimplemented!()
        }

        async fn delete_substitute(&self, _id: SubstituteId) -> Result<(), InventoryError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_calculate_recipe_cost_basic() {
        let repo = Arc::new(MockRecipeRepository::new());

        // Create a recipe with yield of 10 units
        let product_id = ProductId::new();
        let recipe = Recipe::create_for_product(product_id, "Test Recipe".to_string(), dec!(10)).unwrap();
        let recipe_id = recipe.id();
        repo.add_recipe(recipe);

        // Add ingredient 1: 2 kg at $10/kg = $20
        let mut ing1 = RecipeIngredient::create_for_product(
            recipe_id,
            ProductId::new(),
            dec!(2),
            UnitOfMeasure::Kg,
        ).unwrap();
        ing1.set_estimated_cost_per_unit(Some(dec!(10)));
        repo.add_ingredient(recipe_id, ing1);

        // Add ingredient 2: 1 liter at $5/liter = $5
        let mut ing2 = RecipeIngredient::create_for_product(
            recipe_id,
            ProductId::new(),
            dec!(1),
            UnitOfMeasure::Liter,
        ).unwrap();
        ing2.set_estimated_cost_per_unit(Some(dec!(5)));
        repo.add_ingredient(recipe_id, ing2);

        let use_case = CalculateRecipeCostUseCase::new(repo);
        let result = use_case.execute(recipe_id.into_uuid()).await.unwrap();

        // Total: $20 + $5 = $25
        // Cost per unit: $25 / 10 = $2.50
        assert_eq!(result.total_ingredient_cost, dec!(25));
        assert_eq!(result.cost_per_unit, dec!(2.5));
        assert_eq!(result.yield_quantity, dec!(10));
        assert_eq!(result.ingredient_count, 2);
        assert_eq!(result.ingredients_with_cost, 2);
    }

    #[tokio::test]
    async fn test_calculate_recipe_cost_with_waste() {
        let repo = Arc::new(MockRecipeRepository::new());

        // Create a recipe with yield of 10 units
        let product_id = ProductId::new();
        let recipe = Recipe::create_for_product(product_id, "Test Recipe".to_string(), dec!(10)).unwrap();
        let recipe_id = recipe.id();
        repo.add_recipe(recipe);

        // Add ingredient with 10% waste: 2 kg at $10/kg
        // Effective cost: 2 * (1 + 0.1) * 10 = 2 * 1.1 * 10 = $22
        let mut ing = RecipeIngredient::create_for_product(
            recipe_id,
            ProductId::new(),
            dec!(2),
            UnitOfMeasure::Kg,
        ).unwrap();
        ing.set_estimated_cost_per_unit(Some(dec!(10)));
        ing.set_estimated_waste_percentage(dec!(0.1)).unwrap();
        repo.add_ingredient(recipe_id, ing);

        let use_case = CalculateRecipeCostUseCase::new(repo);
        let result = use_case.execute(recipe_id.into_uuid()).await.unwrap();

        // Total: $22
        // Cost per unit: $22 / 10 = $2.20
        assert_eq!(result.total_ingredient_cost, dec!(22));
        assert_eq!(result.cost_per_unit, dec!(2.2));
    }

    #[tokio::test]
    async fn test_calculate_recipe_cost_missing_cost_data() {
        let repo = Arc::new(MockRecipeRepository::new());

        // Create a recipe
        let product_id = ProductId::new();
        let recipe = Recipe::create_for_product(product_id, "Test Recipe".to_string(), dec!(10)).unwrap();
        let recipe_id = recipe.id();
        repo.add_recipe(recipe);

        // Add ingredient with cost
        let mut ing1 = RecipeIngredient::create_for_product(
            recipe_id,
            ProductId::new(),
            dec!(2),
            UnitOfMeasure::Kg,
        ).unwrap();
        ing1.set_estimated_cost_per_unit(Some(dec!(10)));
        repo.add_ingredient(recipe_id, ing1);

        // Add ingredient without cost (no estimated_cost_per_unit set)
        let ing2 = RecipeIngredient::create_for_product(
            recipe_id,
            ProductId::new(),
            dec!(1),
            UnitOfMeasure::Liter,
        ).unwrap();
        repo.add_ingredient(recipe_id, ing2);

        let use_case = CalculateRecipeCostUseCase::new(repo);
        let result = use_case.execute(recipe_id.into_uuid()).await.unwrap();

        // Only ingredient 1 has cost: $20
        // Cost per unit: $20 / 10 = $2.00
        assert_eq!(result.total_ingredient_cost, dec!(20));
        assert_eq!(result.cost_per_unit, dec!(2));
        assert_eq!(result.ingredient_count, 2);
        assert_eq!(result.ingredients_with_cost, 1);
    }

    #[tokio::test]
    async fn test_calculate_recipe_cost_no_ingredients() {
        let repo = Arc::new(MockRecipeRepository::new());

        // Create a recipe with no ingredients
        let product_id = ProductId::new();
        let recipe = Recipe::create_for_product(product_id, "Empty Recipe".to_string(), dec!(10)).unwrap();
        let recipe_id = recipe.id();
        repo.add_recipe(recipe);

        let use_case = CalculateRecipeCostUseCase::new(repo);
        let result = use_case.execute(recipe_id.into_uuid()).await.unwrap();

        assert_eq!(result.total_ingredient_cost, dec!(0));
        assert_eq!(result.cost_per_unit, dec!(0));
        assert_eq!(result.ingredient_count, 0);
        assert_eq!(result.ingredients_with_cost, 0);
    }

    #[tokio::test]
    async fn test_calculate_recipe_cost_not_found() {
        let repo = Arc::new(MockRecipeRepository::new());
        let use_case = CalculateRecipeCostUseCase::new(repo);

        let non_existent_id = RecipeId::new().into_uuid();
        let result = use_case.execute(non_existent_id).await;

        assert!(matches!(result, Err(InventoryError::RecipeNotFound(_))));
    }
}
