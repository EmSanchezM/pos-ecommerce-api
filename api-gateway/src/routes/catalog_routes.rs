// Catalog routes.
//
// /api/v1/catalog/listings/...           — auth-required CRUD
// /api/v1/catalog/listings/{id}/images   — multipart upload
// /api/v1/catalog/listings/{id}/reviews  — submit review (customer auth)
// /api/v1/catalog/reviews/...            — moderate
// /api/v1/catalog/wishlist               — customer
// /api/v1/catalog/storage-providers      — super-admin gated
// /api/v1/catalog/public/...             — NO auth (SEO + anonymous browsing)

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

use crate::handlers::catalog::{
    add_to_wishlist_handler, approve_review_handler, configure_storage_provider_handler,
    create_listing_handler, delete_image_handler, delete_listing_handler, delete_review_handler,
    delete_storage_provider_handler, get_listing_detail_handler, get_listing_handler,
    get_wishlist_handler, list_images_handler, list_listings_handler, list_pending_reviews_handler,
    list_storage_providers_handler, public_featured_handler, public_get_by_slug_handler,
    public_listing_reviews_handler, public_search_listings_handler, publish_listing_handler,
    remove_from_wishlist_handler, reorder_images_handler, submit_review_handler,
    unpublish_listing_handler, update_image_handler, update_listing_handler,
    update_storage_provider_handler, upload_image_handler,
};
use crate::middleware::auth_middleware;
use crate::state::AppState;

pub fn catalog_listings_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_listing_handler).get(list_listings_handler))
        .route(
            "/{id}",
            get(get_listing_handler)
                .put(update_listing_handler)
                .delete(delete_listing_handler),
        )
        .route("/{id}/detail", get(get_listing_detail_handler))
        .route("/{id}/publish", put(publish_listing_handler))
        .route("/{id}/unpublish", put(unpublish_listing_handler))
        .route(
            "/{id}/images",
            post(upload_image_handler).get(list_images_handler),
        )
        .route("/{id}/images/reorder", put(reorder_images_handler))
        .route("/{id}/reviews", post(submit_review_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn catalog_images_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/{id}",
            put(update_image_handler).delete(delete_image_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn catalog_reviews_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/pending", get(list_pending_reviews_handler))
        .route("/{id}/approve", put(approve_review_handler))
        .route("/{id}", delete(delete_review_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn catalog_wishlist_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_wishlist_handler))
        .route("/items", post(add_to_wishlist_handler))
        .route("/items/remove", post(remove_from_wishlist_handler))
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

pub fn catalog_storage_providers_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(configure_storage_provider_handler).get(list_storage_providers_handler),
        )
        .route(
            "/{id}",
            put(update_storage_provider_handler).delete(delete_storage_provider_handler),
        )
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}

/// PUBLIC catalog routes (no auth). Mounted under `/api/v1/catalog/public`.
pub fn catalog_public_router() -> Router<AppState> {
    Router::new()
        .route("/listings", get(public_search_listings_handler))
        .route("/listings/featured", get(public_featured_handler))
        .route("/listings/by-slug/{slug}", get(public_get_by_slug_handler))
        .route(
            "/listings/{id}/reviews",
            get(public_listing_reviews_handler),
        )
}
