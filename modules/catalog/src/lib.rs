//! # Catalog Module
//!
//! Public/eCommerce-facing layer over `inventory.products`. Adds slug-based
//! routing, SEO metadata, image gallery, customer reviews (with moderation
//! and verified-purchase auto-detection), and wishlists.
//!
//! ## Image storage adapter
//!
//! Image bytes go through an [`ImageStorageAdapter`] selected by an
//! [`ImageStorageProvider`] row (per-store config, super-admin managed).
//! The `LocalServer` adapter writes to disk under `IMAGE_STORAGE_ROOT/
//! store_<store_id>/<uuid>.<ext>` and the API gateway serves the directory
//! via `tower-http::ServeDir`. S3, GCS, Cloudinary and Azure adapters are
//! stubs that fail loudly until their credentials and SDK calls are filled in.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::CatalogError;

// Domain - Value Objects
pub use domain::value_objects::{
    ImageStorageProviderId, ProductImageId, ProductListingId, ProductReviewId, StorageProviderType,
    WishlistId, WishlistItemId,
};

// Domain - Entities
pub use domain::entities::{
    ImageStorageProvider, ProductImage, ProductListing, ProductReview, Wishlist, WishlistItem,
};

// Domain - Repository traits
pub use domain::repositories::{
    CatalogFilter, ImageStorageProviderRepository, ProductImageRepository,
    ProductListingRepository, ProductReviewRepository, WishlistRepository,
};

// Application - DTOs + Use cases
pub use application::dtos::*;
pub use application::use_cases::*;

// Infrastructure
pub use infrastructure::adapters::{
    AzureBlobAdapter, CloudinaryAdapter, DefaultImageStorageRegistry, GcsAdapter,
    ImageStorageAdapter, ImageStorageRegistry, LocalServerAdapter, S3Adapter, UploadRequest,
    UploadResult,
};

pub use infrastructure::persistence::{
    PgImageStorageProviderRepository, PgProductImageRepository, PgProductListingRepository,
    PgProductReviewRepository, PgWishlistRepository,
};
