// CreateCategoryUseCase - creates a new product category
//
// Requirements: 1A.1, 1A.2

use std::sync::Arc;

use crate::application::dtos::commands::CreateCategoryCommand;
use crate::application::dtos::responses::CategoryResponse;
use crate::domain::entities::ProductCategory;
use crate::domain::repositories::CategoryRepository;
use crate::domain::value_objects::CategoryId;
use crate::InventoryError;

/// Use case for creating a new product category
///
/// Validates slug uniqueness and parent existence before creating the category.
///
/// Requirements: 1A.1, 1A.2
pub struct CreateCategoryUseCase<C>
where
    C: CategoryRepository,
{
    category_repo: Arc<C>,
}

impl<C> CreateCategoryUseCase<C>
where
    C: CategoryRepository,
{
    /// Creates a new instance of CreateCategoryUseCase
    pub fn new(category_repo: Arc<C>) -> Self {
        Self { category_repo }
    }

    /// Executes the use case to create a new category
    ///
    /// # Arguments
    /// * `command` - The create category command containing category data
    ///
    /// # Returns
    /// CategoryResponse on success
    ///
    /// # Errors
    /// * `InventoryError::DuplicateCategorySlug` - If slug already exists
    /// * `InventoryError::ParentCategoryNotFound` - If parent_id is provided but doesn't exist
    pub async fn execute(
        &self,
        command: CreateCategoryCommand,
    ) -> Result<CategoryResponse, InventoryError> {
        // Validate slug uniqueness (Requirement 1A.2)
        if self
            .category_repo
            .find_by_slug(&command.slug)
            .await?
            .is_some()
        {
            return Err(InventoryError::DuplicateCategorySlug(command.slug));
        }

        // Validate parent exists if provided (Requirement 1A.2)
        if let Some(parent_uuid) = command.parent_id {
            let parent_id = CategoryId::from_uuid(parent_uuid);
            if self.category_repo.find_by_id(parent_id).await?.is_none() {
                return Err(InventoryError::ParentCategoryNotFound(parent_uuid));
            }
        }

        // Create category entity
        let mut category = if let Some(parent_uuid) = command.parent_id {
            let parent_id = CategoryId::from_uuid(parent_uuid);
            ProductCategory::create_subcategory(parent_id, command.name, command.slug)
        } else {
            ProductCategory::create(command.name, command.slug)
        };

        // Apply optional fields
        if let Some(description) = command.description {
            category.set_description(Some(description));
        }
        if let Some(icon) = command.icon {
            category.set_icon(Some(icon));
        }
        category.set_sort_order(command.sort_order);

        // Save to repository
        self.category_repo.save(&category).await?;

        // Convert to response
        Ok(CategoryResponse {
            id: category.id().into_uuid(),
            parent_id: category.parent_id().map(|id| id.into_uuid()),
            name: category.name().to_string(),
            description: category.description().map(|s| s.to_string()),
            slug: category.slug().to_string(),
            icon: category.icon().map(|s| s.to_string()),
            sort_order: category.sort_order(),
            is_active: category.is_active(),
            created_at: category.created_at(),
            updated_at: category.updated_at(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock repository for testing
    struct MockCategoryRepository {
        categories: Mutex<HashMap<CategoryId, ProductCategory>>,
        slugs: Mutex<HashMap<String, CategoryId>>,
    }

    impl MockCategoryRepository {
        fn new() -> Self {
            Self {
                categories: Mutex::new(HashMap::new()),
                slugs: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CategoryRepository for MockCategoryRepository {
        async fn save(&self, category: &ProductCategory) -> Result<(), InventoryError> {
            let mut categories = self.categories.lock().unwrap();
            let mut slugs = self.slugs.lock().unwrap();
            categories.insert(category.id(), category.clone());
            slugs.insert(category.slug().to_string(), category.id());
            Ok(())
        }

        async fn find_by_id(
            &self,
            id: CategoryId,
        ) -> Result<Option<ProductCategory>, InventoryError> {
            let categories = self.categories.lock().unwrap();
            Ok(categories.get(&id).cloned())
        }

        async fn find_by_slug(
            &self,
            slug: &str,
        ) -> Result<Option<ProductCategory>, InventoryError> {
            let slugs = self.slugs.lock().unwrap();
            let categories = self.categories.lock().unwrap();
            Ok(slugs.get(slug).and_then(|id| categories.get(id).cloned()))
        }

        async fn find_root_categories(&self) -> Result<Vec<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn find_children(
            &self,
            _parent_id: CategoryId,
        ) -> Result<Vec<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn find_all_active(&self) -> Result<Vec<ProductCategory>, InventoryError> {
            unimplemented!()
        }

        async fn update(&self, _category: &ProductCategory) -> Result<(), InventoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: CategoryId) -> Result<(), InventoryError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_create_root_category() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = CreateCategoryUseCase::new(repo);

        let command = CreateCategoryCommand {
            name: "Electronics".to_string(),
            slug: "electronics".to_string(),
            parent_id: None,
            description: Some("Electronic devices".to_string()),
            icon: Some("icon-electronics".to_string()),
            sort_order: 1,
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.name, "Electronics");
        assert_eq!(response.slug, "electronics");
        assert_eq!(response.description, Some("Electronic devices".to_string()));
        assert_eq!(response.icon, Some("icon-electronics".to_string()));
        assert_eq!(response.sort_order, 1);
        assert!(response.is_active);
        assert!(response.parent_id.is_none());
    }

    #[tokio::test]
    async fn test_create_subcategory() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = CreateCategoryUseCase::new(repo.clone());

        // First create parent
        let parent = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        repo.save(&parent).await.unwrap();

        // Then create subcategory
        let command = CreateCategoryCommand {
            name: "Smartphones".to_string(),
            slug: "smartphones".to_string(),
            parent_id: Some(parent.id().into_uuid()),
            description: None,
            icon: None,
            sort_order: 0,
        };

        let result = use_case.execute(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.name, "Smartphones");
        assert_eq!(response.parent_id, Some(parent.id().into_uuid()));
    }

    #[tokio::test]
    async fn test_duplicate_slug_error() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = CreateCategoryUseCase::new(repo.clone());

        // Create first category
        let category = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        repo.save(&category).await.unwrap();

        // Try to create another with same slug
        let command = CreateCategoryCommand {
            name: "Electronics 2".to_string(),
            slug: "electronics".to_string(),
            parent_id: None,
            description: None,
            icon: None,
            sort_order: 0,
        };

        let result = use_case.execute(command).await;
        assert!(matches!(
            result,
            Err(InventoryError::DuplicateCategorySlug(_))
        ));
    }

    #[tokio::test]
    async fn test_parent_not_found_error() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = CreateCategoryUseCase::new(repo);

        let non_existent_parent = CategoryId::new().into_uuid();
        let command = CreateCategoryCommand {
            name: "Smartphones".to_string(),
            slug: "smartphones".to_string(),
            parent_id: Some(non_existent_parent),
            description: None,
            icon: None,
            sort_order: 0,
        };

        let result = use_case.execute(command).await;
        assert!(matches!(
            result,
            Err(InventoryError::ParentCategoryNotFound(_))
        ));
    }
}
