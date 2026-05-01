//! ProductReview - one review per (listing, customer).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::CatalogError;
use crate::domain::value_objects::{ProductListingId, ProductReviewId};
use identity::UserId;
use sales::CustomerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductReview {
    id: ProductReviewId,
    listing_id: ProductListingId,
    customer_id: CustomerId,
    rating: i16,
    title: Option<String>,
    comment: Option<String>,
    is_verified_purchase: bool,
    is_approved: bool,
    approved_by_id: Option<UserId>,
    approved_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ProductReview {
    pub fn create(
        listing_id: ProductListingId,
        customer_id: CustomerId,
        rating: i16,
        title: Option<String>,
        comment: Option<String>,
        is_verified_purchase: bool,
    ) -> Result<Self, CatalogError> {
        if !(1..=5).contains(&rating) {
            return Err(CatalogError::InvalidRating);
        }
        let now = Utc::now();
        Ok(Self {
            id: ProductReviewId::new(),
            listing_id,
            customer_id,
            rating,
            title,
            comment,
            is_verified_purchase,
            is_approved: false,
            approved_by_id: None,
            approved_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ProductReviewId,
        listing_id: ProductListingId,
        customer_id: CustomerId,
        rating: i16,
        title: Option<String>,
        comment: Option<String>,
        is_verified_purchase: bool,
        is_approved: bool,
        approved_by_id: Option<UserId>,
        approved_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            listing_id,
            customer_id,
            rating,
            title,
            comment,
            is_verified_purchase,
            is_approved,
            approved_by_id,
            approved_at,
            created_at,
            updated_at,
        }
    }

    pub fn approve(&mut self, by: UserId) -> Result<(), CatalogError> {
        if self.is_approved {
            return Err(CatalogError::ReviewAlreadyApproved);
        }
        let now = Utc::now();
        self.is_approved = true;
        self.approved_by_id = Some(by);
        self.approved_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn id(&self) -> ProductReviewId {
        self.id
    }
    pub fn listing_id(&self) -> ProductListingId {
        self.listing_id
    }
    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }
    pub fn rating(&self) -> i16 {
        self.rating
    }
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
    pub fn is_verified_purchase(&self) -> bool {
        self.is_verified_purchase
    }
    pub fn is_approved(&self) -> bool {
        self.is_approved
    }
    pub fn approved_by_id(&self) -> Option<UserId> {
        self.approved_by_id
    }
    pub fn approved_at(&self) -> Option<DateTime<Utc>> {
        self.approved_at
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
