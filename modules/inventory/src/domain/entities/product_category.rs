// ProductCategory entity - hierarchical product categorization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::CategoryId;

/// ProductCategory entity for organizing products into a hierarchical structure.
/// Supports parent-child relationships for nested categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    id: CategoryId,
    parent_id: Option<CategoryId>,
    name: String,
    description: Option<String>,
    slug: String,
    icon: Option<String>,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ProductCategory {
    /// Creates a new root category (no parent)
    pub fn create(name: String, slug: String) -> Self {
        let now = Utc::now();
        Self {
            id: CategoryId::new(),
            parent_id: None,
            name,
            description: None,
            slug,
            icon: None,
            sort_order: 0,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new subcategory under a parent category
    pub fn create_subcategory(parent_id: CategoryId, name: String, slug: String) -> Self {
        let mut category = Self::create(name, slug);
        category.parent_id = Some(parent_id);
        category
    }

    /// Reconstitutes a ProductCategory from persistence
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: CategoryId,
        parent_id: Option<CategoryId>,
        name: String,
        description: Option<String>,
        slug: String,
        icon: Option<String>,
        sort_order: i32,
        is_active: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            parent_id,
            name,
            description,
            slug,
            icon,
            sort_order,
            is_active,
            created_at,
            updated_at,
        }
    }

    /// Returns true if this is a root category (no parent)
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    // =========================================================================
    // Getters
    // =========================================================================

    pub fn id(&self) -> CategoryId {
        self.id
    }

    pub fn parent_id(&self) -> Option<CategoryId> {
        self.parent_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    // =========================================================================
    // Setters
    // =========================================================================

    pub fn set_parent_id(&mut self, parent_id: Option<CategoryId>) {
        self.parent_id = parent_id;
        self.updated_at = Utc::now();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_slug(&mut self, slug: String) {
        self.slug = slug;
        self.updated_at = Utc::now();
    }

    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
        self.updated_at = Utc::now();
    }

    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.updated_at = Utc::now();
    }

    pub fn set_active(&mut self, is_active: bool) {
        self.is_active = is_active;
        self.updated_at = Utc::now();
    }

    /// Deactivates the category without deleting it
    pub fn deactivate(&mut self) {
        self.set_active(false);
    }

    /// Activates the category
    pub fn activate(&mut self) {
        self.set_active(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_root_category() {
        let category = ProductCategory::create("Electronics".to_string(), "electronics".to_string());
        
        assert!(category.is_root());
        assert_eq!(category.name(), "Electronics");
        assert_eq!(category.slug(), "electronics");
        assert!(category.is_active());
        assert_eq!(category.sort_order(), 0);
        assert!(category.description().is_none());
        assert!(category.icon().is_none());
    }

    #[test]
    fn test_create_subcategory() {
        let parent_id = CategoryId::new();
        let category = ProductCategory::create_subcategory(
            parent_id,
            "Smartphones".to_string(),
            "smartphones".to_string(),
        );
        
        assert!(!category.is_root());
        assert_eq!(category.parent_id(), Some(parent_id));
        assert_eq!(category.name(), "Smartphones");
    }

    #[test]
    fn test_setters() {
        let mut category = ProductCategory::create("Test".to_string(), "test".to_string());
        let original_updated_at = category.updated_at();
        
        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        category.set_name("Updated Name".to_string());
        assert_eq!(category.name(), "Updated Name");
        assert!(category.updated_at() >= original_updated_at);
        
        category.set_description(Some("A description".to_string()));
        assert_eq!(category.description(), Some("A description"));
        
        category.set_icon(Some("icon.png".to_string()));
        assert_eq!(category.icon(), Some("icon.png"));
        
        category.set_sort_order(10);
        assert_eq!(category.sort_order(), 10);
    }

    #[test]
    fn test_deactivate_activate() {
        let mut category = ProductCategory::create("Test".to_string(), "test".to_string());
        assert!(category.is_active());
        
        category.deactivate();
        assert!(!category.is_active());
        
        category.activate();
        assert!(category.is_active());
    }

    #[test]
    fn test_reconstitute() {
        let id = CategoryId::new();
        let parent_id = Some(CategoryId::new());
        let now = Utc::now();
        
        let category = ProductCategory::reconstitute(
            id,
            parent_id,
            "Reconstituted".to_string(),
            Some("Description".to_string()),
            "reconstituted".to_string(),
            Some("icon.svg".to_string()),
            5,
            true,
            now,
            now,
        );
        
        assert_eq!(category.id(), id);
        assert_eq!(category.parent_id(), parent_id);
        assert_eq!(category.name(), "Reconstituted");
        assert_eq!(category.description(), Some("Description"));
        assert_eq!(category.slug(), "reconstituted");
        assert_eq!(category.icon(), Some("icon.svg"));
        assert_eq!(category.sort_order(), 5);
        assert!(category.is_active());
    }
}
