//! Catalog module error types.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CatalogError {
    // -------------------------------------------------------------------------
    // Listings
    // -------------------------------------------------------------------------
    #[error("Listing not found: {0}")]
    ListingNotFound(Uuid),

    #[error("Duplicate slug: {0}")]
    DuplicateSlug(String),

    #[error("Listing already exists for product: {0}")]
    DuplicateProductListing(Uuid),

    #[error("Product not found: {0}")]
    ProductNotFound(Uuid),

    #[error("Listing is unpublished")]
    ListingUnpublished,

    #[error("Invalid slug: must be lowercase letters/digits/hyphens only")]
    InvalidSlug,

    // -------------------------------------------------------------------------
    // Images
    // -------------------------------------------------------------------------
    #[error("Image not found: {0}")]
    ImageNotFound(Uuid),

    #[error("Image upload failed: {0}")]
    ImageUploadFailed(String),

    #[error("Maximum images exceeded ({0})")]
    MaxImagesExceeded(usize),

    #[error("Unsupported image content type: {0}")]
    UnsupportedContentType(String),

    #[error("Image too large: {0} bytes")]
    ImageTooLarge(u64),

    // -------------------------------------------------------------------------
    // Reviews
    // -------------------------------------------------------------------------
    #[error("Review not found: {0}")]
    ReviewNotFound(Uuid),

    #[error("Duplicate review by customer for this listing")]
    DuplicateReview,

    #[error("Invalid rating: must be 1..=5")]
    InvalidRating,

    #[error("Review already approved")]
    ReviewAlreadyApproved,

    // -------------------------------------------------------------------------
    // Wishlist
    // -------------------------------------------------------------------------
    #[error("Wishlist not found: {0}")]
    WishlistNotFound(Uuid),

    #[error("Wishlist item not found")]
    WishlistItemNotFound,

    #[error("Customer not found: {0}")]
    CustomerNotFound(Uuid),

    // -------------------------------------------------------------------------
    // Image storage providers
    // -------------------------------------------------------------------------
    #[error("Image storage provider not found: {0}")]
    StorageProviderNotFound(Uuid),

    #[error("No default image storage provider for store: {0}")]
    NoDefaultStorageProvider(Uuid),

    #[error("Duplicate storage provider name: {0}")]
    DuplicateProviderName(String),

    #[error("Invalid storage provider type")]
    InvalidStorageProviderType,

    #[error("Storage provider error: {0}")]
    StorageProviderError(String),

    // -------------------------------------------------------------------------
    // System
    // -------------------------------------------------------------------------
    #[error("IO error: {0}")]
    Io(String),

    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not implemented")]
    NotImplemented,
}
