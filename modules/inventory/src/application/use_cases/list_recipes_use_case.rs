// ListRecipesUseCase - lists recipes with pagination and filters

use std::sync::Arc;

use crate::application::dtos::responses::{PaginatedResponse, RecipeResponse};
use crate::domain::repositories::RecipeRepository;
use crate::InventoryError;

/// Query parameters for listing recipes
#[derive(Debug, Clone)]
pub struct ListRecipesQuery {
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search term for name/description
    pub search: Option<String>,
    /// Page number (1-indexed)
    pub page: i64,
    /// Number of items per page
    pub page_size: i64,
}

impl Default for ListRecipesQuery {
    fn default() -> Self {
        Self {
            is_active: None,
            search: None,
            page: 1,
            page_size: 20,
        }
    }
}

/// Use case for listing recipes with pagination and filters
pub struct ListRecipesUseCase<R>
where
    R: RecipeRepository,
{
    recipe_repo: Arc<R>,
}

impl<R> ListRecipesUseCase<R>
where
    R: RecipeRepository,
{
    /// Creates a new instance of ListRecipesUseCase
    pub fn new(recipe_repo: Arc<R>) -> Self {
        Self { recipe_repo }
    }

    /// Executes the use case to list recipes
    ///
    /// # Arguments
    /// * `query` - Query parameters with filters and pagination
    ///
    /// # Returns
    /// Paginated response with products
    pub async fn execute(
        &self,
        query: ListRecipesQuery,
    ) -> Result<PaginatedResponse<RecipeResponse>, InventoryError> {
        // Validate and clamp pagination
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);

        // Fetch recipes with pagination
        let (recipes, total_items) = self
            .recipe_repo
            .find_paginated(
                query.is_active,
                query.search.as_deref(),
                page,
                page_size,
            )
            .await?;

        // Convert to response DTOs
        let recipe_responses: Vec<RecipeResponse> = recipes
            .into_iter()
            .map(|r| RecipeResponse {
                id: r.id().into_uuid(),
                product_id: r.product_id().map(|id| id.into_uuid()),
                variant_id: r.variant_id().map(|id| id.into_uuid()),
                name: r.name().to_string(),
                description: r.description().map(|s| s.to_string()),
                version: r.version(),
                yield_quantity: r.yield_quantity(),
                preparation_time_minutes: r.preparation_time_minutes(),
                calculate_cost_from_ingredients: r.calculate_cost_from_ingredients(),
                calculated_cost: None,
                is_active: r.is_active(),
                created_at: r.created_at(),
                updated_at: r.updated_at(),
            })
            .collect();

        Ok(PaginatedResponse::new(
            recipe_responses,
            page,
            page_size,
            total_items,
        ))
    }
}
