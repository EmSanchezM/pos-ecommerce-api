// UpdateRecipeUseCase - updates an existing recipe
//
// - Validates recipe exists
// - Updates fields as requested
// - Returns updated recipe response

use std::sync::Arc;

use crate::application::dtos::commands::UpdateRecipeCommand;
use crate::application::dtos::responses::RecipeResponse;
use crate::domain::repositories::RecipeRepository;
use crate::domain::value_objects::RecipeId;
use crate::InventoryError;

/// Use case for updating an existing recipe
///
/// Updates the recipe fields as specified in the command.
/// Only non-None fields in the command will be updated.
pub struct UpdateRecipeUseCase<R>
where
    R: RecipeRepository,
{
    recipe_repo: Arc<R>,
}

impl<R> UpdateRecipeUseCase<R>
where
    R: RecipeRepository,
{
    /// Creates a new instance of UpdateRecipeUseCase
    pub fn new(recipe_repo: Arc<R>) -> Self {
        Self { recipe_repo }
    }

    /// Executes the use case to update a recipe
    ///
    /// # Arguments
    /// * `recipe_id` - The UUID of the recipe to update
    /// * `command` - The update command with optional fields to change
    ///
    /// # Returns
    /// RecipeResponse with updated recipe details
    ///
    /// # Errors
    /// * `InventoryError::RecipeNotFound` - If the recipe doesn't exist
    /// * `InventoryError::InvalidYieldQuantity` - If yield_quantity is not positive
    pub async fn execute(
        &self,
        recipe_id: uuid::Uuid,
        command: UpdateRecipeCommand,
    ) -> Result<RecipeResponse, InventoryError> {
        let recipe_id_vo = RecipeId::from_uuid(recipe_id);

        // Find the recipe
        let mut recipe = self
            .recipe_repo
            .find_by_id(recipe_id_vo)
            .await?
            .ok_or(InventoryError::RecipeNotFound(recipe_id))?;

        // Apply updates from command
        if let Some(name) = command.name {
            recipe.set_name(name);
        }

        if let Some(description) = command.description {
            recipe.set_description(Some(description));
        }

        if let Some(yield_quantity) = command.yield_quantity {
            recipe.set_yield_quantity(yield_quantity)?;
        }

        if let Some(preparation_time_minutes) = command.preparation_time_minutes {
            recipe.set_preparation_time_minutes(Some(preparation_time_minutes));
        }

        if let Some(calculate_cost) = command.calculate_cost_from_ingredients {
            recipe.set_calculate_cost_from_ingredients(calculate_cost);
        }

        if let Some(notes) = command.notes {
            recipe.set_notes(Some(notes));
        }

        if let Some(metadata) = command.metadata {
            recipe.set_metadata(metadata);
        }

        if let Some(is_active) = command.is_active {
            if is_active {
                recipe.activate();
            } else {
                recipe.deactivate();
            }
        }

        // If version was explicitly provided, increment it
        if command.version.is_some() {
            recipe.increment_version();
        }

        // Save updated recipe
        self.recipe_repo.update(&recipe).await?;

        // Build response
        Ok(RecipeResponse {
            id: recipe.id().into_uuid(),
            product_id: recipe.product_id().map(|id| id.into_uuid()),
            variant_id: recipe.variant_id().map(|id| id.into_uuid()),
            name: recipe.name().to_string(),
            description: recipe.description().map(|s| s.to_string()),
            version: recipe.version(),
            yield_quantity: recipe.yield_quantity(),
            is_active: recipe.is_active(),
            preparation_time_minutes: recipe.preparation_time_minutes(),
            calculate_cost_from_ingredients: recipe.calculate_cost_from_ingredients(),
            calculated_cost: None,
            created_at: recipe.created_at(),
            updated_at: recipe.updated_at(),
        })
    }
}
