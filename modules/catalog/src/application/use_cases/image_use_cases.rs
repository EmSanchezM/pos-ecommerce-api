//! Image use cases — upload, update metadata, reorder, delete.
//!
//! Upload routes through the `ImageStorageRegistry` based on the listing's
//! store default `ImageStorageProvider`. Validates content-type and size at
//! the boundary.

use std::sync::Arc;

use uuid::Uuid;

use crate::CatalogError;
use crate::application::dtos::{
    ImageResponse, ReorderImagesCommand, UpdateImageCommand, UploadImageCommand,
};
use crate::domain::entities::ProductImage;
use crate::domain::repositories::{
    ImageStorageProviderRepository, ProductImageRepository, ProductListingRepository,
};
use crate::domain::value_objects::{ProductImageId, ProductListingId};
use crate::infrastructure::adapters::{ImageStorageRegistry, UploadRequest};

const MAX_IMAGES_PER_LISTING: i64 = 20;
const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024; // 10 MiB
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/avif",
];

pub struct UploadImageUseCase {
    listing_repo: Arc<dyn ProductListingRepository>,
    image_repo: Arc<dyn ProductImageRepository>,
    provider_repo: Arc<dyn ImageStorageProviderRepository>,
    registry: Arc<dyn ImageStorageRegistry>,
}

impl UploadImageUseCase {
    pub fn new(
        listing_repo: Arc<dyn ProductListingRepository>,
        image_repo: Arc<dyn ProductImageRepository>,
        provider_repo: Arc<dyn ImageStorageProviderRepository>,
        registry: Arc<dyn ImageStorageRegistry>,
    ) -> Self {
        Self {
            listing_repo,
            image_repo,
            provider_repo,
            registry,
        }
    }

    pub async fn execute(&self, cmd: UploadImageCommand) -> Result<ImageResponse, CatalogError> {
        let listing_id = ProductListingId::from_uuid(cmd.listing_id);
        let listing = self
            .listing_repo
            .find_by_id(listing_id)
            .await?
            .ok_or(CatalogError::ListingNotFound(cmd.listing_id))?;

        // Boundary validation.
        if !ALLOWED_CONTENT_TYPES.contains(&cmd.content_type.as_str()) {
            return Err(CatalogError::UnsupportedContentType(cmd.content_type));
        }
        let size = cmd.bytes.len() as u64;
        if size > MAX_IMAGE_BYTES {
            return Err(CatalogError::ImageTooLarge(size));
        }
        let count = self.image_repo.count_by_listing(listing_id).await?;
        if count >= MAX_IMAGES_PER_LISTING {
            return Err(CatalogError::MaxImagesExceeded(
                MAX_IMAGES_PER_LISTING as usize,
            ));
        }

        // Resolve the store's default storage provider.
        let provider = self
            .provider_repo
            .find_default(listing.store_id())
            .await?
            .ok_or_else(|| {
                CatalogError::NoDefaultStorageProvider(listing.store_id().into_uuid())
            })?;
        let adapter = self.registry.for_type(provider.provider_type());

        let result = adapter
            .upload(UploadRequest {
                store_id: listing.store_id().into_uuid(),
                bytes: cmd.bytes,
                original_filename: cmd.original_filename,
                content_type: cmd.content_type,
            })
            .await?;

        // First image of a listing gets is_primary=true automatically.
        let is_primary = cmd.is_primary || count == 0;

        let image = ProductImage::create(
            listing_id,
            result.url,
            result.storage_key,
            Some(provider.id()),
            cmd.alt_text,
            count as i32, // append at the end
            is_primary,
            Some(result.content_type),
            Some(result.size_bytes),
        );

        // Single-primary invariant: clear before insert (mirrors the lesson
        // learned with payment_gateways).
        if is_primary {
            self.image_repo
                .unset_primary_except(listing_id, image.id())
                .await?;
        }
        self.image_repo.save(&image).await?;
        Ok(ImageResponse::from(image))
    }
}

pub struct ListImagesUseCase {
    image_repo: Arc<dyn ProductImageRepository>,
}

impl ListImagesUseCase {
    pub fn new(image_repo: Arc<dyn ProductImageRepository>) -> Self {
        Self { image_repo }
    }
    pub async fn execute(&self, listing_id: Uuid) -> Result<Vec<ImageResponse>, CatalogError> {
        let images = self
            .image_repo
            .find_by_listing(ProductListingId::from_uuid(listing_id))
            .await?;
        Ok(images.into_iter().map(ImageResponse::from).collect())
    }
}

pub struct UpdateImageUseCase {
    image_repo: Arc<dyn ProductImageRepository>,
}

impl UpdateImageUseCase {
    pub fn new(image_repo: Arc<dyn ProductImageRepository>) -> Self {
        Self { image_repo }
    }
    pub async fn execute(&self, cmd: UpdateImageCommand) -> Result<ImageResponse, CatalogError> {
        let id = ProductImageId::from_uuid(cmd.image_id);
        let mut image = self
            .image_repo
            .find_by_id(id)
            .await?
            .ok_or(CatalogError::ImageNotFound(cmd.image_id))?;

        if let Some(alt) = cmd.alt_text {
            image.set_alt_text(alt);
        }
        if let Some(o) = cmd.sort_order {
            image.set_sort_order(o);
        }
        if let Some(p) = cmd.is_primary {
            if p {
                self.image_repo
                    .unset_primary_except(image.listing_id(), image.id())
                    .await?;
            }
            image.set_primary(p);
        }
        self.image_repo.save(&image).await?;
        Ok(ImageResponse::from(image))
    }
}

pub struct DeleteImageUseCase {
    image_repo: Arc<dyn ProductImageRepository>,
    provider_repo: Arc<dyn ImageStorageProviderRepository>,
    registry: Arc<dyn ImageStorageRegistry>,
}

impl DeleteImageUseCase {
    pub fn new(
        image_repo: Arc<dyn ProductImageRepository>,
        provider_repo: Arc<dyn ImageStorageProviderRepository>,
        registry: Arc<dyn ImageStorageRegistry>,
    ) -> Self {
        Self {
            image_repo,
            provider_repo,
            registry,
        }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), CatalogError> {
        let iid = ProductImageId::from_uuid(id);
        let image = self
            .image_repo
            .find_by_id(iid)
            .await?
            .ok_or(CatalogError::ImageNotFound(id))?;

        // Best-effort delete the underlying file. We log/swallow rather than
        // block the row deletion — orphan files can be GC'd later.
        if let Some(provider_id) = image.storage_provider_id()
            && let Some(provider) = self.provider_repo.find_by_id(provider_id).await?
        {
            let adapter = self.registry.for_type(provider.provider_type());
            let _ = adapter.delete(image.storage_key()).await;
        }
        self.image_repo.delete(iid).await
    }
}

pub struct ReorderImagesUseCase {
    image_repo: Arc<dyn ProductImageRepository>,
}

impl ReorderImagesUseCase {
    pub fn new(image_repo: Arc<dyn ProductImageRepository>) -> Self {
        Self { image_repo }
    }
    pub async fn execute(
        &self,
        listing_id: Uuid,
        cmd: ReorderImagesCommand,
    ) -> Result<(), CatalogError> {
        let ids: Vec<ProductImageId> = cmd
            .image_ids
            .into_iter()
            .map(ProductImageId::from_uuid)
            .collect();
        self.image_repo
            .reorder(ProductListingId::from_uuid(listing_id), ids)
            .await
    }
}
