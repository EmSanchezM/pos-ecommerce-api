// ListCategoriesUseCase - lists product categories with tree support

use std::sync::Arc;

use crate::InventoryError;
use crate::application::dtos::responses::{CategoryResponse, CategoryTreeResponse};
use crate::domain::entities::ProductCategory;
use crate::domain::repositories::CategoryRepository;
use crate::domain::value_objects::CategoryId;

/// Use case for listing product categories
pub struct ListCategoriesUseCase<C>
where
    C: CategoryRepository,
{
    category_repo: Arc<C>,
}

impl<C> ListCategoriesUseCase<C>
where
    C: CategoryRepository,
{
    pub fn new(category_repo: Arc<C>) -> Self {
        Self { category_repo }
    }

    /// Lists all active categories as a flat list
    pub async fn execute_flat(&self) -> Result<Vec<CategoryResponse>, InventoryError> {
        let categories = self.category_repo.find_all_active().await?;
        Ok(categories.iter().map(Self::to_response).collect())
    }

    /// Lists categories as a hierarchical tree starting from root categories
    pub async fn execute_tree(&self) -> Result<Vec<CategoryTreeResponse>, InventoryError> {
        let all_categories = self.category_repo.find_all_active().await?;

        // Separate roots and children
        let roots: Vec<&ProductCategory> = all_categories.iter().filter(|c| c.is_root()).collect();

        let tree: Vec<CategoryTreeResponse> = roots
            .iter()
            .map(|root| self.build_tree(root, &all_categories))
            .collect();

        Ok(tree)
    }

    /// Lists children of a specific parent category
    pub async fn execute_children(
        &self,
        parent_id: uuid::Uuid,
    ) -> Result<Vec<CategoryResponse>, InventoryError> {
        let parent_id = CategoryId::from_uuid(parent_id);
        let children = self.category_repo.find_children(parent_id).await?;
        Ok(children.iter().map(Self::to_response).collect())
    }

    fn build_tree(
        &self,
        category: &ProductCategory,
        all: &[ProductCategory],
    ) -> CategoryTreeResponse {
        let children: Vec<CategoryTreeResponse> = all
            .iter()
            .filter(|c| c.parent_id() == Some(category.id()))
            .map(|child| self.build_tree(child, all))
            .collect();

        CategoryTreeResponse {
            id: category.id().into_uuid(),
            parent_id: category.parent_id().map(|id| id.into_uuid()),
            name: category.name().to_string(),
            description: category.description().map(|s| s.to_string()),
            slug: category.slug().to_string(),
            icon: category.icon().map(|s| s.to_string()),
            sort_order: category.sort_order(),
            is_active: category.is_active(),
            children,
            created_at: category.created_at(),
            updated_at: category.updated_at(),
        }
    }

    fn to_response(category: &ProductCategory) -> CategoryResponse {
        CategoryResponse {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        fn add_category(&self, category: ProductCategory) {
            let mut categories = self.categories.lock().unwrap();
            categories.insert(category.id(), category);
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
            let categories = self.categories.lock().unwrap();
            Ok(categories
                .values()
                .filter(|c| c.is_root() && c.is_active())
                .cloned()
                .collect())
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
            let categories = self.categories.lock().unwrap();
            Ok(categories
                .values()
                .filter(|c| c.is_active())
                .cloned()
                .collect())
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
    async fn test_list_flat() {
        let repo = Arc::new(MockCategoryRepository::new());

        let cat1 = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let cat2 = ProductCategory::create("Clothing".to_string(), "clothing".to_string());
        repo.add_category(cat1);
        repo.add_category(cat2);

        let use_case = ListCategoriesUseCase::new(repo);
        let result = use_case.execute_flat().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_list_tree() {
        let repo = Arc::new(MockCategoryRepository::new());

        let parent = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let parent_id = parent.id();
        repo.add_category(parent);

        let child = ProductCategory::create_subcategory(
            parent_id,
            "Phones".to_string(),
            "phones".to_string(),
        );
        repo.add_category(child);

        let use_case = ListCategoriesUseCase::new(repo);
        let result = use_case.execute_tree().await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].children.len(), 1);
        assert_eq!(result[0].children[0].name, "Phones");
    }

    #[tokio::test]
    async fn test_list_children() {
        let repo = Arc::new(MockCategoryRepository::new());

        let parent = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        let parent_id = parent.id();
        repo.add_category(parent);

        let child = ProductCategory::create_subcategory(
            parent_id,
            "Phones".to_string(),
            "phones".to_string(),
        );
        repo.add_category(child);

        let use_case = ListCategoriesUseCase::new(repo);
        let result = use_case
            .execute_children(parent_id.into_uuid())
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Phones");
    }
}
