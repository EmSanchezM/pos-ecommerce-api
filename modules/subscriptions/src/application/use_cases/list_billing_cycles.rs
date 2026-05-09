use std::sync::Arc;

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::application::dtos::{
    BillingCycleResponse, ListBillingCyclesQuery, PaginatedBillingCycles,
};
use crate::domain::repositories::{BillingCycleRepository, SubscriptionRepository};

pub struct ListBillingCyclesUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
    cycle_repo: Arc<dyn BillingCycleRepository>,
}

impl ListBillingCyclesUseCase {
    pub fn new(
        sub_repo: Arc<dyn SubscriptionRepository>,
        cycle_repo: Arc<dyn BillingCycleRepository>,
    ) -> Self {
        Self {
            sub_repo,
            cycle_repo,
        }
    }

    pub async fn execute(
        &self,
        q: ListBillingCyclesQuery,
    ) -> Result<PaginatedBillingCycles, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(q.organization_id);
        let subscription = self
            .sub_repo
            .find_active_by_organization(org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(q.organization_id))?;

        let (cycles, total) = self
            .cycle_repo
            .find_by_subscription(subscription.id(), q.page, q.page_size)
            .await?;
        Ok(PaginatedBillingCycles {
            items: cycles.into_iter().map(BillingCycleResponse::from).collect(),
            total,
            page: q.page.max(1),
            page_size: q.page_size.clamp(1, 200),
        })
    }
}
