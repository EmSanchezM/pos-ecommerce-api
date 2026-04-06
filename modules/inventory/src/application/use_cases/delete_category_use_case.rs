// DeleteCategoryUseCase - soft-deletes a category by deactivating it

use std::sync::Arc;

use crate::InventoryError;
use crate::domain::repositories::CategoryRepository;
use crate::domain::value_objects::CategoryId;

/// Use case for deleting (deactivating) a product category
pub struct DeleteCategoryUseCase<C>
where
    C: CategoryRepository,
{
    category_repo: Arc<C>,
}

impl<C> DeleteCategoryUseCase<C>
where
    C: CategoryRepository,
{
    pub fn new(category_repo: Arc<C>) -> Self {
        Self { category_repo }
    }

    pub async fn execute(&self, id: uuid::Uuid) -> Result<(), InventoryError> {
        let category_id = CategoryId::from_uuid(id);
        let mut category = self
            .category_repo
            .find_by_id(category_id)
            .await?
            .ok_or(InventoryError::CategoryNotFound(id))?;

        // Check for children - don't delete if has active children
        let children = self.category_repo.find_children(category_id).await?;
        let active_children: Vec<_> = children.iter().filter(|c| c.is_active()).collect();
        if !active_children.is_empty() {
            return Err(InventoryError::InvalidOperation(
                "Cannot delete category with active children".to_string(),
            ));
        }

        category.deactivate();
        self.category_repo.update(&category).await?;

        Ok(())
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
            parent_id: CategoryId,
        ) -> Result<Vec<ProductCategory>, InventoryError> {
            let categories = self.categories.lock().unwrap();
            Ok(categories
                .values()
                .filter(|c| c.parent_id() == Some(parent_id) && c.is_active())
                .cloned()
                .collect())
        }
        async fn find_all_active(&self) -> Result<Vec<ProductCategory>, InventoryError> {
            Ok(vec![])
        }
        async fn update(&self, category: &ProductCategory) -> Result<(), InventoryError> {
            let mut categories = self.categories.lock().unwrap();
            categories.insert(category.id(), category.clone());
            Ok(())
        }
        async fn delete(&self, _id: CategoryId) -> Result<(), InventoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_delete_category_success() {
        let repo = Arc::new(MockCategoryRepository::new());
        let category =
            ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let category_id = category.id().into_uuid();
        repo.save(&category).await.unwrap();

        let use_case = DeleteCategoryUseCase::new(repo.clone());
        let result = use_case.execute(category_id).await;
        assert!(result.is_ok());

        // Verify it was deactivated
        let updated = repo
            .find_by_id(CategoryId::from_uuid(category_id))
            .await
            .unwrap()
            .unwrap();
        assert!(!updated.is_active());
    }

    #[tokio::test]
    async fn test_delete_category_with_children() {
        let repo = Arc::new(MockCategoryRepository::new());

        let parent = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let parent_id = parent.id();
        repo.save(&parent).await.unwrap();

        let child = ProductCategory::create_subcategory(
            parent_id,
            "Phones".to_string(),
            "phones".to_string(),
        );
        repo.save(&child).await.unwrap();

        let use_case = DeleteCategoryUseCase::new(repo);
        let result = use_case.execute(parent_id.into_uuid()).await;
        assert!(matches!(result, Err(InventoryError::InvalidOperation(_))));
    }

    #[tokio::test]
    async fn test_delete_category_not_found() {
        let repo = Arc::new(MockCategoryRepository::new());
        let use_case = DeleteCategoryUseCase::new(repo);
        let result = use_case.execute(CategoryId::new().into_uuid()).await;
        assert!(matches!(result, Err(InventoryError::CategoryNotFound(_))));
    }
}
