//! Paginated payout listing.

use std::sync::Arc;

use uuid::Uuid;

use crate::PaymentsError;
use crate::application::dtos::{PayoutListResponse, PayoutResponse};
use crate::domain::repositories::PayoutRepository;
use identity::StoreId;

pub struct ListPayoutsUseCase {
    payout_repo: Arc<dyn PayoutRepository>,
}

impl ListPayoutsUseCase {
    pub fn new(payout_repo: Arc<dyn PayoutRepository>) -> Self {
        Self { payout_repo }
    }

    pub async fn execute(
        &self,
        store_id: Uuid,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<PayoutListResponse, PaymentsError> {
        let store_id = StoreId::from_uuid(store_id);
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(50).clamp(1, 200);

        let (rows, total) = self
            .payout_repo
            .find_by_store(store_id, page, page_size)
            .await?;

        Ok(PayoutListResponse {
            items: rows.into_iter().map(PayoutResponse::from).collect(),
            total,
            page,
            page_size,
        })
    }
}
