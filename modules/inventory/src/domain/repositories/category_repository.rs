// CategoryRepository trait - repository for product category operations

use async_trait::async_trait;

use crate::domain::entities::ProductCategory;
use crate::domain::value_objects::CategoryId;
use crate::InventoryError;

/// Repository trait for ProductCategory persistence operations.
/// Supports hierarchical category management with parent-child relationships.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Saves a new category to the repository
    async fn save(&self, category: &ProductCategory) -> Result<(), InventoryError>;

    /// Finds a category by its unique ID
    async fn find_by_id(&self, id: CategoryId) -> Result<Option<ProductCategory>, InventoryError>;

    /// Finds a category by its unique slug
    async fn find_by_slug(&self, slug: &str) -> Result<Option<ProductCategory>, InventoryError>;

    /// Finds all root categories (categories without a parent)
    /// Results are ordered by sort_order
    async fn find_root_categories(&self) -> Result<Vec<ProductCategory>, InventoryError>;

    /// Finds all direct children of a parent category
    /// Results are ordered by sort_order
    async fn find_children(&self, parent_id: CategoryId) -> Result<Vec<ProductCategory>, InventoryError>;

    /// Finds all active categories
    /// Results are ordered by sort_order within each level
    async fn find_all_active(&self) -> Result<Vec<ProductCategory>, InventoryError>;

    /// Updates an existing category
    async fn update(&self, category: &ProductCategory) -> Result<(), InventoryError>;

    /// Deletes a category by ID
    /// Child categories will have their parent_id set to NULL (orphaned)
    async fn delete(&self, id: CategoryId) -> Result<(), InventoryError>;
}
