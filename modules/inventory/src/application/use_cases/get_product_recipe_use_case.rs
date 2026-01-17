// GetProductRecipeUseCase - retrieves the active recipe for a product
//
// - Validates product exists
// - Finds active recipe for product
// - Loads all ingredients with their substitutes
// - Returns detailed response

use std::sync::Arc;

use crate::application::dtos::responses::{
    IngredientSubstituteResponse, RecipeDetailResponse, RecipeIngredientResponse,
    ProductResponse,
};
use crate::domain::repositories::{ProductRepository, RecipeRepository};
use crate::domain::value_objects::ProductId;
use crate::InventoryError;

/// Use case for getting the active recipe for a product
///
/// Retrieves the active recipe for a product with all its ingredients and substitutes.
pub struct GetProductRecipeUseCase<R, P>
where
    R: RecipeRepository,
    P: ProductRepository,
{
    recipe_repo: Arc<R>,
    product_repo: Arc<P>,
}

impl<R, P> GetProductRecipeUseCase<R, P>
where
    R: RecipeRepository,
    P: ProductRepository,
{
    /// Creates a new instance of GetProductRecipeUseCase
    pub fn new(recipe_repo: Arc<R>, product_repo: Arc<P>) -> Self {
        Self {
            recipe_repo,
            product_repo,
        }
    }

    /// Executes the use case to get the active recipe for a product
    ///
    /// # Arguments
    /// * `product_id` - The UUID of the product to get the recipe for
    ///
    /// # Returns
    /// RecipeDetailResponse with full recipe details including ingredients
    ///
    /// # Errors
    /// * `InventoryError::ProductNotFound` - If the product doesn't exist
    /// * `InventoryError::RecipeNotFound` - If the product has no active recipe
    pub async fn execute(
        &self,
        product_id: uuid::Uuid,
    ) -> Result<RecipeDetailResponse, InventoryError> {
        let product_id_vo = ProductId::from_uuid(product_id);

        // Validate product exists
        let product = self
            .product_repo
            .find_by_id(product_id_vo)
            .await?
            .ok_or(InventoryError::ProductNotFound(product_id))?;

        // Find the active recipe for this product
        let recipe = self
            .recipe_repo
            .find_active_by_product(product_id_vo)
            .await?
            .ok_or(InventoryError::RecipeNotFound(product_id))?;

        // Build product response
        let product_response = Some(ProductResponse {
            id: product.id().into_uuid(),
            sku: product.sku().to_string(),
            barcode: product.barcode().map(|b| b.to_string()),
            name: product.name().to_string(),
            description: product.description().map(|s| s.to_string()),
            category_id: product.category_id().map(|id| id.into_uuid()),
            brand: product.brand().map(|s| s.to_string()),
            unit_of_measure: product.unit_of_measure().to_string(),
            base_price: product.base_price(),
            cost_price: product.cost_price(),
            currency: product.currency().to_string(),
            is_perishable: product.is_perishable(),
            is_trackable: product.is_trackable(),
            has_variants: product.has_variants(),
            tax_rate: product.tax_rate(),
            tax_included: product.tax_included(),
            is_active: product.is_active(),
            created_at: product.created_at(),
            updated_at: product.updated_at(),
        });

        // Get all ingredients for this recipe
        let ingredients = self
            .recipe_repo
            .find_ingredients_by_recipe(recipe.id())
            .await?;

        // Build ingredient responses with substitutes
        let mut ingredient_responses = Vec::with_capacity(ingredients.len());
        for ingredient in ingredients {
            // Get substitutes for this ingredient
            let substitutes = self
                .recipe_repo
                .find_substitutes_by_ingredient(ingredient.id())
                .await?;

            let substitute_responses: Vec<IngredientSubstituteResponse> = substitutes
                .into_iter()
                .map(|s| IngredientSubstituteResponse {
                    id: s.id().into_uuid(),
                    recipe_ingredient_id: s.recipe_ingredient_id().into_uuid(),
                    substitute_product_id: s.substitute_product_id().map(|id| id.into_uuid()),
                    substitute_variant_id: s.substitute_variant_id().map(|id| id.into_uuid()),
                    substitute_product: None,
                    substitute_variant: None,
                    conversion_ratio: s.conversion_ratio(),
                    priority: s.priority(),
                    notes: s.notes().map(|s| s.to_string()),
                    created_at: s.created_at(),
                })
                .collect();

            ingredient_responses.push(RecipeIngredientResponse {
                id: ingredient.id().into_uuid(),
                recipe_id: ingredient.recipe_id().into_uuid(),
                ingredient_product_id: ingredient.ingredient_product_id().map(|id| id.into_uuid()),
                ingredient_variant_id: ingredient.ingredient_variant_id().map(|id| id.into_uuid()),
                ingredient_product: None,
                ingredient_variant: None,
                quantity: ingredient.quantity(),
                unit_of_measure: ingredient.unit_of_measure().to_string(),
                is_optional: ingredient.is_optional(),
                can_substitute: ingredient.can_substitute(),
                sort_order: ingredient.sort_order(),
                preparation_step: ingredient.preparation_step().map(|s| s.to_string()),
                estimated_cost_per_unit: ingredient.estimated_cost_per_unit(),
                estimated_waste_percentage: ingredient.estimated_waste_percentage(),
                notes: ingredient.notes().map(|s| s.to_string()),
                substitutes: substitute_responses,
                created_at: ingredient.created_at(),
                updated_at: ingredient.updated_at(),
            });
        }

        Ok(RecipeDetailResponse {
            id: recipe.id().into_uuid(),
            product_id: recipe.product_id().map(|id| id.into_uuid()),
            variant_id: recipe.variant_id().map(|id| id.into_uuid()),
            product: product_response,
            variant: None,
            name: recipe.name().to_string(),
            description: recipe.description().map(|s| s.to_string()),
            version: recipe.version(),
            yield_quantity: recipe.yield_quantity(),
            is_active: recipe.is_active(),
            preparation_time_minutes: recipe.preparation_time_minutes(),
            calculate_cost_from_ingredients: recipe.calculate_cost_from_ingredients(),
            calculated_cost: None,
            notes: recipe.notes().map(|s| s.to_string()),
            metadata: Some(recipe.metadata().clone()),
            ingredients: ingredient_responses,
            created_at: recipe.created_at(),
            updated_at: recipe.updated_at(),
        })
    }
}
