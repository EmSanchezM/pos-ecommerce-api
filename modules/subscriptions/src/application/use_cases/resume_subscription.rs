use std::sync::Arc;

use sqlx::{Postgres, Transaction};
use tenancy::OrganizationId;

use crate::SubscriptionError;
use crate::application::dtos::{ResumeSubscriptionCommand, SubscriptionResponse};
use crate::domain::repositories::SubscriptionRepository;

pub struct ResumeSubscriptionUseCase {
    sub_repo: Arc<dyn SubscriptionRepository>,
}

impl ResumeSubscriptionUseCase {
    pub fn new(sub_repo: Arc<dyn SubscriptionRepository>) -> Self {
        Self { sub_repo }
    }

    pub async fn execute(
        &self,
        cmd: ResumeSubscriptionCommand,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(cmd.organization_id);
        let mut subscription = self
            .sub_repo
            .find_active_by_organization(org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(cmd.organization_id))?;

        subscription.resume()?;
        self.sub_repo.update_with_version(&subscription).await?;
        Ok(SubscriptionResponse::from(subscription))
    }

    /// Transactional path — load/resume/persist inside the caller's tx.
    pub async fn execute_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        cmd: ResumeSubscriptionCommand,
    ) -> Result<SubscriptionResponse, SubscriptionError> {
        let org_id = OrganizationId::from_uuid(cmd.organization_id);
        let mut subscription = self
            .sub_repo
            .find_active_by_organization_in_tx(tx, org_id)
            .await?
            .ok_or(SubscriptionError::SubscriptionNotFound(cmd.organization_id))?;

        subscription.resume()?;
        self.sub_repo
            .update_with_version_in_tx(tx, &subscription)
            .await?;
        Ok(SubscriptionResponse::from(subscription))
    }
}
