use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ProductListing;

#[derive(Debug, Deserialize)]
pub struct CreateListingCommand {
    pub store_id: Uuid,
    pub product_id: Uuid,
    pub slug: String,
    pub title: String,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    #[serde(default)]
    pub seo_keywords: Vec<String>,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateListingCommand {
    #[serde(default)]
    pub listing_id: Uuid,
    pub slug: Option<String>,
    pub title: Option<String>,
    pub short_description: Option<Option<String>>,
    pub long_description: Option<Option<String>>,
    pub seo_title: Option<Option<String>>,
    pub seo_description: Option<Option<String>>,
    pub seo_keywords: Option<Vec<String>>,
    pub sort_order: Option<i32>,
    pub is_featured: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SearchListingsQuery {
    pub store_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub is_published: Option<bool>,
    pub is_featured: Option<bool>,
    pub search: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    /// `price_asc`, `price_desc`, `newest`, `popular`
    pub sort_by: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListingResponse {
    pub id: Uuid,
    pub store_id: Uuid,
    pub product_id: Uuid,
    pub slug: String,
    pub title: String,
    pub short_description: Option<String>,
    pub long_description: Option<String>,
    pub is_published: bool,
    pub is_featured: bool,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Vec<String>,
    pub sort_order: i32,
    pub view_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProductListing> for ListingResponse {
    fn from(l: ProductListing) -> Self {
        Self {
            id: l.id().into_uuid(),
            store_id: l.store_id().into_uuid(),
            product_id: l.product_id(),
            slug: l.slug().to_string(),
            title: l.title().to_string(),
            short_description: l.short_description().map(str::to_string),
            long_description: l.long_description().map(str::to_string),
            is_published: l.is_published(),
            is_featured: l.is_featured(),
            seo_title: l.seo_title().map(str::to_string),
            seo_description: l.seo_description().map(str::to_string),
            seo_keywords: l.seo_keywords().to_vec(),
            sort_order: l.sort_order(),
            view_count: l.view_count(),
            created_at: l.created_at(),
            updated_at: l.updated_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListingListResponse {
    pub items: Vec<ListingResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Listing detail enriched with images and rating aggregate.
#[derive(Debug, Serialize)]
pub struct ListingDetailResponse {
    pub listing: ListingResponse,
    pub images: Vec<crate::application::dtos::ImageResponse>,
    pub average_rating: Decimal,
    pub review_count: i64,
}
