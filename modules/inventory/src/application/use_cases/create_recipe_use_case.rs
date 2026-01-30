// CreateRecipeUseCase - creates a new recipe for a product or variant
//
// - Validates no active recipe exists for product/variant
// - Creates recipe with ingredients and substitutes

use std::str::FromStr;
use std::sync::Arc;

use crate::application::dtos::commands::CreateRecipeCommand;
use crate::application::dtos::responses::RecipeResponse;
use crate::domain::entities::{IngredientSubstitute, Recipe, RecipeIngredient};
use crate::domain::repositories::{ProductRepository, RecipeRepository};
use crate::domain::value_objects::{ProductId, UnitOfMeasure, VariantId};
use crate::InventoryError;

/// Use case for creating a new recipe
///
/// Validates that no active recipe exists for the product/variant,
/// creates the recipe with ingredients and substitutes.
pub struct CreateRecipeUseCase<R, P>
where
    R: RecipeRepository,
    P: ProductRepository,
{
    recipe_repo: Arc<R>,
    product_repo: Arc<P>,
}

impl<R, P> CreateRecipeUseCase<R, P>
where
    R: RecipeRepository,
    P: ProductRepository,
{
    /// Creates a new instance of CreateRecipeUseCase
    pub fn new(recipe_repo: Arc<R>, product_repo: Arc<P>) -> Self {
        Self {
            recipe_repo,
            product_repo,
        }
    }

    /// Executes the use case to create a new recipe
    ///
    /// # Arguments
    /// * `command` - The create recipe command containing recipe data
    ///
    /// # Returns
    /// RecipeResponse on success
    ///
    /// # Errors
    /// * `InventoryError::InvalidProductVariantConstraint` - If neither or both product_id and variant_id are provided
    /// * `InventoryError::ProductNotFound` - If product_id is provided but doesn't exist
    /// * `InventoryError::VariantNotFound` - If variant_id is provided but doesn't exist
    /// * `InventoryError::ActiveRecipeExists` - If an active recipe already exists for the product/variant
    /// * `InventoryError::InvalidYieldQuantity` - If yield_quantity is not positive
    /// * `InventoryError::InvalidUnitOfMeasure` - If ingredient unit of measure is invalid
    pub async fn execute(&self, command: CreateRecipeCommand) -> Result<RecipeResponse, InventoryError> {
        // Validate XOR constraint: exactly one of product_id or variant_id must be set
        let (product_id, variant_id) = self.validate_product_variant_constraint(
            command.product_id,
            command.variant_id,
        )?;

        // Validate product/variant exists and check for active recipe
        if let Some(pid) = product_id {
            // Validate product exists
            if self.product_repo.find_by_id(pid).await?.is_none() {
                return Err(InventoryError::ProductNotFound(pid.into_uuid()));
            }
            // Check no active recipe exists (Requirement 6.4)
            if self.recipe_repo.find_active_by_product(pid).await?.is_some() {
                return Err(InventoryError::ActiveRecipeExists);
            }
        }

        if let Some(vid) = variant_id {
            // Validate variant exists
            if self.product_repo.find_variant_by_id(vid).await?.is_none() {
                return Err(InventoryError::VariantNotFound(vid.into_uuid()));
            }
            // Check no active recipe exists (Requirement 6.4)
            if self.recipe_repo.find_active_by_variant(vid).await?.is_some() {
                return Err(InventoryError::ActiveRecipeExists);
            }
        }

        // Create recipe entity (Requirement 6.1)
        let recipe = if let Some(pid) = product_id {
            Recipe::create_for_product(pid, command.name.clone(), command.yield_quantity)?
        } else {
            Recipe::create_for_variant(variant_id.unwrap(), command.name.clone(), command.yield_quantity)?
        };

        // Save recipe first
        self.recipe_repo.save(&recipe).await?;

        // Create and save ingredients
        for ing_cmd in &command.ingredients {
            let ingredient = self.create_ingredient(&recipe, ing_cmd).await?;
            self.recipe_repo.save_ingredient(&ingredient).await?;

            // Create and save substitutes for this ingredient
            for sub_cmd in &ing_cmd.substitutes {
                let substitute = self.create_substitute(&ingredient, sub_cmd)?;
                self.recipe_repo.save_substitute(&substitute).await?;
            }
        }

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
            calculated_cost: None, // Cost calculation is done separately
            created_at: recipe.created_at(),
            updated_at: recipe.updated_at(),
        })
    }

    /// Validates the product/variant XOR constraint
    fn validate_product_variant_constraint(
        &self,
        product_id: Option<uuid::Uuid>,
        variant_id: Option<uuid::Uuid>,
    ) -> Result<(Option<ProductId>, Option<VariantId>), InventoryError> {
        match (product_id, variant_id) {
            (Some(pid), None) => Ok((Some(ProductId::from_uuid(pid)), None)),
            (None, Some(vid)) => Ok((None, Some(VariantId::from_uuid(vid)))),
            _ => Err(InventoryError::InvalidProductVariantConstraint),
        }
    }

    /// Creates a RecipeIngredient from command data
    async fn create_ingredient(
        &self,
        recipe: &Recipe,
        cmd: &crate::application::dtos::commands::RecipeIngredientCommand,
    ) -> Result<RecipeIngredient, InventoryError> {
        // Validate XOR constraint for ingredient
        let (ing_product_id, ing_variant_id) = self.validate_product_variant_constraint(
            cmd.ingredient_product_id,
            cmd.ingredient_variant_id,
        )?;

        // Validate ingredient product/variant exists
        if let Some(pid) = ing_product_id
            && self.product_repo.find_by_id(pid).await?.is_none() {
                return Err(InventoryError::ProductNotFound(pid.into_uuid()));
            }
        if let Some(vid) = ing_variant_id
            && self.product_repo.find_variant_by_id(vid).await?.is_none() {
                return Err(InventoryError::VariantNotFound(vid.into_uuid()));
            }

        // Parse unit of measure
        let unit_of_measure = UnitOfMeasure::from_str(&cmd.unit_of_measure)?;

        // Create ingredient
        let mut ingredient = if let Some(pid) = ing_product_id {
            RecipeIngredient::create_for_product(recipe.id(), pid, cmd.quantity, unit_of_measure)?
        } else {
            RecipeIngredient::create_for_variant(recipe.id(), ing_variant_id.unwrap(), cmd.quantity, unit_of_measure)?
        };

        // Apply optional fields
        ingredient.set_optional(cmd.is_optional);
        ingredient.set_can_substitute(cmd.can_substitute);
        ingredient.set_sort_order(cmd.sort_order);
        if let Some(ref step) = cmd.preparation_step {
            ingredient.set_preparation_step(Some(step.clone()));
        }
        if let Some(cost) = cmd.estimated_cost_per_unit {
            ingredient.set_estimated_cost_per_unit(Some(cost));
        }
        if cmd.estimated_waste_percentage > rust_decimal::Decimal::ZERO {
            ingredient.set_estimated_waste_percentage(cmd.estimated_waste_percentage)?;
        }
        if let Some(ref notes) = cmd.notes {
            ingredient.set_notes(Some(notes.clone()));
        }

        Ok(ingredient)
    }

    /// Creates an IngredientSubstitute from command data
    fn create_substitute(
        &self,
        ingredient: &RecipeIngredient,
        cmd: &crate::application::dtos::commands::IngredientSubstituteCommand,
    ) -> Result<IngredientSubstitute, InventoryError> {
        // Validate ingredient allows substitutes
        if !ingredient.can_substitute() {
            return Err(InventoryError::SubstitutesNotAllowed);
        }

        // Validate XOR constraint for substitute
        match (cmd.substitute_product_id, cmd.substitute_variant_id) {
            (Some(pid), None) => {
                IngredientSubstitute::create_for_product(
                    ingredient.id(),
                    ProductId::from_uuid(pid),
                    cmd.conversion_ratio,
                    cmd.priority,
                )
            }
            (None, Some(vid)) => {
                IngredientSubstitute::create_for_variant(
                    ingredient.id(),
                    VariantId::from_uuid(vid),
                    cmd.conversion_ratio,
                    cmd.priority,
                )
            }
            _ => Err(InventoryError::InvalidProductVariantConstraint),
        }
    }
}
