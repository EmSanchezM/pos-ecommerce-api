// GetCategoryUseCase - gets a single category by ID

use std::sync::Arc;

use crate::InventoryError;
use crate::application::dtos::responses::CategoryResponse;
use crate::domain::repositories::CategoryRepository;
use crate::domain::value_objects::CategoryId;

/// Use case for getting a single product category by ID
pub struct GetCategoryUseCase<C>
where
    C: CategoryRepository,
{
    category_repo: Arc<C>,
}

impl<C> GetCategoryUseCase<C>
where
    C: CategoryRepository,
{
    pub fn new(category_repo: Arc<C>) -> Self {
        Self { category_repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<CategoryResponse, InventoryError> {
        let category_id = CategoryId::from_uuid(id);
        let category = self
            .category_repo
            .find_by_id(category_id)
            .await?
            .ok_or(InventoryError::CategoryNotFound(id))?;

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
    }

    impl MockCategoryRepository {
        fn new() -> Self {
            Self {
                categories: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CategoryRepository for MockCategoryRepository {
        async fn save(&self, category: &ProductCategory) -> Result<(), InventoryError> {
            let mut categories = self.categories.lock().unwrap();
            categories.insert(category.id(), category.clone());
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
            _slug: &str,
        ) -> Result<Option<ProductCategory>, InventoryError> {
            Ok(None)
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
        async fn update(&self, _category: &ProductCategory) -> Result<(), InventoryError> {
            Ok(())
        }
        async fn delete(&self, _id: CategoryId) -> Result<(), InventoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_category_success() {
        let repo = Arc::new(MockCategoryRepository::new());
        let category =
            ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let category_id = category.id().into_uuid();
        repo.save(&category).await.unwrap();

        let use_case = GetCategoryUseCase::new(repo);
        let result = use_case.execute(category_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Electronics");
    }

    #[tokio::test]
    async fn test_get_category_not_found() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = GetCategoryUseCase::new(repo);
        let result = use_case.execute(CategoryId::new().into_uuid()).await;
        assert!(matches!(result, Err(InventoryError::CategoryNotFound(_))));
    }
}
