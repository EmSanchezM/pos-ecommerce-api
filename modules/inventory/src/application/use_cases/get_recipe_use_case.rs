// GetRecipeUseCase - retrieves a recipe by ID with full details
//
// - Fetches recipe by ID
// - Loads all ingredients with their substitutes
// - Returns detailed response with product/variant info

use std::sync::Arc;

use crate::application::dtos::responses::{
    IngredientSubstituteResponse, RecipeDetailResponse, RecipeIngredientResponse,
    ProductResponse, VariantResponse
};
use crate::domain::repositories::{ProductRepository, RecipeRepository};
use crate::domain::value_objects::RecipeId;
use crate::InventoryError;

/// Use case for getting a recipe by ID with full details
///
/// Retrieves the recipe and all its ingredients with substitutes.
pub struct GetRecipeUseCase<R, P>
where
    R: RecipeRepository,
    P: ProductRepository,
{
    recipe_repo: Arc<R>,
    product_repo: Arc<P>,
}

impl<R, P> GetRecipeUseCase<R, P>
where
    R: RecipeRepository,
    P: ProductRepository,
{
    /// Creates a new instance of GetRecipeUseCase
    pub fn new(recipe_repo: Arc<R>, product_repo: Arc<P>) -> Self {
        Self {
            recipe_repo,
            product_repo,
        }
    }

    /// Executes the use case to get a recipe by ID
    ///
    /// # Arguments
    /// * `recipe_id` - The UUID of the recipe to retrieve
    ///
    /// # Returns
    /// RecipeDetailResponse with full recipe details including ingredients
    ///
    /// # Errors
    /// * `InventoryError::RecipeNotFound` - If the recipe doesn't exist
    pub async fn execute(
        &self,
        recipe_id: uuid::Uuid,
    ) -> Result<RecipeDetailResponse, InventoryError> {
        let recipe_id_vo = RecipeId::from_uuid(recipe_id);

        // Find the recipe
        let recipe = self
            .recipe_repo
            .find_by_id(recipe_id_vo)
            .await?
            .ok_or(InventoryError::RecipeNotFound(recipe_id))?;

        // Get product/variant info if available
        let product_response = if let Some(product_id) = recipe.product_id() {
            self.product_repo
                .find_by_id(product_id)
                .await?
                .map(|p| ProductResponse {
                    id: p.id().into_uuid(),
                    sku: p.sku().to_string(),
                    barcode: p.barcode().map(|b| b.to_string()),
                    name: p.name().to_string(),
                    description: p.description().map(|s| s.to_string()),
                    category_id: p.category_id().map(|id| id.into_uuid()),
                    brand: p.brand().map(|s| s.to_string()),
                    unit_of_measure: p.unit_of_measure().to_string(),
                    base_price: p.base_price(),
                    cost_price: p.cost_price(),
                    currency: p.currency().to_string(),
                    is_perishable: p.is_perishable(),
                    is_trackable: p.is_trackable(),
                    has_variants: p.has_variants(),
                    tax_rate: p.tax_rate(),
                    tax_included: p.tax_included(),
                    is_active: p.is_active(),
                    created_at: p.created_at(),
                    updated_at: p.updated_at(),
                })
        } else {
            None
        };

        let variant_response = if let Some(variant_id) = recipe.variant_id() {
            self.product_repo
                .find_variant_by_id(variant_id)
                .await?
                .map(|v| VariantResponse {
                    id: v.id().into_uuid(),
                    product_id: v.product_id().into_uuid(),
                    sku: v.sku().to_string(),
                    barcode: v.barcode().map(|b| b.to_string()),
                    name: v.name().to_string(),
                    variant_attributes: v.variant_attributes().clone(),
                    price: v.price(),
                    cost_price: v.cost_price(),
                    effective_price: v.price().unwrap_or_default(),
                    effective_cost: v.cost_price().unwrap_or_default(),
                    is_active: v.is_active(),
                    created_at: v.created_at(),
                    updated_at: v.updated_at(),
                })
        } else {
            None
        };

        // Get all ingredients for this recipe
        let ingredients = self
            .recipe_repo
            .find_ingredients_by_recipe(recipe_id_vo)
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
                    substitute_product: None, // Could be loaded if needed
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
                ingredient_product: None, // Could be loaded if needed
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
            variant: variant_response,
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
