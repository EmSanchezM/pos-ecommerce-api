use std::sync::Arc;

use crate::SubscriptionError;
use crate::application::dtos::{ListPlansQuery, PaginatedPlans, PlanResponse};
use crate::domain::repositories::SubscriptionPlanRepository;

pub struct ListPlansUseCase {
    plan_repo: Arc<dyn SubscriptionPlanRepository>,
}

impl ListPlansUseCase {
    pub fn new(plan_repo: Arc<dyn SubscriptionPlanRepository>) -> Self {
        Self { plan_repo }
    }

    /// Public listing — only `is_active = true` plans, ordered by sort.
    pub async fn list_active(&self) -> Result<Vec<PlanResponse>, SubscriptionError> {
        let plans = self.plan_repo.find_active().await?;
        Ok(plans.into_iter().map(PlanResponse::from).collect())
    }

    /// Admin paginated listing — every plan, active or not.
    pub async fn list_paginated(
        &self,
        q: ListPlansQuery,
    ) -> Result<PaginatedPlans, SubscriptionError> {
        let (plans, total) = self.plan_repo.list_paginated(q.page, q.page_size).await?;
        Ok(PaginatedPlans {
            items: plans.into_iter().map(PlanResponse::from).collect(),
            total,
            page: q.page.max(1),
            page_size: q.page_size.clamp(1, 200),
        })
    }
}
