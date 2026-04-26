//! Listing CRUD + search + detail.

use std::sync::Arc;

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::CatalogError;
use crate::application::dtos::{
    CreateListingCommand, ImageResponse, ListingDetailResponse, ListingListResponse,
    ListingResponse, SearchListingsQuery, UpdateListingCommand,
};
use crate::domain::entities::ProductListing;
use crate::domain::repositories::{
    CatalogFilter, ProductImageRepository, ProductListingRepository, ProductReviewRepository,
};
use crate::domain::value_objects::ProductListingId;
use identity::StoreId;

// =============================================================================
// CreateListing
// =============================================================================

pub struct CreateListingUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl CreateListingUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateListingCommand,
    ) -> Result<ListingResponse, CatalogError> {
        let store_id = StoreId::from_uuid(cmd.store_id);

        // Reject duplicate slug + duplicate product (DB also enforces).
        if self
            .listing_repo
            .find_by_slug(store_id, &cmd.slug)
            .await?
            .is_some()
        {
            return Err(CatalogError::DuplicateSlug(cmd.slug));
        }
        if self
            .listing_repo
            .find_by_product_id(cmd.product_id)
            .await?
            .is_some()
        {
            return Err(CatalogError::DuplicateProductListing(cmd.product_id));
        }

        let listing = ProductListing::create(
            store_id,
            cmd.product_id,
            cmd.slug,
            cmd.title,
            cmd.short_description,
            cmd.long_description,
            cmd.seo_title,
            cmd.seo_description,
            cmd.seo_keywords,
            cmd.sort_order,
        )?;
        self.listing_repo.save(&listing).await?;
        Ok(ListingResponse::from(listing))
    }
}

// =============================================================================
// UpdateListing
// =============================================================================

pub struct UpdateListingUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl UpdateListingUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateListingCommand,
    ) -> Result<ListingResponse, CatalogError> {
        let id = ProductListingId::from_uuid(cmd.listing_id);
        let mut listing = self
            .listing_repo
            .find_by_id(id)
            .await?
            .ok_or(CatalogError::ListingNotFound(cmd.listing_id))?;

        if let Some(slug) = cmd.slug {
            // Skip duplicate check when the slug isn't changing.
            if slug != listing.slug()
                && self
                    .listing_repo
                    .find_by_slug(listing.store_id(), &slug)
                    .await?
                    .is_some()
            {
                return Err(CatalogError::DuplicateSlug(slug));
            }
            listing.set_slug(slug)?;
        }
        if let Some(title) = cmd.title {
            listing.set_title(title);
        }
        if let Some(d) = cmd.short_description {
            listing.set_short_description(d);
        }
        if let Some(d) = cmd.long_description {
            listing.set_long_description(d);
        }
        if cmd.seo_title.is_some() || cmd.seo_description.is_some() || cmd.seo_keywords.is_some() {
            let seo_title = cmd
                .seo_title
                .unwrap_or_else(|| listing.seo_title().map(str::to_string));
            let seo_description = cmd
                .seo_description
                .unwrap_or_else(|| listing.seo_description().map(str::to_string));
            let seo_keywords = cmd
                .seo_keywords
                .unwrap_or_else(|| listing.seo_keywords().to_vec());
            listing.set_seo(seo_title, seo_description, seo_keywords);
        }
        if let Some(o) = cmd.sort_order {
            listing.set_sort_order(o);
        }
        if let Some(f) = cmd.is_featured {
            listing.set_featured(f);
        }
        self.listing_repo.update(&listing).await?;
        Ok(ListingResponse::from(listing))
    }
}

// =============================================================================
// PublishListing / UnpublishListing
// =============================================================================

pub struct PublishListingUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl PublishListingUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }
    pub async fn execute(&self, id: Uuid) -> Result<ListingResponse, CatalogError> {
        let mut listing = self
            .listing_repo
            .find_by_id(ProductListingId::from_uuid(id))
            .await?
            .ok_or(CatalogError::ListingNotFound(id))?;
        listing.publish();
        self.listing_repo.update(&listing).await?;
        Ok(ListingResponse::from(listing))
    }
}

pub struct UnpublishListingUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl UnpublishListingUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }
    pub async fn execute(&self, id: Uuid) -> Result<ListingResponse, CatalogError> {
        let mut listing = self
            .listing_repo
            .find_by_id(ProductListingId::from_uuid(id))
            .await?
            .ok_or(CatalogError::ListingNotFound(id))?;
        listing.unpublish();
        self.listing_repo.update(&listing).await?;
        Ok(ListingResponse::from(listing))
    }
}

// =============================================================================
// DeleteListing
// =============================================================================

