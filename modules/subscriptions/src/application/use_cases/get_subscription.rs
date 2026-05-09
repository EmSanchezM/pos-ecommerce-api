use std::sync::Arc;
use uuid::Uuid;

use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::application::dtos::SubscriptionResponse;
use crate::domain::repositories::SubscriptionRepository;

pub struct GetSubscriptionUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
}

impl GetSubscriptionUseCase {
    pub fn new(sub_repo: Arc<dyn SubscriptionRepository>) -> Self {
        Self { sub_repo }
    }

    /// Returns the org's single non-`Canceled` subscription, if any.
    pub async fn execute(
        &self,
        organization_id: Uuid,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(organization_id);
        let subscription = self
            .sub_repo
            .find_active_by_organization(org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(organization_id))?;
        Ok(SubscriptionResponse::from(subscription))
    }
}
