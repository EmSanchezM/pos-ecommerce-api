use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::ProductReview;

#[derive(Debug, Deserialize)]
pub struct SubmitReviewCommand {
    #[serde(default)]
    pub listing_id: Uuid,
    pub customer_id: Uuid,
    pub rating: i16,
    pub title: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub customer_id: Uuid,
    pub rating: i16,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub is_verified_purchase: bool,
    pub is_approved: bool,
    pub approved_by_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<ProductReview> for ReviewResponse {
    fn from(r: ProductReview) -> Self {
        Self {
            id: r.id().into_uuid(),
            listing_id: r.listing_id().into_uuid(),
            customer_id: r.customer_id().into_uuid(),
            rating: r.rating(),
            title: r.title().map(str::to_string),
            comment: r.comment().map(str::to_string),
            is_verified_purchase: r.is_verified_purchase(),
            is_approved: r.is_approved(),
            approved_by_id: r.approved_by_id().map(|u| u.into_uuid()),
            approved_at: r.approved_at(),
            created_at: r.created_at(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ReviewListResponse {
    pub items: Vec<ReviewResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub average_rating: Decimal,
}