pub struct DeleteListingUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl DeleteListingUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }
    pub async fn execute(&self, id: Uuid) -> Result<(), CatalogError> {
        let lid = ProductListingId::from_uuid(id);
        if self.listing_repo.find_by_id(lid).await?.is_none() {
            return Err(CatalogError::ListingNotFound(id));
        }
        self.listing_repo.delete(lid).await
    }
}

// =============================================================================
// SearchListings + Featured
// =============================================================================

pub struct SearchListingsUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl SearchListingsUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }
    pub async fn execute(
        &self,
        query: SearchListingsQuery,
    ) -> Result<ListingListResponse, CatalogError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(24).clamp(1, 200);
        let filter = CatalogFilter {
            store_id: query.store_id.map(StoreId::from_uuid),
            category_id: query.category_id,
            is_published: query.is_published,
            is_featured: query.is_featured,
            search: query.search,
            min_price: query.min_price,
            max_price: query.max_price,
            sort_by: query.sort_by,
        };
        let (rows, total) = self
            .listing_repo
            .find_paginated(filter, page, page_size)
            .await?;
        Ok(ListingListResponse {
            items: rows.into_iter().map(ListingResponse::from).collect(),
            total,
            page,
            page_size,
        })
    }
}

pub struct GetFeaturedListingsUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl GetFeaturedListingsUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }
    pub async fn execute(
        &self,
        store_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<ListingResponse>, CatalogError> {
        let limit = limit.unwrap_or(12).clamp(1, 100);
        let rows = self
            .listing_repo
            .find_featured(StoreId::from_uuid(store_id), limit)
            .await?;
        Ok(rows.into_iter().map(ListingResponse::from).collect())
    }
}

// =============================================================================
// GetListingDetail (by id) + GetBySlug + GetById
// =============================================================================

pub struct GetListingUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
}

impl GetListingUseCase {
    pub fn new(listing_repo: Arc<dyn ProductListingRepository>) -> Self {
        Self { listing_repo }
    }
    pub async fn execute(&self, id: Uuid) -> Result<ListingResponse, CatalogError> {
        let listing = self
            .listing_repo
            .find_by_id(ProductListingId::from_uuid(id))
            .await?
            .ok_or(CatalogError::ListingNotFound(id))?;
        Ok(ListingResponse::from(listing))
    }
}

pub struct GetListingDetailUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
    image_repo: Arc<dyn ProductImageRepository>,
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl GetListingDetailUseCase {
    pub fn new(
        listing_repo: Arc<dyn ProductListingRepository>,
        image_repo: Arc<dyn ProductImageRepository>,
        review_repo: Arc<dyn ProductReviewRepository>,
    ) -> Self {
        Self {
            listing_repo,
            image_repo,
            review_repo,
        }
    }

    pub async fn execute(&self, id: Uuid) -> Result<ListingDetailResponse, CatalogError> {
        let lid = ProductListingId::from_uuid(id);
        let listing = self
            .listing_repo
            .find_by_id(lid)
            .await?
            .ok_or(CatalogError::ListingNotFound(id))?;
        let images = self.image_repo.find_by_listing(lid).await?;
        let (avg, count) = self.review_repo.average_rating(lid).await?;
        Ok(ListingDetailResponse {
            listing: ListingResponse::from(listing),
            images: images.into_iter().map(ImageResponse::from).collect(),
            average_rating: avg,
            review_count: count,
        })
    }
}

/// Public-facing get-by-slug. Increments view_count as a side effect.
pub struct GetListingBySlugUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
    image_repo: Arc<dyn ProductImageRepository>,
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl GetListingBySlugUseCase {
    pub fn new(
        listing_repo: Arc<dyn ProductListingRepository>,
        image_repo: Arc<dyn ProductImageRepository>,
        review_repo: Arc<dyn ProductReviewRepository>,
    ) -> Self {
        Self {
            listing_repo,
            image_repo,
            review_repo,
        }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        slug: &str,
    ) -> Result<ListingDetailResponse, CatalogError> {
        let listing = self
            .listing_repo
            .find_by_slug(StoreId::from_uuid(store_id), slug)
            .await?
            .ok_or_else(|| CatalogError::ListingNotFound(Uuid::nil()))?;

        // Best-effort view count bump; not a hard failure if it errors.
        let _ = self.listing_repo.increment_view_count(listing.id()).await;

        let images = self.image_repo.find_by_listing(listing.id()).await?;
        let (avg, count) = self.review_repo.average_rating(listing.id()).await?;
        let mut response = ListingDetailResponse {
            listing: ListingResponse::from(listing),
            images: images.into_iter().map(ImageResponse::from).collect(),
            average_rating: avg,
            review_count: count,
        };
        // Reflect the increment we just performed.
        response.listing.view_count += 1;
        // Suppress unused.
        let _ = Decimal::ZERO;
        Ok(response)
    }
}
