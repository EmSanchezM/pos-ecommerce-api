//! ProductListing - public/eCommerce-facing wrapper over inventory.products.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::CatalogError;
use crate::domain::value_objects::ProductListingId;
use identity::StoreId;

/// Validates a slug: lowercase letters/digits/hyphens, no leading/trailing
/// hyphen, no consecutive hyphens, length between 2 and 200.
fn is_valid_slug(slug: &str) -> bool {
    let len = slug.len();
    if !(2..=200).contains(&len) {
        return false;
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return false;
    }
    let mut prev_hyphen = false;
    for c in slug.chars() {
        match c {
            'a'..='z' | '0'..='9' => prev_hyphen = false,
            '-' => {
                if prev_hyphen {
                    return false;
                }
                prev_hyphen = true;
            }
            _ => return false,
        }
    }
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductListing {
    id: ProductListingId,
    store_id: StoreId,
    product_id: Uuid,
    slug: String,
    title: String,
    short_description: Option<String>,
    long_description: Option<String>,
    is_published: bool,
    is_featured: bool,
    seo_title: Option<String>,
    seo_description: Option<String>,
    seo_keywords: Vec<String>,
    sort_order: i32,
    view_count: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ProductListing {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        store_id: StoreId,
        product_id: Uuid,
        slug: String,
        title: String,
        short_description: Option<String>,
        long_description: Option<String>,
        seo_title: Option<String>,
        seo_description: Option<String>,
        seo_keywords: Vec<String>,
        sort_order: i32,
    ) -> Result<Self, CatalogError> {
        if !is_valid_slug(&slug) {
            return Err(CatalogError::InvalidSlug);
        }
        let now = Utc::now();
        Ok(Self {
            id: ProductListingId::new(),
            store_id,
            product_id,
            slug,
            title,
            short_description,
            long_description,
            is_published: false,
            is_featured: false,
            seo_title,
            seo_description,
            seo_keywords,
            sort_order,
            view_count: 0,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ProductListingId,
        store_id: StoreId,
        product_id: Uuid,
        slug: String,
        title: String,
        short_description: Option<String>,
        long_description: Option<String>,
        is_published: bool,
        is_featured: bool,
        seo_title: Option<String>,
        seo_description: Option<String>,
        seo_keywords: Vec<String>,
        sort_order: i32,
        view_count: i64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            product_id,
            slug,
            title,
            short_description,
            long_description,
            is_published,
            is_featured,
            seo_title,
            seo_description,
            seo_keywords,
            sort_order,
            view_count,
            created_at,
            updated_at,
        }
    }

    pub fn set_slug(&mut self, slug: String) -> Result<(), CatalogError> {
        if !is_valid_slug(&slug) {
            return Err(CatalogError::InvalidSlug);
        }
        self.slug = slug;
        self.touch();
        Ok(())
    }
    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.touch();
    }
    pub fn set_short_description(&mut self, value: Option<String>) {
        self.short_description = value;
        self.touch();
    }
    pub fn set_long_description(&mut self, value: Option<String>) {
        self.long_description = value;
        self.touch();
    }
    pub fn set_seo(
        &mut self,
        seo_title: Option<String>,
        seo_description: Option<String>,
        seo_keywords: Vec<String>,
    ) {
        self.seo_title = seo_title;
        self.seo_description = seo_description;
        self.seo_keywords = seo_keywords;
        self.touch();
    }
    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.touch();
    }
    pub fn publish(&mut self) {
        self.is_published = true;
        self.touch();
    }
    pub fn unpublish(&mut self) {
        self.is_published = false;
        self.touch();
    }
    pub fn set_featured(&mut self, is_featured: bool) {
        self.is_featured = is_featured;
        self.touch();
    }
    pub fn increment_view_count(&mut self) {
        self.view_count += 1;
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ProductListingId {
        self.id
    }
    pub fn store_id(&self) -> StoreId {
        self.store_id
    }
    pub fn product_id(&self) -> Uuid {
        self.product_id
    }
    pub fn slug(&self) -> &str {
        &self.slug
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn short_description(&self) -> Option<&str> {
        self.short_description.as_deref()
    }
    pub fn long_description(&self) -> Option<&str> {
        self.long_description.as_deref()
    }
    pub fn is_published(&self) -> bool {
        self.is_published
    }
    pub fn is_featured(&self) -> bool {
        self.is_featured
    }
    pub fn seo_title(&self) -> Option<&str> {
        self.seo_title.as_deref()
    }
    pub fn seo_description(&self) -> Option<&str> {
        self.seo_description.as_deref()
    }
    pub fn seo_keywords(&self) -> &[String] {
        &self.seo_keywords
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }
    pub fn view_count(&self) -> i64 {
        self.view_count
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_validation() {
        assert!(is_valid_slug("camiseta-negra-xl"));
        assert!(is_valid_slug("a1"));
        assert!(!is_valid_slug(""));
        assert!(!is_valid_slug("a"));
        assert!(!is_valid_slug("Camiseta")); // uppercase
        assert!(!is_valid_slug("foo--bar")); // double hyphen
        assert!(!is_valid_slug("-foo")); // leading
        assert!(!is_valid_slug("foo-")); // trailing
        assert!(!is_valid_slug("foo bar")); // space
    }
}
