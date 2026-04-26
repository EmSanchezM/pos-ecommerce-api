//! Review use cases — submit (with verified-purchase auto-detection),
//! approve (moderator), delete, list approved per listing.

use std::sync::Arc;

use uuid::Uuid;

use crate::CatalogError;
use crate::application::dtos::{ReviewListResponse, ReviewResponse, SubmitReviewCommand};
use crate::domain::entities::ProductReview;
use crate::domain::repositories::ProductReviewRepository;
use crate::domain::value_objects::{ProductListingId, ProductReviewId};
use identity::UserId;
use sales::CustomerId;

pub struct SubmitReviewUseCase {
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl SubmitReviewUseCase {
    pub fn new(review_repo: Arc<dyn ProductReviewRepository>) -> Self {
        Self { review_repo }
    }

    pub async fn execute(&self, cmd: SubmitReviewCommand) -> Result<ReviewResponse, CatalogError> {
        let listing_id = ProductListingId::from_uuid(cmd.listing_id);
        let customer_id = CustomerId::from_uuid(cmd.customer_id);

        // One review per (customer, listing).
        if self
            .review_repo
            .find_by_customer_and_listing(customer_id, listing_id)
            .await?
            .is_some()
        {
            return Err(CatalogError::DuplicateReview);
        }

        // Verified purchase = customer has at least one completed sale that
        // contains the underlying product.
        let verified = self
            .review_repo
            .customer_purchased_product(customer_id, listing_id)
            .await
            .unwrap_or(false);

        let review = ProductReview::create(
            listing_id,
            customer_id,
            cmd.rating,
            cmd.title,
            cmd.comment,
            verified,
        )?;
        self.review_repo.save(&review).await?;
        Ok(ReviewResponse::from(review))
    }
}

pub struct ApproveReviewUseCase {
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl ApproveReviewUseCase {
    pub fn new(review_repo: Arc<dyn ProductReviewRepository>) -> Self {
        Self { review_repo }
    }
    pub async fn execute(
        &self,
        review_id: Uuid,
        approver: UserId,
    ) -> Result<ReviewResponse, CatalogError> {
        let id = ProductReviewId::from_uuid(review_id);
        let mut review = self
            .review_repo
            .find_by_id(id)
            .await?
            .ok_or(CatalogError::ReviewNotFound(review_id))?;
        review.approve(approver)?;
        self.review_repo.update(&review).await?;
        Ok(ReviewResponse::from(review))
    }
}

pub struct DeleteReviewUseCase {
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl DeleteReviewUseCase {
    pub fn new(review_repo: Arc<dyn ProductReviewRepository>) -> Self {
        Self { review_repo }
    }
    pub async fn execute(&self, review_id: Uuid) -> Result<(), CatalogError> {
        let id = ProductReviewId::from_uuid(review_id);
        if self.review_repo.find_by_id(id).await?.is_none() {
            return Err(CatalogError::ReviewNotFound(review_id));
        }
        self.review_repo.delete(id).await
    }
}

pub struct ListReviewsUseCase {
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl ListReviewsUseCase {
    pub fn new(review_repo: Arc<dyn ProductReviewRepository>) -> Self {
        Self { review_repo }
    }

    pub async fn execute(
        &self,
        listing_id: Uuid,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<ReviewListResponse, CatalogError> {
        let lid = ProductListingId::from_uuid(listing_id);
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        let (rows, total) = self
            .review_repo
            .find_approved_by_listing(lid, page, page_size)
            .await?;
        let (avg, _) = self.review_repo.average_rating(lid).await?;
        Ok(ReviewListResponse {
            items: rows.into_iter().map(ReviewResponse::from).collect(),
            total,
            page,
            page_size,
            average_rating: avg,
        })
    }
}

pub struct ListPendingReviewsUseCase {
    review_repo: Arc<dyn ProductReviewRepository>,
}

impl ListPendingReviewsUseCase {
    pub fn new(review_repo: Arc<dyn ProductReviewRepository>) -> Self {
        Self { review_repo }
    }

    pub async fn execute(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<ReviewListResponse, CatalogError> {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        let (rows, total) = self.review_repo.find_pending(page, page_size).await?;
        Ok(ReviewListResponse {
            items: rows.into_iter().map(ReviewResponse::from).collect(),
            total,
            page,
            page_size,
            average_rating: rust_decimal::Decimal::ZERO,
        })
    }
}
