use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ProductImage;

/// Inline image upload payload — bytes carried via multipart at the API
/// layer; the use case sees them already decoded.
#[derive(Debug)]
pub struct UploadImageCommand {
    pub listing_id: Uuid,
    pub bytes: Vec<u8>,
    pub original_filename: String,
    pub content_type: String,
    pub alt_text: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateImageCommand {
    #[serde(default)]
    pub image_id: Uuid,
    pub alt_text: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderImagesCommand {
    pub image_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ImageResponse {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub url: String,
    pub storage_provider_id: Option<Uuid>,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub content_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl From<ProductImage> for ImageResponse {
    fn from(i: ProductImage) -> Self {
        Self {
            id: i.id().into_uuid(),
            listing_id: i.listing_id().into_uuid(),
            url: i.url().to_string(),
            storage_provider_id: i.storage_provider_uuid(),
            alt_text: i.alt_text().map(str::to_string),
            sort_order: i.sort_order(),
            is_primary: i.is_primary(),
            content_type: i.content_type().map(str::to_string),
            size_bytes: i.size_bytes(),
            created_at: i.created_at(),
        }
    }
}
