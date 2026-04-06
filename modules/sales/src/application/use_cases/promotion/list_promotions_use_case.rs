use std::sync::Arc;

use serde::Deserialize;

use crate::SalesError;
use crate::application::dtos::promotion::PromotionResponse;
use crate::domain::repositories::{PromotionFilter, PromotionRepository};

#[derive(Debug, Clone, Deserialize)]
pub struct ListPromotionsQuery {
    pub status: Option<String>,
    pub store_id: Option<uuid::Uuid>,
    pub search: Option<String>,
    pub page: i64,
    pub page_size: i64,
}

pub struct ListPromotionsUseCase<P: PromotionRepository> {
    promotion_repo: Arc<P>,
}

impl<P: PromotionRepository> ListPromotionsUseCase<P> {
    pub fn new(promotion_repo: Arc<P>) -> Self {
        Self { promotion_repo }
    }

    pub async fn execute(
        &self,
        query: ListPromotionsQuery,
    ) -> Result<(Vec<PromotionResponse>, i64), SalesError> {
        let filter = PromotionFilter {
            status: query.status,
            store_id: query.store_id,
            search: query.search,
        };

        let (promotions, total) = self
            .promotion_repo
            .find_paginated(filter, query.page, query.page_size)
            .await?;

        let responses: Vec<PromotionResponse> =
            promotions.iter().map(PromotionResponse::from).collect();

        Ok((responses, total))
    }
}
