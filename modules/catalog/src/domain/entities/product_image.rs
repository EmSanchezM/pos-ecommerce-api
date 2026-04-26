//! ProductImage - one image in a listing's gallery.
//!
//! `storage_key` is the adapter-specific identifier needed to delete the file
//! later (filesystem path for LocalServer, S3 key for S3, public_id for
//! Cloudinary, etc). `url` is what the frontend renders.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{ImageStorageProviderId, ProductImageId, ProductListingId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImage {
    id: ProductImageId,
    listing_id: ProductListingId,
    url: String,
    storage_key: String,
    storage_provider_id: Option<ImageStorageProviderId>,
    alt_text: Option<String>,
    sort_order: i32,
    is_primary: bool,
    content_type: Option<String>,
    size_bytes: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ProductImage {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        listing_id: ProductListingId,
        url: String,
        storage_key: String,
        storage_provider_id: Option<ImageStorageProviderId>,
        alt_text: Option<String>,
        sort_order: i32,
        is_primary: bool,
        content_type: Option<String>,
        size_bytes: Option<i64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: ProductImageId::new(),
            listing_id,
            url,
            storage_key,
            storage_provider_id,
            alt_text,
            sort_order,
            is_primary,
            content_type,
            size_bytes,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ProductImageId,
        listing_id: ProductListingId,
        url: String,
        storage_key: String,
        storage_provider_id: Option<ImageStorageProviderId>,
        alt_text: Option<String>,
        sort_order: i32,
        is_primary: bool,
        content_type: Option<String>,
        size_bytes: Option<i64>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            listing_id,
            url,
            storage_key,
            storage_provider_id,
            alt_text,
            sort_order,
            is_primary,
            content_type,
            size_bytes,
            created_at,
            updated_at,
        }
    }

    pub fn set_alt_text(&mut self, alt_text: Option<String>) {
        self.alt_text = alt_text;
        self.touch();
    }
    pub fn set_sort_order(&mut self, sort_order: i32) {
        self.sort_order = sort_order;
        self.touch();
    }
    pub fn set_primary(&mut self, is_primary: bool) {
        self.is_primary = is_primary;
        self.touch();
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> ProductImageId {
        self.id
    }
    pub fn listing_id(&self) -> ProductListingId {
        self.listing_id
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn storage_key(&self) -> &str {
        &self.storage_key
    }
    pub fn storage_provider_id(&self) -> Option<ImageStorageProviderId> {
        self.storage_provider_id
    }
    pub fn alt_text(&self) -> Option<&str> {
        self.alt_text.as_deref()
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }
    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
    pub fn size_bytes(&self) -> Option<i64> {
        self.size_bytes
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
    /// Convenience for repository helpers and JSON.
    pub fn storage_provider_uuid(&self) -> Option<Uuid> {
        self.storage_provider_id.map(|p| p.into_uuid())
    }
}
