// UpdateCategoryUseCase - updates an existing product category

use std::sync::Arc;

use crate::InventoryError;
use crate::application::dtos::commands::UpdateCategoryCommand;
use crate::application::dtos::responses::CategoryResponse;
use crate::domain::repositories::CategoryRepository;
use crate::domain::value_objects::CategoryId;

/// Use case for updating an existing product category
pub struct UpdateCategoryUseCase<C>
where
    C: CategoryRepository,
{
    category_repo: Arc<C>,
}

impl<C> UpdateCategoryUseCase<C>
where
    C: CategoryRepository,
{
    pub fn new(category_repo: Arc<C>) -> Self {
        Self { category_repo }
    }

    pub async fn execute(
        &self,
        id: uuid::Uuid,
        command: UpdateCategoryCommand,
    ) -> Result<CategoryResponse, InventoryError> {
        let category_id = CategoryId::from_uuid(id);
        let mut category = self
            .category_repo
            .find_by_id(category_id)
            .await?
            .ok_or(InventoryError::CategoryNotFound(id))?;

        // Validate slug uniqueness if changing
        if let Some(ref new_slug) = command.slug {
            if new_slug != category.slug() {
                if self
                    .category_repo
                    .find_by_slug(new_slug)
                    .await?
                    .is_some()
                {
                    return Err(InventoryError::DuplicateCategorySlug(new_slug.clone()));
                }
            }
        }

        // Validate parent exists if changing
        if let Some(parent_uuid) = command.parent_id {
            let parent_id = CategoryId::from_uuid(parent_uuid);
            // Prevent self-referencing
            if parent_id == category_id {
                return Err(InventoryError::InvalidOperation(
                    "Category cannot be its own parent".to_string(),
                ));
            }
            if self.category_repo.find_by_id(parent_id).await?.is_none() {
                return Err(InventoryError::ParentCategoryNotFound(parent_uuid));
            }
        }

        // Apply updates
        if let Some(name) = command.name {
            category.set_name(name);
        }
        if let Some(slug) = command.slug {
            category.set_slug(slug);
        }
        if let Some(parent_uuid) = command.parent_id {
            category.set_parent_id(Some(CategoryId::from_uuid(parent_uuid)));
        }
        if let Some(description) = command.description {
            category.set_description(Some(description));
        }
        if let Some(icon) = command.icon {
            category.set_icon(Some(icon));
        }
        if let Some(sort_order) = command.sort_order {
            category.set_sort_order(sort_order);
        }
        if let Some(is_active) = command.is_active {
            category.set_active(is_active);
        }

        self.category_repo.update(&category).await?;

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
    use crate::domain::entities::ProductCategory;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

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
            Ok(vec![])
        }
        async fn find_children(
            &self,
            _parent_id: CategoryId,
        ) -> Result<Vec<ProductCategory>, InventoryError> {
            Ok(vec![])
        }
        async fn find_all_active(&self) -> Result<Vec<ProductCategory>, InventoryError> {
            Ok(vec![])
        }
        async fn update(&self, category: &ProductCategory) -> Result<(), InventoryError> {
            let mut categories = self.categories.lock().unwrap();
            let mut slugs = self.slugs.lock().unwrap();
            categories.insert(category.id(), category.clone());
            slugs.insert(category.slug().to_string(), category.id());
            Ok(())
        }
        async fn delete(&self, _id: CategoryId) -> Result<(), InventoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_update_category_name() {
        let repo = Arc::new(MockCategoryRepository::new());
        let category =
            ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let category_id = category.id().into_uuid();
        repo.save(&category).await.unwrap();

        let use_case = UpdateCategoryUseCase::new(repo);
        let command = UpdateCategoryCommand {
            name: Some("Tech".to_string()),
            slug: None,
            parent_id: None,
            description: None,
            icon: None,
            sort_order: None,
            is_active: None,
        };

        let result = use_case.execute(category_id, command).await.unwrap();
        assert_eq!(result.name, "Tech");
        assert_eq!(result.slug, "electronics");
    }

    #[tokio::test]
    async fn test_update_category_duplicate_slug() {
        let repo = Arc::new(MockCategoryRepository::new());

        let cat1 = ProductCategory::create("Cat 1".to_string(), "cat-1".to_string());
        let cat2 = ProductCategory::create("Cat 2".to_string(), "cat-2".to_string());
        let cat2_id = cat2.id().into_uuid();
        repo.save(&cat1).await.unwrap();
        repo.save(&cat2).await.unwrap();

        let use_case = UpdateCategoryUseCase::new(repo);
        let command = UpdateCategoryCommand {
            name: None,
            slug: Some("cat-1".to_string()),
            parent_id: None,
            description: None,
            icon: None,
            sort_order: None,
            is_active: None,
        };

        let result = use_case.execute(cat2_id, command).await;
        assert!(matches!(
            result,
            Err(InventoryError::DuplicateCategorySlug(_))
        ));
    }

    #[tokio::test]
    async fn test_update_category_not_found() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = UpdateCategoryUseCase::new(repo);
        let command = UpdateCategoryCommand {
            name: Some("Test".to_string()),
            slug: None,
            parent_id: None,
            description: None,
            icon: None,
            sort_order: None,
            is_active: None,
        };

        let result = use_case.execute(CategoryId::new().into_uuid(), command).await;
        assert!(matches!(result, Err(InventoryError::CategoryNotFound(_))));
    }
}
